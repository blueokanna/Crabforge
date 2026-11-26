use super::*;
use crate::{config, service::Service};
use futures_util::{AsyncReadExt, AsyncWriteExt};

#[derive(NetworkBehaviour)]
struct TestRelay {
    relay: relay::Behaviour,
    identify: identify::Behaviour,
}

fn new_relay(key: Keypair) -> Result<Swarm<TestRelay>> {
    Ok(SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| TestRelay {
            relay: relay::Behaviour::new(key.public().to_peer_id(), Default::default()),
            identify: identify::Behaviour::new(identify::Config::new(
                "/fixture".into(),
                key.public(),
            )),
        })?
        .build())
}

#[tokio::test]
async fn dcutr_upgrades_an_ipv6_circuit_to_authenticated_direct_quic() -> Result<()> {
    let relay_key = Keypair::generate_ed25519();
    let mut relay = new_relay(relay_key.clone())?;
    let relay_id = *relay.local_peer_id();
    relay.listen_on("/ip6/::1/udp/0/quic-v1".parse()?)?;
    let relay_addr = timeout(HANDSHAKE, async {
        loop {
            if let SwarmEvent::NewListenAddr { address, .. } = relay.select_next_some().await {
                break address;
            }
        }
    })
    .await?;
    relay.add_external_address(relay_addr.clone());
    let relay_task = tokio::spawn(async move {
        loop {
            relay.select_next_some().await;
        }
    });

    let key = Keypair::generate_ed25519();
    let client_id = key.public().to_peer_id();
    let root = tempfile::tempdir()?;
    let cfg = config::Config {
        listen: "127.0.0.1:0".parse()?,
        peer_listen: format!("{relay_addr}/p2p/{relay_id}/p2p-circuit"),
        allowed_peers: vec![client_id.to_string()],
        ..Default::default()
    };
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &serde_json::to_string(&cfg)?,
    )?;
    let server = Service::start(root.path()).await?;
    let gateway_id = identity(root.path())?.public().to_peer_id();
    let mut client = network(key, [relay_id, gateway_id])?;
    listen_for_punch(&mut client, &relay_addr)?;
    client.dial(server.peer.as_ref().unwrap().address.clone())?;
    let circuit = timeout(HANDSHAKE, async {
        let mut connections = HashSet::new();
        let mut upgraded = None;
        let mut circuit = None;
        loop {
            match client.select_next_some().await {
                SwarmEvent::ConnectionEstablished {
                    peer_id,
                    connection_id,
                    endpoint,
                    ..
                } if peer_id == gateway_id => {
                    if endpoint.is_relayed() {
                        circuit = Some(connection_id);
                    } else {
                        connections.insert(connection_id);
                    }
                }
                SwarmEvent::Behaviour(NetworkEvent::Punch(event))
                    if event.remote_peer_id == gateway_id =>
                {
                    upgraded = Some(event.result.context("DCUtR negotiation failed")?);
                }
                _ => (),
            }
            if let (Some(id), Some(circuit_id)) = (upgraded, circuit)
                && connections.contains(&id)
            {
                return Ok::<_, anyhow::Error>(circuit_id);
            }
        }
    })
    .await
    .context("no DCUtR upgrade observed")??;

    // Losing the relay leaves only direct connections for the next gateway request.
    relay_task.abort();
    let _ = relay_task.await;
    timeout(HANDSHAKE, async {
        loop {
            if let SwarmEvent::ConnectionClosed {
                peer_id,
                endpoint,
                connection_id,
                ..
            } = client.select_next_some().await
                && peer_id == gateway_id
                && endpoint.is_relayed()
            {
                assert_eq!(connection_id, circuit);
                break;
            }
        }
    })
    .await
    .context("relay circuit remained open")?;
    let mut control = client.behaviour().streams.new_control();
    let task = tokio::spawn(async move {
        loop {
            client.select_next_some().await;
        }
    });
    let key = config::token(root.path())?;
    timeout(HANDSHAKE, async {
        let mut stream = control.open_stream(gateway_id, PROTOCOL).await?;
        stream.write_all(format!("GET /v1/models HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {key}\r\nConnection: close\r\n\r\n").as_bytes()).await?;
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        ensure!(response.starts_with("HTTP/1.1 200"));
        Ok::<_,anyhow::Error>(())
    }).await??;
    let mut restarted = new_relay(relay_key)?;
    // Quinn can retain the old UDP socket briefly while draining its connections.
    let deadline = tokio::time::Instant::now() + HANDSHAKE;
    loop {
        match restarted.listen_on(relay_addr.clone()) {
            Ok(_) => break,
            Err(error) if tokio::time::Instant::now() >= deadline => return Err(error.into()),
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    restarted.add_external_address(relay_addr);
    timeout(Duration::from_secs(30), async {
        loop {
            if let SwarmEvent::Behaviour(TestRelayEvent::Relay(
                relay::Event::ReservationReqAccepted { src_peer_id, .. },
            )) = restarted.select_next_some().await
                && src_peer_id == gateway_id
            {
                break;
            }
        }
    })
    .await
    .context("gateway did not reserve again after relay restart")?;
    task.abort();
    server.stop().await;
    Ok(())
}
