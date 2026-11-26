use anyhow::{Context, Result};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

pub(super) fn configure(
    command: &mut Command,
    dir: &Path,
    root: &str,
    model: &str,
    ids: Vec<String>,
) -> Result<()> {
    let home = std::path::absolute(dir.join("agents/crush"))?;
    let profile = home.join("config");
    // Crush's TUI reads the picks from its data file after the global config.
    // Isolate that file by selection so concurrent starts can choose different models.
    let profile_id = hex::encode(Sha256::digest(model.as_bytes()));
    let data = home.join("data").join(&profile_id);
    fs::create_dir_all(&profile)?;
    let lock = fs::File::options()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(home.join("crabforge.lock"))?;
    lock.lock()?;
    let models: Vec<_> = ids
        .into_iter()
        .map(|id| json!({"id":id,"name":id}))
        .collect();
    super::edit_json_object(&profile.join("crush.json"), "providers", |providers| {
        let provider = json!({
            "type":"completions", "name":"Crabforge", "base_url":format!("{root}/v1"),
            "api_key":"$CRABFORGE_GATEWAY_KEY", "models":models
        });
        providers.insert("crabforge".into(), provider);
        Ok(())
    })?;
    super::edit_json_object(&data.join("crush.json"), "models", |picks| {
        for size in ["large", "small"] {
            if picks
                .get(size)
                .is_none_or(|pick| pick["provider"] != "crabforge" || pick["model"] != model)
            {
                picks.insert(size.into(), json!({"provider":"crabforge","model":model}));
            }
        }
        Ok(())
    })?;
    command
        .env("CRUSH_GLOBAL_CONFIG", profile)
        .env("CRUSH_GLOBAL_DATA", data)
        .env("CRUSH_CACHE_DIR", home.join("cache"))
        .env("CRUSH_CLIENT_SERVER", "0");
    if std::env::var_os("CRUSH_DATA_DIR").is_none() {
        command.env(
            "CRUSH_DATA_DIR",
            Path::new(".crush/crabforge").join(profile_id),
        );
    }
    Ok(())
}

// Crush 0.102's embedded `run` can exit before printing its final text event.
// Its server protocol includes the full response in the run-complete event.
pub(super) struct RunServer {
    child: Child,
    _runtime: tempfile::TempDir,
}

impl RunServer {
    pub(super) fn start(client: &mut Command) -> Result<Self> {
        let mut temporary = tempfile::Builder::new();
        temporary.prefix("crabforge-crush-").rand_bytes(16);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            temporary.permissions(fs::Permissions::from_mode(0o700));
        }
        let runtime = temporary.tempdir()?;
        #[cfg(windows)]
        let (endpoint, host) = {
            let name = runtime.path().file_name().unwrap().to_string_lossy();
            (
                std::path::PathBuf::from(format!(r"\\.\pipe\{name}")),
                format!("npipe:////./pipe/{name}"),
            )
        };
        #[cfg(not(windows))]
        let (endpoint, host) = {
            let socket = runtime.path().join("s");
            let host = format!("unix://{}", socket.display());
            (socket, host)
        };
        client
            .env("CRUSH_CLIENT_SERVER", "1")
            .env("CRUSH_CACHE_DIR", runtime.path())
            .env("CRUSH_SERVER_IDLE_TIMEOUT", "0")
            .env("CRUSH_SERVER_DETACH_GRACE", "0");
        let mut server = Command::new(client.get_program());
        for (k, v) in client.get_envs() {
            if let Some(v) = v {
                server.env(k, v);
            } else {
                server.env_remove(k);
            }
        }
        let child = server
            .args(["server", "--host", &host])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .context("start Crush run server")?;
        let mut guard = Self {
            child,
            _runtime: runtime,
        };
        let started = Instant::now();
        loop {
            if let Some(status) = guard.child.try_wait()? {
                anyhow::bail!("Crush server exited during startup with {status}");
            }
            #[cfg(windows)]
            let listening = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&endpoint)
                .is_ok();
            #[cfg(not(windows))]
            let listening = std::os::unix::net::UnixStream::connect(&endpoint).is_ok();
            if listening {
                break;
            }
            anyhow::ensure!(
                started.elapsed() < Duration::from_secs(15),
                "Crush run server did not start"
            );
            thread::sleep(Duration::from_millis(20));
        }
        client.args(["--host", &host]);
        Ok(guard)
    }

    pub(super) fn finish(mut self) -> Result<()> {
        let start = Instant::now();
        while self.child.try_wait()?.is_none() && start.elapsed() < Duration::from_secs(2) {
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }
}

impl Drop for RunServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
