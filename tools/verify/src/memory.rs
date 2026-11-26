use anyhow::{Context, Result, bail, ensure};
use clap::ValueEnum;
use courierust::{
    courierust_body::Body,
    courierust_client::{Client, ClientConfig},
    courierust_http::{Method, request::Request, response::Response},
    courierust_server::Server,
};
use nextjson::{Value, json};
use std::{
    collections::HashSet,
    env, fs,
    mem::size_of,
    path::Path,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{
    net::TcpListener,
    process::{Child, Command},
    time::sleep,
};
use windows_sys::Win32::{
    Foundation::*,
    System::{Diagnostics::ToolHelp::*, ProcessStatus::*, Threading::*},
    UI::WindowsAndMessaging::*,
};

struct OwnedHandle(HANDLE);
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

fn counters(pid: u32) -> Result<(usize, usize)> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, pid);
        ensure!(!handle.is_null(), "cannot open process {pid}");
        let handle = OwnedHandle(handle);
        let mut data = PROCESS_MEMORY_COUNTERS_EX::default();
        let bytes = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        ensure!(
            K32GetProcessMemoryInfo(handle.0, (&raw mut data).cast(), bytes) != 0,
            "process {pid} disappeared"
        );
        Ok((data.WorkingSetSize, data.PrivateUsage))
    }
}

async fn sample(ids: &[u32]) -> Result<(f64, f64)> {
    let mut working = [0; 5];
    let mut private = [0; 5];
    for i in 0..5 {
        for &id in ids {
            let (w, p) = counters(id)?;
            working[i] += w;
            private[i] += p;
        }
        sleep(Duration::from_millis(200)).await;
    }
    working.sort_unstable();
    private.sort_unstable();
    Ok((working[2] as f64 / 1048576., private[2] as f64 / 1048576.))
}

fn webviews(root: u32) -> Result<Vec<u32>> {
    let mut entries = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        ensure!(snap != INVALID_HANDLE_VALUE, "cannot enumerate processes");
        let snap = OwnedHandle(snap);
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut valid = Process32FirstW(snap.0, &mut entry);
        let mut magpie = false;
        while valid != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
            if name.eq_ignore_ascii_case("msedgewebview2.exe") {
                entries.push((entry.th32ParentProcessID, entry.th32ProcessID));
            }
            if entry.th32ProcessID == root {
                magpie = name.eq_ignore_ascii_case("magpie.exe");
            }
            valid = Process32NextW(snap.0, &mut entry);
        }
        ensure!(magpie, "expected the PID of a Magpie window");
    }
    let mut tree = HashSet::from([root]);
    loop {
        let before = tree.len();
        for &(parent, pid) in &entries {
            if tree.contains(&parent) {
                tree.insert(pid);
            }
        }
        if tree.len() == before {
            break;
        }
    }
    Ok(tree.into_iter().collect())
}

fn window(pid: u32) -> HWND {
    unsafe {
        let mut hwnd = GetTopWindow(std::ptr::null_mut());
        while !hwnd.is_null() {
            let mut owner = 0;
            GetWindowThreadProcessId(hwnd, &mut owner);
            if owner == pid && IsWindowVisible(hwnd) != 0 && GetWindow(hwnd, GW_OWNER).is_null() {
                break;
            }
            hwnd = GetWindow(hwnd, GW_HWNDNEXT);
        }
        hwnd
    }
}

async fn row(label: &str, round: u8, ids: &[u32]) -> Result<()> {
    let (w, p) = sample(ids).await?;
    println!("{label},{round},{},{w:.2},{p:.2}", ids.len());
    Ok(())
}

