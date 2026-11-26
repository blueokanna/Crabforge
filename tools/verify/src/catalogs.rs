use anyhow::{Context, Result, ensure};
use futures_util::{StreamExt, stream};
use nextjson::{Value, json};
use std::{collections::BTreeMap, fs, path::Path, time::Duration};
use tokio::{process::Command, time::timeout};

struct Row(&'static str, Option<usize>, String);

async fn inspect(cli: &Path, p: &Value) -> Result<Row> {
    if p.get("resource").is_some() {
        return Ok(Row("Resource", None, "Requires a resource URL".into()));
    }
    if p.get("id").and_then(Value::as_str) == Some("bedrock-mantle") {
        return Ok(Row("Credentials", None, "Requires an AWS account".into()));
    }
    let endpoints = p.get("endpoints").context("missing endpoints")?;
    if endpoints
        .as_object()
        .context("expected endpoint object")?
        .values()
        .filter_map(Value::as_str)
        .filter_map(|s| url::Url::parse(s).ok())
        .any(|url| matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")))
    {
        return Ok(Row("Local", None, "Requires a local server".into()));
    }
    let dir = tempfile::tempdir()?;
    fs::write(
        dir.path().join("settings.json"),
        nextjson::to_vec(
            &json!({"providers":[{"id":"survey","endpoints":endpoints,"models":["survey"]}]}),
        )
        .map_err(anyhow::Error::msg)?,
    )?;
    let mut cmd = Command::new(cli);
    cmd.args(["models", "survey"])
        .kill_on_drop(true)
        .env("CRABFORGE_HOME", dir.path());
    for k in [
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "AWS_SESSION_TOKEN",
    ] {
        cmd.env_remove(k);
    }
    let Ok(output) = timeout(Duration::from_secs(100), cmd.output()).await else {
        return Ok(Row("Timeout", None, "CLI exceeded 100 seconds".into()));
    };
    let output = output?;
    let detail = String::from_utf8(output.stderr)?;
    if !output.status.success() {
        return Ok(Row(
            "Error",
            None,
            detail.lines().collect::<Vec<_>>().join(" "),
        ));
    }
    let count = String::from_utf8(output.stdout)?
        .lines()
        .filter(|s| !s.is_empty())
        .count();
    Ok(Row(
        if detail.contains("Using built-in model IDs") {
            "Built-in"
        } else {
            "Live"
        },
        Some(count),
        String::new(),
    ))
}

pub async fn run(cli: &Path, parallel: u8) -> Result<()> {
    let output = Command::new(cli).arg("catalog").output().await?;
    ensure!(output.status.success(), "cannot read provider presets");
    let presets: Vec<Value> = nextjson::from_slice(&output.stdout).map_err(anyhow::Error::msg)?;
    let mut unique = BTreeMap::new();
    let mut ids = Vec::new();
    for p in &presets {
        let id = p
            .get("id")
            .and_then(Value::as_str)
            .context("missing preset ID")?;
        let key = if p.get("resource").is_some() {
            id.to_owned()
        } else {
            // Equivalent endpoints can arrive in a different JSON property order.
            let endpoints = p
                .get("endpoints")
                .and_then(Value::as_object)
                .context("missing endpoints")?;
            nextjson::to_string(&endpoints.iter().collect::<BTreeMap<_, _>>())
                .map_err(anyhow::Error::msg)?
        };
        unique.entry(key.clone()).or_insert(p);
        ids.push((id, key));
    }
    let mut jobs = stream::iter(unique)
        .map(|(k, p)| async move { Ok::<_, anyhow::Error>((k, inspect(cli, p).await?)) })
        .buffer_unordered(usize::from(parallel));
    let mut results = BTreeMap::new();
    while let Some(row) = jobs.next().await {
        let (k, v) = row?;
        results.insert(k, v);
    }
    println!("Preset,Result,Models,Detail");
    for (id, key) in ids {
        let Row(kind, count, detail) = &results[&key];
        println!(
            "\"{}\",{kind},{},\"{}\"",
            id.replace('"', "\"\""),
            count.map(|n| n.to_string()).unwrap_or_default(),
            detail.replace('"', "\"\"")
        );
    }
    Ok(())
}
