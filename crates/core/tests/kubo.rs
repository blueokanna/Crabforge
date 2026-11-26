use anyhow::{Context, Result, ensure};
use crabforge_core::{config, peer, service::Service};
use nextjson::{Value, json};
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    net::TcpListener,
    process::Command,
    time::{sleep, timeout},
};

const PROTOCOL: &str = "/crabforge/gateway/1.0.0";

struct Kubo {
    exe: PathBuf,
    repo: PathBuf,
}

impl Kubo {
    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.exe);
        cmd.env("IPFS_PATH", &self.repo).kill_on_drop(true);
        cmd
    }

    async fn run(&self, args: &[&str]) -> Result<String> {
        let out = timeout(Duration::from_secs(20), self.command().args(args).output()).await??;
        ensure!(
            out.status.success(),
            "ipfs {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        Ok(String::from_utf8(out.stdout)?)
    }
}

async fn check_catalog(addr: std::net::SocketAddr, key: &str) -> Result<()> {
    let client = crabforge_test_support::Client::with_timeout(Duration::from_secs(8));
    let url = format!("http://{addr}/v1/models");
    assert_eq!(client.get(&url).send().await?.status(), 401);
    let response = client
        .get(url)
        .bearer_auth(key)
        .send()
        .await?
        .error_for_status()?;
    let list: Value = nextjson::from_slice(&response.bytes().await?).map_err(anyhow::Error::msg)?;
    assert!(
        list["data"]
            .as_array()
            .context("missing model list")?
            .iter()
            .any(|m| m["id"] == json!("local/interop"))
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires KUBO_TEST_EXE pointing to Kubo ipfs"]
async fn go_and_rust_peers_exchange_authenticated_http_in_both_directions() -> Result<()> {
    let root = tempfile::tempdir()?;
    let kubo = Kubo {
        exe: Path::new(&env::var_os("KUBO_TEST_EXE").context("set KUBO_TEST_EXE to ipfs")?).into(),
        repo: root.path().join("ipfs"),
    };
    kubo.run(&["init", "--profile=test,autoconf-off"]).await?;
    let path = kubo.repo.join("config");
    let mut cfg: Value =
        nextjson::from_slice(&std::fs::read(&path)?).map_err(anyhow::Error::msg)?;
    cfg.get_mut("Experimental")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("Libp2pStreamMounting".into(), true.into());
    cfg.get_mut("Addresses")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("Swarm".into(), json!(["/ip4/127.0.0.1/udp/0/quic-v1"]));
    let peer_id = cfg["Identity"]["PeerID"]
        .as_str()
        .context("missing Kubo peer ID")?;
    std::fs::write(path, nextjson::to_vec(&cfg).map_err(anyhow::Error::msg)?)?;

    let mut daemon = kubo
        .command()
        .args(["daemon", "--routing=none"])
        .stdout(Stdio::null())
        .spawn()?;
    timeout(Duration::from_secs(20), async {
        while !kubo.repo.join("api").is_file() {
            ensure!(daemon.try_wait()?.is_none(), "Kubo exited during startup");
            sleep(Duration::from_millis(100)).await;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await??;

    let home = root.path().join("gateway");
    let cfg = json!({"listen":"127.0.0.1:0", "peer_listen":"/ip4/127.0.0.1/udp/0/quic-v1", "allowed_peers":[peer_id],
        "providers":[{"id":"local","endpoints":{"chat":"http://127.0.0.1:1"},"models":["interop"]}]});
    config::save(&home, &config::snapshot(&home)?, &cfg.to_string())?;
    let gateway = Service::start(&home).await?;
    let key = config::token(&home)?;
    let rust_addr = gateway
        .peer
        .as_ref()
        .context("missing QUIC listener")?
        .address
        .to_string();
    kubo.run(&["swarm", "connect", &rust_addr]).await?;

    let forward = TcpListener::bind("127.0.0.1:0").await?.local_addr()?;
    let local = format!("/ip4/127.0.0.1/tcp/{}", forward.port());
    let remote = format!("/p2p/{}", peer::identity(&home)?.public().to_peer_id());
    kubo.run(&[
        "p2p",
        "forward",
        "--allow-custom-protocol",
        PROTOCOL,
        &local,
        &remote,
    ])
    .await?;
    check_catalog(forward, &key).await?;

    let target = format!("/ip4/127.0.0.1/tcp/{}", gateway.addr.port());
    kubo.run(&[
        "p2p",
        "listen",
        "--allow-custom-protocol",
        PROTOCOL,
        &target,
    ])
    .await?;
    let addresses = kubo.run(&["swarm", "addrs", "listen"]).await?;
    let quic = addresses
        .lines()
        .map(str::trim)
        .find(|s| s.ends_with("/quic-v1"))
        .context("missing Kubo QUIC address")?;
    let (reverse, tunnel) = peer::tunnel(
        peer::identity(&root.path().join("client"))?,
        &format!("{quic}/p2p/{peer_id}"),
        "127.0.0.1:0".parse()?,
    )
    .await?;
    check_catalog(reverse, &key).await?;
    tunnel.abort();
    let _ = tunnel.await;
    gateway.stop().await;
    kubo.run(&["shutdown"]).await?;
    timeout(Duration::from_secs(10), daemon.wait()).await??;
    Ok(())
}
