use anyhow::Result;
use std::process::Command;

#[cfg(not(windows))]
pub fn installed_command(binary: &str, _package: &str, _runtime: &str) -> Result<Command> {
    Ok(Command::new(binary))
}

/// Resolve npm launchers without a command shell, preserving the client's argument boundaries.
#[cfg(windows)]
pub fn installed_command(binary: &str, package: &str, runtime: &str) -> Result<Command> {
    use anyhow::{Context, ensure};
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths).filter(|dir| dir.is_absolute()) {
        let exe = dir.join(format!("{binary}.exe"));
        if exe.is_file() {
            return Ok(Command::new(exe));
        }
        if package.is_empty() || !dir.join(format!("{binary}.cmd")).is_file() {
            continue;
        }
        let modules = if dir.ends_with(".bin") {
            dir.parent().unwrap().to_owned()
        } else {
            dir.join("node_modules")
        };
        let package_dir = modules.join(package);
        let manifest = package_dir.join("package.json");
        if !manifest.is_file() {
            continue;
        }
        let meta: nextjson::Value =
            nextjson::from_slice(&std::fs::read(manifest)?).map_err(anyhow::Error::msg)?;
        let bin = meta.get("bin").context("missing npm client entry point")?;
        let entry = bin
            .get(binary)
            .and_then(nextjson::Value::as_str)
            .or_else(|| bin.as_str())
            .context("missing npm client entry point")?;
        let script = package_dir.join(entry);
        ensure!(
            script.is_file(),
            "missing client entry point {}",
            script.display()
        );
        if script.extension().is_some_and(|e| e == "exe") {
            return Ok(Command::new(script));
        }
        let mut command = Command::new(runtime);
        command.arg(script);
        return Ok(command);
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("install {binary} on PATH"),
    )
    .into())
}
