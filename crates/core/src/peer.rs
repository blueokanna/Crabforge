use std::{net::SocketAddr, path::Path, time::Duration};

use anyhow::{Context, Result, bail, ensure};
use axum::Router;
use futures_util::StreamExt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder, allow_block_list, connection_limits,
    identity::Keypair,
    multiaddr::Protocol,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use tokio::{
    net::TcpListener,
    task::{JoinHandle, JoinSet},
    time::timeout,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;

const PROTOCOL: StreamProtocol = StreamProtocol::new("/crabforge/gateway/1.0.0");
const HANDSHAKE: Duration = Duration::from_secs(15);
const STREAM_LIFETIME: Duration = Duration::from_secs(3600);

pub fn identity(dir: &Path) -> Result<Keypair> {
    let codec = rustbinary::options()
        .with_limit(4096)
        .with_collection_limit(256);
    let bytes = crate::config::private_file(dir, "peer.identity", || {
        let key = Keypair::generate_ed25519().to_protobuf_encoding()?;
        codec
            .serialize(&key)
            .map_err(|e| anyhow::anyhow!("encode identity: {e}"))
    })?;
    let key: Vec<u8> = codec
        .deserialize(&bytes)
        .map_err(|e| anyhow::anyhow!("invalid identity: {e}"))?;
    Ok(Keypair::from_protobuf_encoding(&key)?)
}

pub fn listen_address(s: &str) -> Result<Multiaddr> {
    let addr: Multiaddr = s.parse()?;
    let mut parts = addr.iter();
    ensure!(
        matches!(parts.next(), Some(Protocol::Ip4(_) | Protocol::Ip6(_)))
            && matches!(parts.next(), Some(Protocol::Udp(_)))
            && parts.next() == Some(Protocol::QuicV1)
            && parts.next().is_none(),
        "expected /ip4/IP/udp/PORT/quic-v1 or /ip6/IP/udp/PORT/quic-v1"
    );
    Ok(addr)
}

#[derive(NetworkBehaviour)]
struct Network {
    access: allow_block_list::Behaviour<allow_block_list::AllowedPeers>,
    limits: connection_limits::Behaviour,
    streams: libp2p_stream::Behaviour,
}

fn network(key: Keypair, allowed: impl IntoIterator<Item = PeerId>) -> Result<Swarm<Network>> {
    let mut access = allow_block_list::Behaviour::default();
    for peer in allowed {
        access.allow_peer(peer);
    }
    let limits = connection_limits::ConnectionLimits::default()
        .with_max_pending_incoming(Some(16))
        .with_max_established(Some(32))
        .with_max_established_per_peer(Some(2));
    Ok(SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_quic_config(|mut c| {
            c.max_concurrent_stream_limit = 16;
            c.max_stream_data = 256 * 1024;
            c.max_connection_data = 1024 * 1024;
            c
        })
        .with_behaviour(|_| Network {
            access,
            limits: connection_limits::Behaviour::new(limits),
            streams: libp2p_stream::Behaviour::new(),
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(3600)))
        .build())
}

pub struct PeerServer {
    pub address: Multiaddr,
    pub task: JoinHandle<Result<()>>,
}

pub async fn serve(
    key: Keypair,
    addr: Multiaddr,
    allowed: Vec<PeerId>,
    app: Router,
) -> Result<PeerServer> {
    let mut net = network(key, allowed)?;
    let mut incoming = net.behaviour().streams.new_control().accept(PROTOCOL)?;
    net.listen_on(addr)?;
    let address = timeout(HANDSHAKE, async {
        loop {
            match net.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => return Ok(address),
                SwarmEvent::ListenerClosed { reason, .. } => {
                    reason?;
                    bail!("P2P listener closed");
                }
                _ => (),
            }
        }
    })
    .await
    .context("P2P listen timed out")??
    .with(Protocol::P2p(*net.local_peer_id()));
    let task = tokio::spawn(async move {
        let mut connections = JoinSet::new();
        loop {
            tokio::select! {
                event=net.select_next_some()=> if let SwarmEvent::ListenerClosed{reason,..}=event {reason?; bail!("P2P listener closed");},
                Some((_,stream))=incoming.next()=>if connections.len()<32 {
                    connections.spawn(crate::service::serve_connection(stream.compat(),app.clone()));
                },
                _=connections.join_next(), if !connections.is_empty()=>(),
            }
        }
    });
    Ok(PeerServer { address, task })
}

pub async fn tunnel(
    key: Keypair,
    remote: &str,
    bind: SocketAddr,
) -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    ensure!(bind.ip().is_loopback(), "tunnel must bind to loopback");
    let mut addr: Multiaddr = remote.parse()?;
    let Some(Protocol::P2p(id)) = addr.pop() else {
        bail!("remote address must end with /p2p/PEER_ID");
    };
    listen_address(&addr.to_string())?;
    let mut net = network(key, [id])?;
    net.dial(addr.with(Protocol::P2p(id)))?;
    timeout(HANDSHAKE, async {
        loop {
            match net.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == id => return Ok(()),
                SwarmEvent::OutgoingConnectionError { error, .. } => bail!("P2P dial: {error}"),
                _ => (),
            }
        }
    })
    .await
    .context("P2P dial timed out")??;
    let listener = TcpListener::bind(bind).await?;
    let local = listener.local_addr()?;
    let control = net.behaviour().streams.new_control();
    let task = tokio::spawn(async move {
        let mut sessions = JoinSet::new();
        loop {
            tokio::select! {
                event=net.select_next_some()=>if let SwarmEvent::ConnectionClosed{num_established:0,..}=event {bail!("remote peer disconnected");},
                conn=listener.accept(), if sessions.len()<16=>{
                    let (mut socket,_)=conn?; let mut control=control.clone();
                    sessions.spawn(async move {
                        if let Ok(Ok(stream))=timeout(HANDSHAKE,control.open_stream(id,PROTOCOL)).await {
                            let _=timeout(STREAM_LIFETIME,tokio::io::copy_bidirectional(&mut socket,&mut stream.compat())).await;
                        }
                    });
                },
                _=sessions.join_next(), if !sessions.is_empty()=>(),
            }
        }
    });
    Ok((local, task))
}
