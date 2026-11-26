use anyhow::{Context, Result, ensure};
use crabforge_core::{config, peer, service::Service};
use futures_util::StreamExt;
use libp2p::{SwarmBuilder, identity::Keypair, relay, swarm::SwarmEvent};
use std::{net::SocketAddr, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::timeout,
};

async fn catalog(tunnel: SocketAddr, key: &str) -> Result<String> {
    let mut socket = TcpStream::connect(tunnel).await?;
    socket.write_all(format!("GET /v1/models HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {key}\r\nConnection: close\r\n\r\n").as_bytes()).await?;
    let mut response = String::new();
    timeout(Duration::from_secs(5), socket.read_to_string(&mut response)).await??;
    Ok(response)
}

#[tokio::test]
async fn ipv6_relay_reservation_carries_authenticated_gateway_requests() -> Result<()> {
    check_circuit(false).await
}

#[tokio::test]
async fn failed_direct_ipv6_dial_uses_the_same_gateways_relay() -> Result<()> {
    check_circuit(true).await
}

async fn check_circuit(fallback: bool) -> Result<()> {
    let mut relay = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| relay::Behaviour::new(key.public().to_peer_id(), Default::default()))?
        .build();
    relay.listen_on("/ip6/::1/udp/0/quic-v1".parse()?)?;
    let address = timeout(Duration::from_secs(5), async {
        loop {
            if let SwarmEvent::NewListenAddr { address, .. } = relay.select_next_some().await {
                break address;
            }
        }
    })
    .await?;
    relay.add_external_address(address.clone());
    let circuit = format!("{address}/p2p/{}/p2p-circuit", relay.local_peer_id());
    let (events, mut accepted) = mpsc::unbounded_channel();
    let relay_task = tokio::spawn(async move {
        loop {
            if let SwarmEvent::Behaviour(relay::Event::CircuitReqAccepted {
                src_peer_id,
                dst_peer_id,
            }) = relay.select_next_some().await
            {
                let _ = events.send((src_peer_id, dst_peer_id));
            }
        }
    });
    let root = tempfile::tempdir()?;
    let client = Keypair::generate_ed25519();
    let client_id = client.public().to_peer_id();
    let local: SocketAddr = "127.0.0.1:0".parse()?;
    let settings = format!(
        r#"{{"listen":"{local}","peer_listen":"{circuit}","allowed_peers":["{client_id}"]}}"#
    );
    config::save(root.path(), &config::snapshot(root.path())?, &settings)?;
    let gateway = Service::start(root.path()).await?;
    let gateway_id = peer::identity(root.path())?.public().to_peer_id();
    let address = &gateway
        .peer
        .as_ref()
        .context("missing relay listener")?
        .address;
    assert_eq!(address.to_string(), format!("{circuit}/p2p/{gateway_id}"));
    let remote = address.to_string();
    let (addr, tunnel) = if fallback {
        let unavailable = tokio::net::UdpSocket::bind("[::1]:0").await?;
        let port = unavailable.local_addr()?.port();
        let direct = format!("/ip6/::1/udp/{port}/quic-v1/p2p/{gateway_id}");
        let wrong_destination = format!("{circuit}/p2p/{client_id}");
        let err = peer::tunnel_with_relay(client.clone(), &direct, Some(&wrong_destination), local)
            .await
            .unwrap_err();
        ensure!(err.to_string().contains("same gateway Peer ID"));
        peer::tunnel_with_relay(client, &direct, Some(&remote), local).await?
    } else {
        peer::tunnel(client, &remote, local).await?
    };
    assert_eq!(
        timeout(Duration::from_secs(5), accepted.recv()).await?,
        Some((client_id, gateway_id))
    );
    ensure!(catalog(addr, "invalid").await?.starts_with("HTTP/1.1 401"));
    ensure!(
        catalog(addr, &config::token(root.path())?)
            .await?
            .starts_with("HTTP/1.1 200")
    );
    tunnel.abort();
    gateway.stop().await;
    relay_task.abort();
    Ok(())
}