pub async fn desktop(gui: &Path, reference: Option<u32>, rounds: u8) -> Result<()> {
    println!("App,Round,Processes,WorkingSetMiB,PrivateMiB");
    for round in 1..=rounds {
        let dir = tempfile::tempdir()?;
        let mut app = Command::new(gui)
            .env("CRABFORGE_HOME", dir.path())
            .kill_on_drop(true)
            .spawn()?;
        let pid = app.id().context("desktop exited")?;
        let start = Instant::now();
        while !dir.path().join("peer.identity").is_file() || window(pid).is_null() {
            check_ready(&mut app, start, Duration::from_secs(30))?;
            sleep(Duration::from_millis(100)).await;
        }
        sleep(Duration::from_secs(3)).await;
        row("Crabforge", round, &[pid]).await?;
        if let Some(pid) = reference {
            ensure!(!window(pid).is_null(), "Magpie has no visible window");
            let tree = webviews(pid)?;
            row("Magpie main", round, &[pid]).await?;
            row("Magpie with WebView2", round, &tree).await?;
        }
        unsafe {
            PostMessageW(window(pid), WM_CLOSE, 0, 0);
        }
        if tokio::time::timeout(Duration::from_secs(3), app.wait())
            .await
            .is_err()
        {
            app.kill().await?;
        }
    }
    Ok(())
}

