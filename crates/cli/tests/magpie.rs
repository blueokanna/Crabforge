use anyhow::{Context, Result, ensure};
use axum::{Json, Router, body::Bytes, http::HeaderMap, routing::post};
use crabforge_core::peer;
use serde_json::{Value, json};
use std::{env, fs, net::SocketAddr, path::Path, process::Stdio, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::TcpListener,
    process::Command,
    time::{sleep, timeout},
};

#[path = "../../core/tests/support/llama.rs"]
mod llama;

fn isolated(exe: &Path, root: &Path) -> Command {
    let mut command = Command::new(exe);
    command
        .env_clear()
        .current_dir(root)
        .kill_on_drop(true)
        .stdin(Stdio::null());
    for k in ["SystemRoot", "WINDIR", "SystemDrive", "ComSpec", "PATH"] {
        if let Some(v) = env::var_os(k) {
            command.env(k, v);
        }
    }
    for k in [
        "HOME",
        "USERPROFILE",
        "TEMP",
        "TMP",
        "APPDATA",
        "LOCALAPPDATA",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
        "XDG_STATE_HOME",
        "CRABFORGE_HOME",
    ] {
        command.env(k, root);
    }
    command
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "127.0.0.1,localhost");
    command
}

#[tokio::test]
#[ignore = "requires the Windows Magpie executable in MAGPIE_TEST_EXE"]
async fn quic_client_reaches_a_native_magpie_upstream() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let upstream = listener.local_addr()?;
    let vendor=Router::new().route("/v1/chat/completions",post(|headers:HeaderMap,body:Bytes|async move {
        assert_eq!(headers["authorization"],"Bearer fixture-key");
        let v:Value=serde_json::from_slice(&body).unwrap();
        assert_eq!(v["model"],"m");
        Json(json!({"id":"chatcmpl-fixture","object":"chat.completion","model":"m","choices":[{"index":0,"message":{"role":"assistant","content":"served through Magpie"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":3,"total_tokens":5}}))
    }));
    let task = tokio::spawn(async move { axum::serve(listener, vendor).await });
    let reply = via_magpie(upstream, "Hello").await?;
    assert_eq!(
        reply["choices"][0]["message"]["content"],
        "served through Magpie"
    );
    task.abort();
    let _ = task.await;
    Ok(())
}

#[tokio::test]
#[ignore = "requires MAGPIE_TEST_EXE, LLAMA_TEST_SERVER and LLAMA_TEST_GGUF"]
async fn quic_client_receives_real_model_output_via_magpie() -> Result<()> {
    let (mut server, addr) = llama::start("m").await?;
    let reply = via_magpie(addr, "Complete the sentence with one word: The sky is").await?;
    let text = reply["choices"][0]["message"]["content"]
        .as_str()
        .context("expected assistant text")?;
    let tokens = reply["usage"]["completion_tokens"]
        .as_u64()
        .context("missing generated token count")?;
    ensure!(
        !text.trim().is_empty() && (1..=16).contains(&tokens),
        "unexpected reply: {reply}"
    );
    println!("QUIC via Magpie: {tokens} output tokens, {text:?}");
    server.kill().await?;
    Ok(())
}

async fn via_magpie(upstream: SocketAddr, prompt: &str) -> Result<Value> {
    let original = env::var_os("MAGPIE_TEST_EXE")
        .context("set MAGPIE_TEST_EXE to the reference executable")?;
    let reference = tempfile::tempdir()?;
    let root = reference.path();
    let exe = root.join("magpie.exe");
    fs::copy(original, &exe)?;
    fs::write(root.join(".portable"), [])?;
    fs::create_dir(root.join("data"))?;
    fs::write(
        root.join("data/providers.json"),
        serde_json::to_vec(
            &json!({"providers":[{"id":"bench","name":"Bench","key":"fixture-key","chat":format!("http://{upstream}/v1"),"models":["m"]}]}),
        )?,
    )?;
    let addr = TcpListener::bind("127.0.0.1:0").await?.local_addr()?;
    let log = root.join("gateway-stderr.txt");
    let mut magpie = isolated(&exe, root)
        .env("MAGPIE_ADDR", addr.to_string())
        .env("MAGPIE_NO_STATS", "1")
        .arg("serve")
        .stdout(Stdio::null())
        .stderr(fs::File::create(&log)?)
        .spawn()?;
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(5))
        .build()?;
    timeout(Duration::from_secs(30), async {
        loop {
            ensure!(magpie.try_wait()?.is_none(), "reference gateway exited");
            if let Ok(r) = http
                .get(format!("http://{addr}/v1/models"))
                .bearer_auth("magpie")
                .send()
                .await
                && r.status().is_success()
            {
                break;
            }
            sleep(Duration::from_millis(100)).await;
        }
        Ok::<_, anyhow::Error>(())
    })
    .await
    .with_context(|| {
        format!(
            "waiting for Magpie on {addr}: {}",
            fs::read_to_string(&log).unwrap_or_default()
        )
    })??;
    let node = tempfile::tempdir()?;
    let client = tempfile::tempdir()?;
    let identity = peer::identity(client.path())?;
    let cfg = json!({"listen":"127.0.0.1:0","peer_listen":"/ip4/127.0.0.1/udp/0/quic-v1","allowed_peers":[identity.public().to_peer_id().to_string()],"providers":[{"id":"remote","key_env":"TEST_MAGPIE_KEY","endpoints":{"chat":format!("http://{addr}/v1")},"models":["bench/m"]}]});
    fs::write(node.path().join("settings.json"), serde_json::to_vec(&cfg)?)?;
    let cli = Path::new(env!("CARGO_BIN_EXE_carbforge-cli"));
    let mut gateway = isolated(cli, node.path())
        .env("TEST_MAGPIE_KEY", "magpie")
        .arg("serve")
        .stderr(Stdio::piped())
        .spawn()?;
    let mut lines = BufReader::new(gateway.stderr.take().unwrap()).lines();
    let remote = timeout(Duration::from_secs(15), async {
        while let Some(line) = lines.next_line().await? {
            if let Some(addr) = line.strip_prefix("P2P ") {
                return Ok(addr.to_owned());
            }
        }
        anyhow::bail!("gateway exited before listening on QUIC")
    })
    .await
    .context("waiting for Crabforge's QUIC listening address")??;
    let (local, tunnel) = peer::tunnel(identity, &remote, "127.0.0.1:0".parse()?).await?;
    let response = http
        .post(format!("http://{local}/v1/chat/completions"))
        .bearer_auth(crabforge_core::config::token(node.path())?)
        .body(
            json!({"model":"remote/bench/m","messages":[{"role":"user","content":prompt}],"temperature":0,"max_tokens":16})
                .to_string(),
        )
        .send()
        .await?
        .error_for_status()?;
    let reply: Value = serde_json::from_slice(&response.bytes().await?)?;
    tunnel.abort();
    let _ = tunnel.await;
    gateway.kill().await?;
    magpie.kill().await?;
    Ok(reply)
}
