use anyhow::{Context, Result, bail, ensure};
use axum::{
    Json, Router,
    routing::{get, post},
};
use clap::ValueEnum;
use reqwest::Client;
use serde_json::{Value, json};
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
            check_ready(&mut app, start)?;
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

fn check_ready(child: &mut Child, start: Instant) -> Result<()> {
    if let Some(status) = child.try_wait()? {
        bail!("process exited before readiness: {status}");
    }
    ensure!(
        start.elapsed() < Duration::from_secs(30),
        "readiness check timed out"
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Workload {
    Catalog,
    Chat,
}

pub async fn gateway(cli: &Path, magpie: &Path, work: Workload, rounds: u8) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}/v1", listener.local_addr()?);
    let fixture=Router::new().route("/v1/chat/completions",post(||async{Json(json!({"id":"chatcmpl-local","object":"chat.completion","model":"m","choices":[{"index":0,"message":{"role":"assistant","content":"measured"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":1,"total_tokens":3}}))})).route("/v1/models",get(||async{Json(json!({"object":"list","data":[{"id":"m","object":"model"}]}))}));
    let fixture = tokio::spawn(async move { axum::serve(listener, fixture).await });
    println!("Gateway,Round,Workload,Models,WorkingSetMiB,PrivateMiB");
    for round in 1..=rounds {
        for reference in [round % 2 == 0, round % 2 != 0] {
            measure(
                if reference { magpie } else { cli },
                reference,
                round,
                work,
                &base,
            )
            .await?;
        }
    }
    fixture.abort();
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
            fs::write(root.join("data/providers.json"),json!({"providers":[{"id":"bench","name":"Benchmark","key":"local-test","chat":base,"models":["m"]}]}).to_string())?;
        }
        Command::new(local)
    } else {
        let mut cfg = json!({"listen":addr});
        if matches!(work, Workload::Chat) {
            cfg["providers"] = json!([{"id":"bench","endpoints":{"chat":base},"models":["m"]}]);
        }
        fs::write(root.join("settings.json"), cfg.to_string())?;
        Command::new(exe)
    };
    cmd.arg("serve")
        .kill_on_drop(true)
        .env_clear()
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
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
    let http = Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()?;
    let catalog = format!("http://{addr}/v1/models");
    let start = Instant::now();
    let (key, models) = loop {
        check_ready(&mut process, start)?;
        let key = if reference {
            Some("magpie".to_owned())
        } else {
            fs::read(root.join("gateway.key")).ok().map(hex::encode)
        };
        if let Some(key) = key
            && let Ok(res) = http.get(&catalog).bearer_auth(&key).send().await
            && res.status().is_success()
        {
            let v: Value = serde_json::from_slice(&res.bytes().await?)?;
            break (key, v["data"].as_array().context("expected models")?.len());
        }
        sleep(Duration::from_millis(100)).await;
    };
    for _ in 0..100 {
        let req=match work{Workload::Catalog=>http.get(&catalog),Workload::Chat=>http.post(format!("http://{addr}/v1/chat/completions")).header("content-type","application/json").body(r#"{"model":"bench/m","messages":[{"role":"user","content":"local measurement"}]}"#)};
        let data = req
            .bearer_auth(&key)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let v: Value = serde_json::from_slice(&data)?;
        if matches!(work, Workload::Chat) {
            ensure!(
                v["choices"][0]["message"]["content"] == "measured",
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