fn check_ready(child: &mut Child, start: Instant, limit: Duration) -> Result<()> {
    if let Some(status) = child.try_wait()? {
        bail!("process exited before readiness: {status}");
    }
    ensure!(start.elapsed() < limit, "readiness check timed out");
    Ok(())
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Workload {
    Catalog,
    Chat,
}

pub async fn gateway(cli: &Path, magpie: &Path, work: Workload, rounds: u8) -> Result<()> {
    let server = Server::bind("127.0.0.1:0")?;
    let base = format!("http://{}/v1", server.local_addr()?);
    let fixture = server.serve_background(|req: Request<Body>| {
        let data = match (&req.method, req.uri.path()) {
            (&Method::GET, "/v1/models") => r#"{"object":"list","data":[{"id":"m","object":"model"}]}"#,
            (&Method::POST, "/v1/chat/completions") => r#"{"id":"chatcmpl-local","object":"chat.completion","model":"m","choices":[{"index":0,"message":{"role":"assistant","content":"measured"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":1,"total_tokens":3}}"#,
            _ => return Response::with_status(404.into()),
        };
        Response::ok().header("content-type", "application/json").with_body(Body::from(data))
    })?;
    println!("Gateway,Round,Workload,Models,WorkingSetMiB,PrivateMiB");
    for round in 1..=rounds {
        for reference in [round % 2 == 0, round % 2 != 0] {
            let result = measure(
                if reference { magpie } else { cli },
                reference,
                round,
                work,
                &base,
            )
            .await;
            if let Err(error) = result {
                fixture.stop();
                fixture.join()?;
                return Err(error);
            }
        }
    }
    fixture.stop();
    fixture.join()?;
    Ok(())
}

async fn measure(exe: &Path, reference: bool, round: u8, work: Workload, base: &str) -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = dir.path();
    let addr = TcpListener::bind("127.0.0.1:0").await?.local_addr()?;
    let mut cmd = if reference {
        let local = root.join("magpie.exe");
        fs::copy(exe, &local)?;
        fs::write(root.join(".portable"), "")?;
        if matches!(work, Workload::Chat) {
            fs::create_dir(root.join("data"))?;
            fs::write(
                root.join("data/providers.json"),
                nextjson::to_vec(
                    &json!({"providers":[{"id":"bench","name":"Benchmark","key":"local-test","chat":base,"models":["m"]}]}),
                ).map_err(anyhow::Error::msg)?,
            )?;
        }
        Command::new(local)
    } else {
        let mut cfg = nextjson::Map::new();
        cfg.insert("listen".into(), json!(addr.to_string()));
        if matches!(work, Workload::Chat) {
            cfg.insert(
                "providers".into(),
                json!([{"id":"bench","endpoints":{"chat":base},"models":["m"]}]),
            );
        }
        fs::write(
            root.join("settings.json"),
            nextjson::to_vec(&Value::Object(cfg)).map_err(anyhow::Error::msg)?,
        )?;
        Command::new(exe)
    };
    let startup_log = root.join("gateway-stderr.log");
    cmd.arg("serve")
        .kill_on_drop(true)
        .env_clear()
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(fs::File::create(&startup_log)?);
    for var in [
        "SystemRoot",
        "WINDIR",
        "SystemDrive",
        "ComSpec",
        "PROCESSOR_ARCHITECTURE",
    ] {
        if let Some(v) = env::var_os(var) {
            cmd.env(var, v);
        }
    }
    cmd.env(
        "PATH",
        Path::new(&env::var_os("WINDIR").context("missing WINDIR")?).join("System32"),
    );
    for var in [
        "HOME",
        "USERPROFILE",
        "TEMP",
        "TMP",
        "LOCALAPPDATA",
        "APPDATA",
        "CRABFORGE_HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
    ] {
        cmd.env(var, root);
    }
    cmd.env("MAGPIE_ADDR", addr.to_string())
        .env("MAGPIE_NO_STATS", "1")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "127.0.0.1,localhost");
    let mut process = cmd.spawn()?;
    let http = Client::with_config(ClientConfig {
        read_timeout: Some(Duration::from_secs(10)),
        connect_timeout: Some(Duration::from_secs(10)),
        max_redirects: 0,
        ..Default::default()
    });
    let catalog = format!("http://{addr}/v1/models");
    let start = Instant::now();
    let mut last_probe = String::from("waiting for gateway key");
    let (key, models) = loop {
        check_ready(&mut process, start, Duration::from_secs(90)).with_context(|| {
            format!(
                "{} at {addr}, {work:?} round {round}, last probe {last_probe}\n{}",
                exe.display(),
                fs::read_to_string(&startup_log).unwrap_or_default()
            )
        })?;
        let key = if reference {
            Some("magpie".to_owned())
        } else {
            fs::read(root.join("gateway.key")).ok().map(hex::encode)
        };
        if let Some(key) = key {
            match http.request(&catalog, Method::GET).bearer_auth(&key).send() {
                Ok(res) if res.status.is_success() => {
                    let v = read_json(res)?;
                    break (
                        key,
                        v.get("data")
                            .and_then(Value::as_array)
                            .context("expected models")?
                            .len(),
                    );
                }
                Ok(res) => last_probe = res.status.as_u16().to_string(),
                Err(error) => last_probe = format!("{error:#?}"),
            }
        }
        sleep(Duration::from_millis(100)).await;
    };
    for _ in 0..100 {
        let req = match work {
            Workload::Catalog => http.request(&catalog, Method::GET),
            Workload::Chat => http.request(&format!("http://{addr}/v1/chat/completions"), Method::POST)
                .header("content-type", "application/json")
                .body(r#"{"model":"bench/m","messages":[{"role":"user","content":"local measurement"}]}"#),
        };
        let res = req.bearer_auth(&key).send()?;
        ensure!(
            res.status.is_success(),
            "gateway returned HTTP {}",
            res.status.as_u16()
        );
        let v = read_json(res)?;
        if matches!(work, Workload::Chat) {
            ensure!(
                v.pointer("/choices/0/message/content")
                    .and_then(Value::as_str)
                    == Some("measured"),
                "unexpected benchmark response"
            );
        }
    }
    sleep(Duration::from_secs(2)).await;
    let (w, p) = sample(&[process.id().context("gateway exited")?]).await?;
    println!(
        "{},{round},{work:?},{models},{w:.2},{p:.2}",
        if reference { "Magpie" } else { "Crabforge" }
    );
    process.kill().await?;
    Ok(())
}

fn read_json(res: Response<Body>) -> Result<Value> {
    let bytes = res.body.collect_limited(4 * 1024 * 1024)?;
    nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)
}
