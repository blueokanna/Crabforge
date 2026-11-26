use std::{collections::HashSet, net::SocketAddr, path::Path, time::Duration};

use anyhow::{Context, Result, bail, ensure};
use axum::Router;
use futures_util::StreamExt;
use libp2p::{
    Multiaddr, PeerId, StreamProtocol, Swarm, SwarmBuilder, allow_block_list, connection_limits,
    dcutr, identify,
    identity::Keypair,
    multiaddr::Protocol,
    relay,
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

#[cfg(test)]
mod tests;

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
            && parts.next() == Some(Protocol::QuicV1),
        "expected /ip4/IP/udp/PORT/quic-v1 or /ip6/IP/udp/PORT/quic-v1"
    );
    if let Some(relay_id) = parts.next() {
        ensure!(
            matches!(relay_id, Protocol::P2p(_))
                && parts.next() == Some(Protocol::P2pCircuit)
                && parts.next().is_none(),
            "relay address must end with /p2p/RELAY_ID/p2p-circuit"
        );
    }
    Ok(addr)
}

fn relay_peer(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().find_map(|p| match p {
        Protocol::P2p(peer) => Some(peer),
        _ => None,
    })
}

fn listen_for_punch(net: &mut Swarm<Network>, address: &Multiaddr) -> Result<()> {
    let listen = match address.iter().next() {
        Some(Protocol::Ip6(_)) => "/ip6/::/udp/0/quic-v1",
        _ => "/ip4/0.0.0.0/udp/0/quic-v1",
    };
    net.listen_on(listen.parse()?)?;
    Ok(())
}

#[derive(NetworkBehaviour)]
struct Network {
    access: allow_block_list::Behaviour<allow_block_list::AllowedPeers>,
    limits: connection_limits::Behaviour,
    relay: relay::client::Behaviour,
    identify: identify::Behaviour,
    punch: dcutr::Behaviour,
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
        // Simultaneous QUIC dialing can establish two direct connections while the circuit is open.
        .with_max_established_per_peer(Some(3));
    Ok(SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_quic_config(|mut c| {
            c.max_concurrent_stream_limit = 16;
            c.max_stream_data = 256 * 1024;
            c.max_connection_data = 1024 * 1024;
            c
        })
        .with_relay_client(libp2p::tls::Config::new, libp2p::yamux::Config::default)?
        .with_behaviour(|key, relay| Network {
            relay,
            identify: identify::Behaviour::new(identify::Config::new(
                "/crabforge/1.0".into(),
                key.public(),
            )),
            punch: dcutr::Behaviour::new(key.public().to_peer_id()),
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
    let clients: HashSet<_> = allowed.into_iter().collect();
    let relay = relay_peer(&addr);
    let mut net = network(key, clients.iter().copied().chain(relay))?;
    if relay.is_some() {
        listen_for_punch(&mut net, &addr)?;
    }
    let mut incoming = net.behaviour().streams.new_control().accept(PROTOCOL)?;
    let mut requested = net.listen_on(addr.clone())?;
    let address = timeout(HANDSHAKE, async {
        loop {
            match net.select_next_some().await {
                SwarmEvent::NewListenAddr {
                    listener_id,
                    mut address,
                } if listener_id == requested => {
                    if address.iter().last() == Some(Protocol::P2p(*net.local_peer_id())) {
                        address.pop();
                    }
                    return Ok(address);
                }
                SwarmEvent::ListenerClosed {
                    listener_id,
                    reason,
                    ..
                } if listener_id == requested => {
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
        let mut retry = tokio::time::interval(Duration::from_secs(5));
        let mut reserving = false;
        loop {
            tokio::select! {
                event = net.select_next_some() => {
                    if let SwarmEvent::ListenerClosed { listener_id, reason, .. } = event && listener_id == requested {
                        if relay.is_none() {
                            reason?;
                            bail!("P2P listener closed");
                        }
                        // Keep established direct sessions alive while renewing the reservation.
                        reserving = true;
                        retry.reset();
                    }
                }
                _ = retry.tick(), if reserving => {
                    if let Ok(listener) = net.listen_on(addr.clone()) {
                        requested = listener;
                        reserving = false;
                    }
                }
                Some((peer,stream))=incoming.next()=>if clients.contains(&peer) && connections.len()<32 {
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
    tunnel_with_relay(key, remote, None, bind).await
}

fn destination(remote: &str) -> Result<(Multiaddr, PeerId)> {
    let mut addr: Multiaddr = remote.parse()?;
    let Some(Protocol::P2p(id)) = addr.pop() else {
        bail!("remote address must end with /p2p/PEER_ID");
    };
    Ok((listen_address(&addr.to_string())?, id))
}

async fn dial(key: Keypair, addr: Multiaddr, id: PeerId) -> Result<Swarm<Network>> {
    let relay = relay_peer(&addr);
    let mut net = network(key, std::iter::once(id).chain(relay))?;
    if relay.is_some() {
        listen_for_punch(&mut net, &addr)?;
    }
    net.dial(addr.with(Protocol::P2p(id)))?;
    timeout(HANDSHAKE, async {
        loop {
            match net.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } if peer_id == id => return Ok(()),
                SwarmEvent::OutgoingConnectionError {
                    peer_id: Some(peer),
                    error,
                    ..
                } if peer == id => bail!("P2P dial: {error}"),
                _ => (),
            }
        }
    })
    .await
    .context("P2P dial timed out")??;
    Ok(net)
}

pub async fn tunnel_with_relay(
    key: Keypair,
    remote: &str,
    fallback: Option<&str>,
    bind: SocketAddr,
) -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    ensure!(bind.ip().is_loopback(), "tunnel must bind to loopback");
    let (addr, id) = destination(remote)?;
    let mut net = if let Some(fallback) = fallback {
        let (circuit, target) = destination(fallback)?;
        ensure!(
            target == id,
            "relay fallback must target the same gateway Peer ID"
        );
        ensure!(
            relay_peer(&addr).is_none() && relay_peer(&circuit).is_some(),
            "provide a direct QUIC address first and a relay circuit as fallback"
        );
        match dial(key.clone(), addr, id).await {
            Ok(net) => net,
            Err(error) => dial(key, circuit, id).await.with_context(|| {
                format!("relay fallback failed after direct connection failed: {error:#}")
            })?,
        }
    } else {
        dial(key, addr, id).await?
    };
    let listener = TcpListener::bind(bind).await?;
    let local = listener.local_addr()?;
    let control = net.behaviour().streams.new_control();
    let task = tokio::spawn(async move {
        let mut sessions = JoinSet::new();
        loop {
            tokio::select! {
                event=net.select_next_some()=>if let SwarmEvent::ConnectionClosed{peer_id,num_established:0,..}=event && peer_id==id {bail!("remote peer disconnected");},
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
