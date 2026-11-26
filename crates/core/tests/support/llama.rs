use anyhow::{Context, Result, ensure};
use std::{env, net::SocketAddr, process::Stdio, time::Duration};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
    time::{sleep, timeout},
};

pub async fn start(alias: &str) -> Result<(Child, SocketAddr)> {
    let exe = env::var_os("LLAMA_TEST_SERVER").context("set LLAMA_TEST_SERVER to llama-server")?;
    let weights = env::var_os("LLAMA_TEST_GGUF")
        .context("set LLAMA_TEST_GGUF to a local instruction model")?;
    let addr = TcpListener::bind("127.0.0.1:0").await?.local_addr()?;
    let mut server = Command::new(exe)
        .arg("--model")
        .arg(weights)
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            &addr.port().to_string(),
            "--alias",
            alias,
            "--ctx-size",
            "1024",
            "--parallel",
            "1",
            "--threads",
            "2",
        ])
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let http = crabforge_test_support::Client::default();
    timeout(Duration::from_secs(60), async {
        loop {
            ensure!(
                server.try_wait()?.is_none(),
                "llama-server exited during startup"
            );
            if let Ok(r) = http.get(format!("http://{addr}/health")).send().await
                && r.status().is_success()
            {
                return Ok::<_, anyhow::Error>(());
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await??;
    Ok((server, addr))
}
