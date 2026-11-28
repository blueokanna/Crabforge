use anyhow::{Result, ensure};
use crabforge_core::{config, service::Service};
use crabforge_test_support::server::{Incoming, Json, Router, StatusCode, post};
use nextjson::{Value, json};
use std::{fs, process::Command};
use tokio::net::TcpListener;

fn group_cli(dir: &std::path::Path, args: &[&str]) -> Result<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"))
        .arg("--data-dir")
        .arg(dir)
        .args(args)
        .output()?;
    ensure!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(String::from_utf8(out.stdout)?)
}

#[tokio::test]
async fn provider_fallback_uses_the_selected_group_member_once_and_preserves_order() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move {
        let app = Router::new().route(
            "/embeddings",
            post(|Incoming { body: payload, .. }| async move {
                let body: Value = nextjson::from_slice(&payload).unwrap();
                let status = match body["model"].as_str().unwrap() {
                    "busy" => StatusCode::TOO_MANY_REQUESTS,
                    "down" => StatusCode::SERVICE_UNAVAILABLE,
                    _ => StatusCode::OK,
                };
                (status, Json(body))
            }),
        );
        crabforge_test_support::server::serve(listener, app).await
    });
    let cfg = json!({"listen":"127.0.0.1:0", "providers":[
        {"id":"primary","models":["busy"],"endpoints":{"chat":base}},
        {"id":"spare","models":["down","ok","unexpected"],"endpoints":{"chat":base},"fallback":["spare/unexpected"]}],
        "groups":{"overlap":["primary/busy","spare/down","group/pick"],"pick":["spare/ok","spare/down"]},"group_picks":{"pick":"spare/ok"}});
    fs::write(dir.path().join("settings.json"), cfg.to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    let chain = ["spare/down", "group/overlap"];
    let saved = run(&[
        "provider",
        "fallback",
        "primary",
        "spare/down",
        "group/overlap",
        "spare/down",
    ])?;
    assert_eq!(
        nextjson::from_str::<Vec<String>>(&saved).map_err(anyhow::Error::msg)?,
        chain
    );
    assert!(run(&["provider", "fallback", "primary", "group/missing"]).is_err());
    assert_eq!(run(&["provider", "fallback", "primary"])?, saved);
    run(&["group", "rename", "overlap", "backup"])?;
    assert_eq!(
        config::load(dir.path())?.providers[0].fallback,
        ["spare/down", "group/backup"]
    );
    let service = Service::start(dir.path()).await?;
    let client = crabforge_test_support::Client::default();
    let request = || {
        client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(config::token(dir.path()).unwrap())
            .body(r#"{"model":"primary/busy","input":"hello"}"#)
            .send()
    };
    let reply = request().await?.error_for_status()?.bytes().await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&reply).map_err(anyhow::Error::msg)?["model"],
        json!("ok")
    );
    {
        let attempts = service.history.lock().unwrap();
        assert_eq!(
            attempts.iter().map(|a| a.status).collect::<Vec<_>>(),
            [429, 503, 200]
        );
        assert_eq!(attempts[2].via, ["group/backup", "group/pick"]);
        assert!(attempts.iter().all(|a| a.model == "primary/busy"));
    }
    run(&["group", "pick", "pick", "spare/down"])?;
    assert_eq!(request().await?.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(service.history.lock().unwrap().len(), 5);
    service.stop().await;
    assert_eq!(
        run(&["provider", "fallback", "primary", "none"])?.trim(),
        "[]"
    );
    server.abort();
    Ok(())
}

#[tokio::test]
async fn restricted_providers_only_take_fallbacks_from_other_restricted_providers() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move {
        crabforge_test_support::server::serve(
            listener,
            Router::new().route(
                "/{id}/embeddings",
                post(|Incoming { param: id, .. }| async move {
                    let code = if matches!(id.as_str(), "local" | "first") {
                        StatusCode::SERVICE_UNAVAILABLE
                    } else {
                        StatusCode::OK
                    };
                    (code, id)
                }),
            ),
        )
        .await
    });
    let cfg = json!({"listen":"127.0.0.1:0", "providers":[
        {"id":"local","models":["m"],"endpoints":{"chat":format!("{base}/local")},"fallback":["group/nested","spare/m"]},
        {"id":"first","models":["m"],"endpoints":{"chat":format!("{base}/first")},"noStandIn":true,"fallback":["second/m"]},
        {"id":"second","models":["m"],"endpoints":{"chat":format!("{base}/second")}},
        {"id":"spare","models":["m"],"endpoints":{"chat":format!("{base}/spare")}}],
        "groups":{"marked":["first/m","second/m"],"nested":["group/marked"],"mixed":["local/m","group/marked"]}});
    fs::write(dir.path().join("settings.json"), cfg.to_string())?;
    let run = |value| group_cli(dir.path(), &["provider", "set", "second", value]);
    assert!(run("fallback-for-others=invalid").is_err());
    let key = config::token(dir.path())?;
    let http = crabforge_test_support::Client::default();
    for restricted in [true, false] {
        run(if restricted {
            "fallback-for-others=no"
        } else {
            "fallback-for-others=yes"
        })?;
        let service = Service::start(dir.path()).await?;
        for (model, expected) in [
            ("local/m", if restricted { "spare" } else { "second" }),
            ("first/m", "second"),
            ("group/nested", "second"),
            ("group/mixed", if restricted { "local" } else { "second" }),
        ] {
            let response = http
                .post(format!("http://{}/v1/embeddings", service.addr))
                .bearer_auth(&key)
                .body(format!(r#"{{"model":"{model}","input":"hello"}}"#))
                .send()
                .await?;
            assert_eq!(
                response.status().is_success(),
                expected != "local",
                "{model}"
            );
            assert_eq!(
                service.history.lock().unwrap().back().unwrap().provider,
                expected
            );
        }
        service.stop().await;
    }
    server.abort();
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn pattern_groups_follow_the_catalog_and_round_trip_without_pinning_matches() -> Result<()> {
    use std::{io::Write, process::Stdio};
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move {
        crabforge_test_support::server::serve(listener,Router::new()
        .route("/models",crabforge_test_support::server::get(|_|async {Json(json!({"data":[{"id":"new/free","free":true},{"id":"free-by-name","pricing":{"prompt":0}},{"id":"text","free":"true"}]}))}))
        .route("/embeddings",post(|Incoming {body: payload, ..}|async move { let body: Value = nextjson::from_slice(&payload).unwrap();Json(body)}))).await
    });
    let settings = dir.path().join("settings.json");
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"lab","models":["Alpha","n/One","flash","flash-free"],"free_models":["n/One"],"endpoints":{"chat":base}}]});
    fs::write(&settings, cfg.to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    let legacy_match = concat!("free:mag", "pie/LAB/*");
    let doc = json!({"format":"routing-group","version":1,"group":{"id":"remote","name":"Patterns","members":null,"match":[legacy_match,"re:(?!.*flash).*alpha"],"placed":["lab/n/One","lab/Alpha"],"routing":"manual","pick":"lab/n/One"}});
    let legacy = doc
        .to_string()
        .replacen("\"format\"", concat!("\"mag", "pie\""), 1);
    let mut child = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"))
        .arg("--data-dir")
        .arg(dir.path())
        .args(["group", "import", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child.stdin.take().unwrap().write_all(legacy.as_bytes())?;
    let out = child.wait_with_output()?;
    assert!(out.status.success());
    let list = run(&["group", "show", "patterns"])?;
    assert!(list.contains("1\tlab/n/One\n2\tlab/Alpha"), "{list}");
    let share: Value =
        nextjson::from_str(&run(&["group", "share", "patterns"])?).map_err(anyhow::Error::msg)?;
    assert_eq!(share["format"], json!("routing-group"));
    assert_eq!(share["group"]["members"], json!([]));
    assert_eq!(share["group"]["placed"], doc["group"]["placed"]);
    assert_eq!(share["group"]["match"][0], json!("free:LAB/*"));
    assert!(list.contains("match\tfree:LAB/*\t1"));
    run(&["group", "copy", "patterns", "other"])?;
    run(&["group", "rename", "other", "mirror"])?;
    run(&["group", "place", "mirror", "lab/Alpha"])?;
    assert!(run(&["group", "show", "mirror"])?.contains("1\tlab/Alpha\n2\tlab/n/One"));
    run(&["group", "place", "mirror"])?;
    let shared_file = dir.path().join("shared.json");
    fs::write(&shared_file, share.to_string())?;
    let receiver = tempfile::tempdir()?;
    fs::write(receiver.path().join("settings.json"), json!({"providers":[{"id":"lab","models":["fresh","flash"],"free_models":["fresh"],"endpoints":{"chat":base}}]}).to_string())?;
    group_cli(
        receiver.path(),
        &["group", "import", shared_file.to_str().unwrap()],
    )?;
    let imported = config::load(receiver.path())?;
    assert!(imported.groups["patterns"].is_empty());
    assert_eq!(
        imported.resolved_groups().members["patterns"],
        ["lab/fresh"]
    );
    run(&["group", "set", "future", "re:(?-i)lab/Z.*", "--manual"])?;
    run(&["group", "set", "parent", "group/future", "group/patterns"])?;
    run(&[
        "group",
        "set",
        "mixed",
        "lab/Alpha",
        "lab/flash",
        "free:lab/*",
    ])?;
    run(&[
        "group",
        "place",
        "mixed",
        "lab/Alpha",
        "lab/n/One",
        "lab/flash",
    ])?;
    run(&[
        "group",
        "set",
        "mixed",
        "lab/flash",
        "lab/Alpha",
        "free:lab/*",
    ])?;
    let mixed: Value =
        nextjson::from_str(&run(&["group", "share", "mixed"])?).map_err(anyhow::Error::msg)?;
    assert_eq!(mixed["group"]["members"], json!(["lab/flash", "lab/Alpha"]));
    assert_eq!(
        mixed["group"]["placed"],
        json!(["lab/flash", "lab/n/One", "lab/Alpha"])
    );
    let service = Service::start(dir.path()).await?;
    let token = config::token(dir.path())?;
    let http = crabforge_test_support::Client::default();
    let request = |addr, model: &str| {
        http.post(format!("http://{addr}/v1/embeddings"))
            .bearer_auth(&token)
            .body(json!({"model":model,"input":"hello"}).to_string())
            .send()
    };
    let empty = request(service.addr, "group/future").await?;
    assert_eq!(empty.status(), StatusCode::BAD_REQUEST);
    let mixed = request(service.addr, "group/mixed")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&mixed).map_err(anyhow::Error::msg)?["model"],
        json!("flash")
    );
    let reply = request(service.addr, "group/parent")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&reply).map_err(anyhow::Error::msg)?["model"],
        json!("n/One")
    );
    run(&["group", "pick", "patterns", "lab/Alpha"])?;
    let reply = request(service.addr, "group/parent")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&reply).map_err(anyhow::Error::msg)?["model"],
        json!("Alpha")
    );
    service.stop().await;
    run(&["group", "remove", "mixed"])?;
    run(&["models", "lab", "--sync"])?;
    let saved = config::load(dir.path())?;
    assert_eq!(saved.resolved_groups().members["mirror"], ["lab/new/free"]);
    assert!(saved.groups["mirror"].is_empty());
    let service = Service::start(dir.path()).await?;
    let reply = request(service.addr, "group/parent")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&reply).map_err(anyhow::Error::msg)?["model"],
        json!("new/free")
    );
    let body = http
        .get(format!("http://{}/v1/models", service.addr))
        .bearer_auth(token)
        .send()
        .await?
        .bytes()
        .await?;
    let catalog: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
    assert!(
        catalog["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["id"] == json!("lab/new/free") && e["free"] == json!(true))
    );
    service.stop().await;
    let before = fs::read(&settings)?;
    for invalid in ["re:[", "free:free:lab/*", "free:"] {
        assert!(run(&["group", "set", "mirror", invalid]).is_err());
        assert_eq!(fs::read(&settings)?, before);
    }
    run(&["group", "remove", "mirror"])?;
    server.abort();
    Ok(())
}

#[tokio::test]
async fn group_display_order_survives_rename_and_reaches_all_model_lists() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0", "providers":[
        {"id":"a","models":["m"],"endpoints":{"chat":"http://127.0.0.1:1/v1"}},
        {"id":"b","models":["m"],"endpoints":{"chat":"http://127.0.0.1:2/v1"}}],
        "groups":{"first":["a/m"],"last":["b/m"]}});
    fs::write(dir.path().join("settings.json"), cfg.to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    run(&["group", "order", "group/last", "auto/m"])?;
    let saved = config::snapshot(dir.path())?;
    for invalid in ["group/last", "missing"] {
        assert!(run(&["group", "order", "last", invalid]).is_err());
        assert_eq!(config::snapshot(dir.path())?, saved);
    }
    run(&["group", "rename", "last", "renamed"])?;
    run(&["group", "set", "added", "a/m"])?;
    let expected = ["group/renamed", "auto/m", "group/first", "group/added"];
    let list = run(&["group", "list"])?;
    assert_eq!(
        list.lines()
            .map(|s| s.split('\t').next().unwrap())
            .collect::<Vec<_>>(),
        expected
    );
    let service = Service::start(dir.path()).await?;
    let http = crabforge_test_support::Client::default();
    for (path, root, name) in [
        ("v1/models", "data", "id"),
        ("v1beta/models", "models", "name"),
        ("api/tags", "models", "name"),
    ] {
        let bytes = http
            .get(format!("http://{}/{path}", service.addr))
            .bearer_auth(config::token(dir.path())?)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let list: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let ids: Vec<_> = list[root]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m[name].as_str().unwrap().trim_start_matches("models/"))
            .collect();
        assert_eq!(ids[..expected.len()], expected);
        assert_eq!(
            ids.len(),
            ids.iter().collect::<std::collections::BTreeSet<_>>().len()
        );
        assert!(!ids.contains(&"group/auto-m") && !ids.contains(&"group/last"));
    }
    service.stop().await;
    run(&["group", "remove", "renamed"])?;
    assert!(
        !config::load(dir.path())?
            .group_display_order
            .contains(&"renamed".into())
    );
    Ok(())
}

#[test]
fn shares_remap_members_and_replace_only_the_requested_group() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let settings = dir.path().join("settings.json");
    let cfg = json!({"providers":[{"id":"local","models":["m","n"],"key_env":"PERSONAL_KEY","endpoints":{"chat":"http://127.0.0.1:1/v1"}}],
        "groups":{"work":["local/n"],"parent":["group/work"]},"group_display_order":["parent","work"],"disabled_groups":["work"]});
    fs::write(&settings, cfg.to_string())?;
    let share = dir.path().join("shared.txt");
    let mut doc = json!({"format":"routing-group","version":1,"group":{
        "id":"work","name":"Parent","routing":"weight","members":["sender/m#secret","sender/m","old/m","local/n","lost/x"],
        "weights":{"old/m":5,"sender/m":2,"lost/x":4},"off":["sender/m"],"pick":"local/n","unlists":true,
        "accounts":["local/secret"],"keys":["secret"],"note":"priority   models"}});
    fs::write(&share, format!("Shared in chat\n```json\n{doc}\n```"))?;
    let run = |replace: bool| {
        let mut args = vec!["group", "import", share.to_str().unwrap()];
        if replace {
            args.push("--replace");
        }
        group_cli(dir.path(), &args)
    };
    let report: Value = nextjson::from_str(&run(true)?).map_err(anyhow::Error::msg)?;
    assert_eq!(report["id"], json!("work"));
    assert_eq!(report["name"], json!("Parent 2"));
    assert_eq!(report["replaced"], json!(true));
    assert_eq!(report["missing"], json!(["lost/x"]));
    assert_eq!(
        report["instead"],
        json!(["sender/m → local/m", "old/m → local/m"])
    );
    let saved: Value = nextjson::from_slice(&fs::read(&settings)?).map_err(anyhow::Error::msg)?;
    assert_eq!(saved["groups"]["parent"], cfg["groups"]["parent"]);
    assert_eq!(saved["group_display_order"], cfg["group_display_order"]);
    assert_eq!(saved["disabled_groups"], cfg["disabled_groups"]);
    let exported = group_cli(dir.path(), &["group", "share", "group/work"])?;
    for private in [
        "secret",
        "PERSONAL_KEY",
        "127.0.0.1",
        "accounts",
        "disabled",
    ] {
        assert!(!exported.contains(private));
    }
    let clean: Value = nextjson::from_str(&exported).map_err(anyhow::Error::msg)?;
    let expected = json!({"id":"work","name":"Parent 2","routing":"weight","members":["local/m","local/n"],"weights":{"local/m":5},"off":["local/m"],"pick":"local/n","note":"priority models","unlists":true});
    assert!(
        crabforge_core::json::equivalent(&clean["group"], &expected),
        "{clean}"
    );
    fs::write(&share, exported)?;
    let added: Value = nextjson::from_str(&run(false)?).map_err(anyhow::Error::msg)?;
    assert_eq!(added["id"], json!("parent-2-2"));
    let before = fs::read(&settings)?;
    for (field, value) in [
        ("members", json!(["nothing/here"])),
        ("members", json!(["group/parent"])),
        ("rules", json!([{"use":"local/m"}])),
    ] {
        let previous = doc
            .get_mut("group")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(field.into(), value);
        fs::write(&share, doc.to_string())?;
        assert!(run(true).is_err());
        assert_eq!(fs::read(&settings)?, before);
        let group = doc.get_mut("group").unwrap().as_object_mut().unwrap();
        if let Some(previous) = previous {
            group.insert(field.into(), previous);
        } else {
            group.remove(field);
        }
    }
    Ok(())
}

#[test]
fn copying_groups_preserves_independent_strategies_labels_and_discovered_members() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let cfg = json!({"providers":[
        {"id":"a","models":["m"],"endpoints":{"chat":"http://127.0.0.1:1/v1"}},
        {"id":"b","models":["m"],"endpoints":{"chat":"http://127.0.0.1:2/v1"}}],
        "groups":{"fast":["a/m","b/m"]}, "group_weights":{"fast":{"b/m":3}}, "group_last_picks":{"fast":"b/m"},"former_groups":{"previous":"fast"}});
    fs::write(dir.path().join("settings.json"), cfg.to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    assert_eq!(
        run(&["group", "copy", "group/fast"])?.trim(),
        "fast copy (group/fast-copy)"
    );
    assert_eq!(
        run(&["group", "duplicate", "fast", "Fast", "copy"])?.trim(),
        "Fast copy 2 (group/fast-copy-2)"
    );
    assert!(
        run(&["group", "copy", "Fast copy 2", "GPT", "6..1", "Sol"])?.contains("group/gpt-6.1-sol")
    );
    assert!(run(&["group", "copy", "fast", "Previous"])?.contains("group/previous-2"));
    let cfg = config::load(dir.path())?;
    assert_eq!(cfg.group_weights["gpt-6.1-sol"], cfg.group_weights["fast"]);
    assert_eq!(cfg.group_last_picks["fast-copy"], "b/m");
    assert_eq!(
        cfg.group_display_order
            .windows(2)
            .find(|w| w[0] == "fast")
            .unwrap()[1],
        "previous-2"
    );
    run(&["group", "set", "fast-copy", "a/m", "b/m", "--manual"])?;
    run(&["group", "copy", "fast-copy", "manual"])?;
    run(&["group", "pick", "manual", "a/m"])?;
    assert_eq!(config::load(dir.path())?.group_picks["fast-copy"], "b/m");
    run(&["group", "set", "manual", "a/m", "b/m", "--rotate"])?;
    run(&["group", "copy", "manual", "rotated"])?;
    assert!(
        config::load(dir.path())?
            .rotating_groups
            .contains("rotated")
    );
    let text = run(&["group", "share", "rotated"])?;
    let exported: Value = nextjson::from_str(&text).map_err(anyhow::Error::msg)?;
    assert_eq!(exported["group"]["routing"], json!("rotate"));
    assert_eq!(exported["group"]["weights"], json!({"b/m":3}));
    let file = dir.path().join("share.json");
    fs::write(&file, text)?;
    let imported: Value = nextjson::from_str(&run(&["group", "import", file.to_str().unwrap()])?)
        .map_err(anyhow::Error::msg)?;
    let id = imported["id"].as_str().unwrap();
    run(&["group", "rename", id, "retained"])?;
    assert_eq!(
        config::load(dir.path())?.group_last_weights["retained"]["b/m"],
        3
    );
    run(&["group", "set", "retained", "a/m", "b/m", "--weighted"])?;
    assert_eq!(
        config::load(dir.path())?.group_weights["retained"]["b/m"],
        3
    );
    run(&["group", "remove", "retained"])?;
    run(&["group", "copy", "auto/m", "我的组"])?;
    run(&["group", "auto", "off"])?;
    let cfg = config::load(dir.path())?;
    assert_eq!(cfg.group_labels["group"], "我的组");
    assert_eq!(cfg.groups["group"], ["a/m", "b/m"]);
    let long = "备".repeat(200);
    run(&[
        "group",
        "set",
        "annotated",
        "a/m",
        "--note",
        &format!(" \t项目\n  {long} "),
    ])?;
    run(&["group", "set", "annotated", "b/m", "--rotate"])?;
    run(&["group", "rename", "annotated", "production"])?;
    run(&["group", "copy", "production", "test"])?;
    let cfg = config::load(dir.path())?;
    assert_eq!(
        cfg.group_notes["test"],
        format!("项目 {}", "备".repeat(197))
    );
    assert!(!cfg.group_notes.contains_key("annotated"));
    run(&["group", "set", "test", "b/m", "--note="])?;
    run(&["group", "remove", "production"])?;
    assert!(config::load(dir.path())?.group_notes.is_empty());
    let before = config::snapshot(dir.path())?;
    assert!(run(&["group", "copy", "missing"]).is_err());
    assert_eq!(before, config::snapshot(dir.path())?);
    Ok(())
}

#[tokio::test]
async fn listing_only_the_group_keeps_its_direct_members_callable() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move {
        crabforge_test_support::server::serve(
            listener,
            Router::new().route(
                "/embeddings",
                post(|Incoming { body: payload, .. }| async move {
                    let body: Value = nextjson::from_slice(&payload).unwrap();
                    Json(body)
                }),
            ),
        )
        .await
    });
    fs::write(dir.path().join("settings.json"), json!({"listen":"127.0.0.1:0","providers":[{"id":"a","models":["one","shared","b/only"],"endpoints":{"chat":base}},{"id":"b","models":["only","shared"],"endpoints":{"chat":base}}],"groups":{"child":["b/only"]}}).to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    run(&[
        "group",
        "set",
        "first",
        "a/one",
        "a/shared",
        "a/b/only",
        "group/child",
        "--listed=no",
    ])?;
    run(&["group", "rename", "first", "main"])?;
    run(&["group", "copy", "main", "copy"])?;
    run(&["group", "set", "main", "a/one", "--listed=yes"])?;
    assert!(run(&["group", "show", "copy"])?.contains("listed members\tno"));
    let cfg = config::load(dir.path())?;
    assert_eq!(cfg.group_unlists.iter().collect::<Vec<_>>(), ["copy"]);
    let http = crabforge_test_support::Client::default();
    let token = config::token(dir.path())?;
    let service = Service::start(dir.path()).await?;
    for (path, root, key) in [
        ("v1/models", "data", "id"),
        ("v1beta/models", "models", "name"),
        ("api/tags", "models", "name"),
    ] {
        let bytes = http
            .get(format!("http://{}/{path}", service.addr))
            .bearer_auth(&token)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let catalog: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
        let ids: Vec<_> = catalog[root]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row[key].as_str().unwrap().trim_start_matches("models/"))
            .collect();
        for id in ["a/one", "one", "a/shared", "a/b/only"] {
            assert!(!ids.contains(&id), "{path}: {id}");
        }
        for id in [
            "group/main",
            "group/copy",
            "group/child",
            "b/only",
            "b/shared",
            "shared",
            "only",
        ] {
            assert!(ids.contains(&id), "{path}: {id}");
        }
    }
    for id in ["one", "a/one", "group/copy"] {
        let bytes = http
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(&token)
            .body(json!({"model":id,"input":"hello"}).to_string())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        assert_eq!(
            nextjson::from_slice::<Value>(&bytes).map_err(anyhow::Error::msg)?["model"],
            json!("one")
        );
    }
    service.stop().await;
    run(&["group", "remove", "copy"])?;
    let cfg = config::load(dir.path())?;
    assert!(cfg.unlisted_model_ids(&cfg.resolved_groups()).is_empty());
    server.abort();
    Ok(())
}

#[tokio::test]
async fn disabled_groups_release_their_models_and_preserve_saved_parent_routes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let app = Router::new().route(
        "/embeddings",
        post(|Incoming { body: payload, .. }| async move {
            let body: Value = nextjson::from_slice(&payload).unwrap();
            Json(body)
        }),
    );
    let server =
        tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
    fs::write(dir.path().join("settings.json"), json!({"listen":"127.0.0.1:0","providers":[{"id":"a","models":["first","second"],"endpoints":{"chat":base}},{"id":"b","models":["first"],"endpoints":{"chat":base}}]}).to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    assert!(run(&["group", "switch", "auto/first", "off"]).is_err());
    run(&["group", "set", "auto/first", "a/first", "--listed=no"])?;
    run(&[
        "group",
        "set",
        "manual",
        "auto/first",
        "a/second",
        "--pick",
        "auto/first",
    ])?;
    run(&[
        "group",
        "set",
        "parent",
        "group/manual",
        "a/second",
        "--weight",
        "group/manual=3",
    ])?;
    run(&["group", "switch", "auto/first", "off"])?;
    run(&["group", "rename", "auto/first", "inner"])?;
    run(&["group", "copy", "inner", "active"])?;
    assert!(!run(&["group", "show", "active"])?.contains("state\toff"));
    run(&["group", "remove", "active"])?;
    run(&[
        "group",
        "set",
        "inner",
        "a/first",
        "--note",
        "edited while off",
    ])?;
    assert!(run(&["group", "show", "inner"])?.contains("state\toff"));
    let key = config::token(dir.path())?;
    let http = crabforge_test_support::Client::default();
    for enabled in [false, true] {
        if enabled {
            run(&["group", "switch", "group/auto-first", "on"])?;
        }
        let service = Service::start(dir.path()).await?;
        let url = format!("http://{}", service.addr);
        let call = |id: &str| {
            http.post(format!("{url}/v1/embeddings"))
                .bearer_auth(&key)
                .body(json!({"model":id,"input":"ok"}).to_string())
                .send()
        };
        for (path, rows, name) in [
            ("v1/models", "data", "id"),
            ("v1beta/models", "models", "name"),
            ("api/tags", "models", "name"),
        ] {
            let body = http
                .get(format!("{url}/{path}"))
                .bearer_auth(&key)
                .send()
                .await?
                .error_for_status()?
                .bytes()
                .await?;
            let list: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
            let listed: Vec<_> = list[rows]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row[name].as_str().unwrap().trim_start_matches("models/"))
                .collect();
            assert_eq!(listed.contains(&"group/inner"), enabled, "{path}");
            assert_ne!(listed.contains(&"a/first"), enabled, "{path}");
        }
        for id in ["auto/first", "group/auto-first", "group/inner"] {
            let response = call(id).await?;
            if enabled {
                response.error_for_status()?;
            } else {
                assert_eq!(response.status(), StatusCode::NOT_FOUND);
                assert!(response.text().await?.contains("switched off"));
            }
        }
        let response = call("group/parent")
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        assert_eq!(
            nextjson::from_slice::<Value>(&response).map_err(anyhow::Error::msg)?["model"],
            json!(if enabled { "first" } else { "second" })
        );
        assert_eq!(call("group/manual").await?.status().is_success(), enabled);
        service.stop().await;
    }
    server.abort();
    Ok(())
}

#[tokio::test]
async fn switched_off_members_keep_their_place_and_weight_and_manual_pick_overrides_them()
-> Result<()> {
    let dir = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", listener.local_addr()?);
    let server = tokio::spawn(async move {
        crabforge_test_support::server::serve(
            listener,
            Router::new().route(
                "/embeddings",
                post(|Incoming { body: payload, .. }| async move {
                    let body: Value = nextjson::from_slice(&payload).unwrap();
                    Json(body)
                }),
            ),
        )
        .await
    });
    fs::write(dir.path().join("settings.json"), json!({"listen":"127.0.0.1:0","providers":[{"id":"local","models":["a","b"],"endpoints":{"chat":base}}]}).to_string())?;
    let run = |args: &[&str]| group_cli(dir.path(), args);
    run(&["group", "set", "child", "local/a"])?;
    run(&[
        "group",
        "set",
        "parent",
        "group/child",
        "local/b",
        "--weight",
        "group/child=3",
    ])?;
    run(&["group", "member", "parent", "group/child", "off"])?;
    let saved = config::snapshot(dir.path())?;
    assert!(run(&["group", "member", "parent", "local/b", "off"]).is_err());
    assert_eq!(config::snapshot(dir.path())?, saved);
    run(&["group", "rename", "child", "new"])?;
    run(&["group", "copy", "parent", "copy"])?;
    assert!(run(&["group", "list"])?.contains("group/new=3 local/b=1\toff=group/new"));
    assert_eq!(run(&["groups"])?, run(&["group", "list"])?);
    assert!(run(&["group", "show", "parent"])?.contains("1\tgroup/new\tweight=3\toff"));
    assert!(run(&["group", "group/child"])?.contains("used by\tgroup/parent"));
    let http = crabforge_test_support::Client::default();
    let ask = |group: &'static str, count| {
        let http = &http;
        let path = dir.path();
        async move {
            let service = Service::start(path).await?;
            let mut ids = Vec::new();
            for _ in 0..count {
                let bytes = http
                    .post(format!("http://{}/v1/embeddings", service.addr))
                    .bearer_auth(config::token(path)?)
                    .body(json!({"model":group,"input":"ok"}).to_string())
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;
                ids.push(
                    nextjson::from_slice::<Value>(&bytes).map_err(anyhow::Error::msg)?["model"]
                        .as_str()
                        .unwrap()
                        .to_owned(),
                );
            }
            service.stop().await;
            Ok::<_, anyhow::Error>(ids)
        }
    };
    assert_eq!(ask("group/copy", 4).await?, ["b", "b", "b", "b"]);
    run(&["group", "member", "parent", "group/new", "on"])?;
    let models = ask("group/parent", 4).await?;
    assert_eq!(models.iter().filter(|m| m.as_str() == "a").count(), 3);
    run(&[
        "group",
        "set",
        "copy",
        "group/new",
        "local/b",
        "--pick",
        "group/new",
    ])?;
    run(&["group", "member", "copy", "local/b", "off"])?;
    assert_eq!(ask("group/copy", 1).await?, ["a"]);
    run(&["group", "set", "copy", "local/a"])?;
    assert!(!config::load(dir.path())?.group_off.contains_key("copy"));
    server.abort();
    Ok(())
}

#[tokio::test]
async fn manual_group_switches_live_and_only_its_parent_can_fall_back() -> Result<()> {
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", upstream.local_addr()?);
    let app = Router::new().route(
        "/embeddings",
        post(|Incoming { body: payload, .. }| async move {
            let body: Value = nextjson::from_slice(&payload).unwrap();
            if body["model"] == json!("busy") {
                (StatusCode::TOO_MANY_REQUESTS, Json(body))
            } else {
                (StatusCode::OK, Json(body))
            }
        }),
    );
    let task =
        tokio::spawn(async move { crabforge_test_support::server::serve(upstream, app).await });
    let root = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"p","models":["busy","ready","outer"],"endpoints":{"chat":base}},
        {"id":"web","web_site":"deepseek","models":["deepseek-web"]}]});
    fs::write(root.path().join("settings.json"), cfg.to_string())?;
    let run = |args: &[&str]| group_cli(root.path(), args);
    run(&[
        "group",
        "set",
        "inner",
        "p/busy",
        "p/ready",
        "web/deepseek-web",
        "--pick",
        "p/busy",
    ])?;
    run(&[
        "group",
        "set",
        "outer",
        "p/ready",
        "group/inner",
        "--pick",
        "group/inner",
    ])?;
    run(&["group", "set", "fallback", "group/outer", "p/outer"])?;
    run(&["group", "set", "web-only", "web/deepseek-web", "p/ready"])?;
    run(&["group", "member", "web-only", "p/ready", "off"])?;
    run(&["group", "rename", "inner", "stable"])?;
    assert!(run(&["group", "list"])?.contains("group/outer\tmanual\tgroup/stable"));
    let before = config::snapshot(root.path())?;
    assert!(run(&["group", "pick", "stable", "p/outer"]).is_err());
    assert!(
        run(&[
            "group", "set", "stable", "p/ready", "--pick", "p/ready", "--rotate"
        ])
        .is_err()
    );
    assert_eq!(config::snapshot(root.path())?, before);
    assert!(
        run(&[
            "group",
            "set",
            "stable",
            "p/ready",
            "--pick",
            "p/ready",
            "--weight",
            "p/ready=2"
        ])
        .is_err()
    );
    let service = Service::start(root.path()).await?;
    let client = crabforge_test_support::Client::default();
    let key = config::token(root.path())?;
    let post = |path: &str, model: &str| {
        client
            .post(format!("http://{}{path}", service.addr))
            .bearer_auth(&key)
            .body(json!({"model":model,"input":"hi"}).to_string())
            .send()
    };
    let tags = client
        .get(format!("http://{}/api/tags", service.addr))
        .bearer_auth(&key)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let tags: Value = nextjson::from_slice(&tags).map_err(anyhow::Error::msg)?;
    assert_eq!(
        tags["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["name"] == json!("group/web-only"))
            .unwrap()["capabilities"],
        json!(["completion"])
    );
    let show = post("/api/show", "group/web-only")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&show).map_err(anyhow::Error::msg)?["capabilities"],
        json!(["completion"])
    );
    assert_eq!(
        post("/v1/embeddings", "group/outer").await?.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(service.history.lock().unwrap().len(), 1);
    let response = post("/v1/embeddings", "group/fallback")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&response).map_err(anyhow::Error::msg)?["model"],
        json!("outer")
    );
    run(&["group", "pick", "group/inner", "p/ready"])?;
    let response = post("/v1/embeddings", "group/outer")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&response).map_err(anyhow::Error::msg)?["model"],
        json!("ready")
    );
    for (pick, caps) in [
        ("web/deepseek-web", json!(["completion"])),
        ("p/ready", json!(["completion", "tools"])),
    ] {
        run(&["group", "pick", "stable", pick])?;
        let show = post("/api/show", "group/outer")
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        assert_eq!(
            nextjson::from_slice::<Value>(&show).map_err(anyhow::Error::msg)?["capabilities"],
            caps
        );
        if pick.starts_with("web/") {
            let attempts = service.history.lock().unwrap().len();
            assert_eq!(
                post("/v1/embeddings", "group/outer").await?.status(),
                StatusCode::BAD_REQUEST
            );
            assert_eq!(service.history.lock().unwrap().len(), attempts);
        }
    }
    service.stop().await;
    let mut cfg = config::load(root.path())?;
    cfg.group_picks.insert("stable".into(), "removed".into());
    fs::write(root.path().join("settings.json"), cfg.to_json()?)?;
    assert!(run(&["group", "list"])?.contains("group/stable\tmanual\tp/busy"));
    let restored = Service::start(root.path()).await?;
    let response = client
        .post(format!("http://{}/v1/embeddings", restored.addr))
        .bearer_auth(&key)
        .body(json!({"model":"group/stable","input":"hi"}).to_string())
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    restored.stop().await;
    run(&["group", "pick", "stable", "p/ready"])?;
    run(&[
        "group",
        "set",
        "outer",
        "p/ready",
        "group/stable",
        "--rotate",
    ])?;
    run(&["group", "rename", "stable", "renamed"])?;
    run(&[
        "group",
        "set",
        "outer",
        "p/ready",
        "group/renamed",
        "--manual",
    ])?;
    assert_eq!(
        config::load(root.path())?.group_picks["outer"],
        "group/renamed"
    );
    run(&["group", "set", "renamed", "p/busy", "p/ready", "--rotate"])?;
    assert!(
        !config::load(root.path())?
            .group_picks
            .contains_key("renamed")
    );
    run(&["group", "set", "renamed", "p/busy", "p/ready", "--manual"])?;
    assert_eq!(config::load(root.path())?.group_picks["renamed"], "p/ready");
    run(&["group", "set", "renamed", "p/busy", "--manual"])?;
    assert_eq!(config::load(root.path())?.group_picks["renamed"], "p/busy");
    task.abort();
    Ok(())
}

#[tokio::test]
async fn a_pending_parent_request_keeps_its_original_manual_selection() -> Result<()> {
    let entered = std::sync::Arc::new(tokio::sync::Notify::new());
    let release = std::sync::Arc::new(tokio::sync::Notify::new());
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", upstream.local_addr()?);
    let app = Router::new().route(
        "/embeddings",
        post({
            let entered = entered.clone();
            let release = release.clone();
            move |Incoming { body: payload, .. }| {
                let body: Value = nextjson::from_slice(&payload).unwrap();
                let entered = entered.clone();
                let release = release.clone();
                async move {
                    if body["model"] == json!("wait") {
                        entered.notify_one();
                        release.notified().await;
                        (StatusCode::TOO_MANY_REQUESTS, Json(body))
                    } else {
                        (StatusCode::OK, Json(body))
                    }
                }
            }
        }),
    );
    let task =
        tokio::spawn(async move { crabforge_test_support::server::serve(upstream, app).await });
    let root = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0", "providers":[{"id":"p","models":["wait","a","b"],"endpoints":{"chat":base}}],
        "groups":{"parent":["p/wait","group/manual"],"manual":["p/a","p/b"]},"group_picks":{"manual":"p/a"}});
    config::save(
        root.path(),
        &config::snapshot(root.path())?,
        &cfg.to_string(),
    )?;
    let service = Service::start(root.path()).await?;
    let client = crabforge_test_support::Client::with_timeout(std::time::Duration::from_secs(10));
    let key = config::token(root.path())?;
    let post = |model: &str| {
        client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(&key)
            .body(json!({"model":model,"input":"hi"}).to_string())
            .send()
    };
    let pending = tokio::spawn(post("group/parent"));
    tokio::time::timeout(std::time::Duration::from_secs(5), entered.notified()).await?;
    group_cli(root.path(), &["group", "pick", "manual", "p/b"])?;
    let newer = post("group/manual")
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&newer).map_err(anyhow::Error::msg)?["model"],
        json!("b")
    );
    release.notify_one();
    let original = pending.await??.error_for_status()?.bytes().await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&original).map_err(anyhow::Error::msg)?["model"],
        json!("a")
    );
    service.stop().await;
    task.abort();
    Ok(())
}

#[tokio::test]
async fn automatic_groups_follow_catalog_changes_and_keep_user_edits() -> Result<()> {
    let upstream = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}", upstream.local_addr()?);
    let app = Router::new()
        .route(
            "/busy/embeddings",
            post(|_| async { StatusCode::TOO_MANY_REQUESTS }),
        )
        .route(
            "/ready/embeddings",
            post(|Incoming { body: payload, .. }| async move {
                let body: Value = nextjson::from_slice(&payload).unwrap();
                Json(body)
            }),
        );
    let task =
        tokio::spawn(async move { crabforge_test_support::server::serve(upstream, app).await });
    let root = tempfile::tempdir()?;
    let run = |args: &[&str]| group_cli(root.path(), args);
    let cfg = json!({"listen":"127.0.0.1:0","providers":[
        {"id":"a","models":["vendor/claude-opus-5.5@1m","Claude-Opus-5.5","solo@1m","solo"],"endpoints":{"chat":format!("{base}/busy")}},
        {"id":"b","models":["claude-opus-5-5@20260801"],"endpoints":{"chat":format!("{base}/ready")}},
        {"id":"c","models":["local-name"],"endpoints":{"chat":format!("{base}/ready")}}],
        "model_same_as":{"c/local-name":"claude-opus-5.5"}});
    fs::write(root.path().join("settings.json"), cfg.to_string())?;
    let name = "auto-claude-opus-5-5";
    let list = run(&["group", "list"])?;
    assert!(list.contains("a/Claude-Opus-5.5 b/claude-opus-5-5@20260801 c/local-name"));
    assert!(!list.contains("solo"));
    run(&[
        "group",
        "set",
        "parent",
        "auto/CLAUDE-OPUS-5.5",
        "--weight",
        "auto/CLAUDE-OPUS-5.5=3",
    ])?;
    let service = Service::start(root.path()).await?;
    let client = crabforge_test_support::Client::default();
    let key = config::token(root.path())?;
    let list_url = format!("http://{}/v1/models", service.addr);
    let body = client
        .get(list_url)
        .bearer_auth(&key)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    let catalog: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
    let public: Vec<_> = catalog["data"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["id"].as_str())
        .filter(|id| id.starts_with("auto/") || id.starts_with("group/auto-"))
        .collect();
    assert_eq!(public, ["auto/claude-opus-5-5"]);
    for (path, field, prefix) in [
        ("/v1beta/models", "name", "models/"),
        ("/api/tags", "model", ""),
    ] {
        let body = client
            .get(format!("http://{}{path}", service.addr))
            .bearer_auth(&key)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        let catalog: Value = nextjson::from_slice(&body).map_err(anyhow::Error::msg)?;
        let groups: Vec<_> = catalog["models"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|m| m[field].as_str()?.strip_prefix(prefix))
            .filter(|id| id.starts_with("auto/") || id.starts_with("group/auto-"))
            .collect();
        assert_eq!(groups, public);
    }
    for id in [
        "auto/Claude-Opus-5.5",
        "auto/CLAUDE-OPUS-5.5[1m]",
        "group/auto-claude-opus-5-5",
        "group/parent",
        "local-name",
    ] {
        let bytes = client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(&key)
            .header("content-type", "application/json")
            .body(json!({"model":id,"input":"hello"}).to_string())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        assert_eq!(
            nextjson::from_slice::<Value>(&bytes).map_err(anyhow::Error::msg)?["model"],
            json!("claude-opus-5-5@20260801")
        );
    }
    service.stop().await;
    assert!(run(&["group", "auto", "off"]).is_err()); // parent still depends on the group
    let previous = config::snapshot(root.path())?;
    assert!(run(&["group", "rename", name, "parent"]).is_err());
    assert_eq!(config::snapshot(root.path())?, previous);
    run(&["group", "rename", "auto/claude-opus-5.5", "chosen"])?;
    run(&["group", "rename", "chosen", "stable"])?;
    let renamed = config::load(root.path())?;
    assert_eq!(renamed.groups["parent"], ["group/stable"]);
    assert_eq!(renamed.group_weights["parent"]["group/stable"], 3);
    assert_eq!(renamed.former_groups[name], "stable");
    assert_eq!(renamed.former_groups["chosen"], "stable");
    assert!(renamed.hidden_auto_groups.contains(name));
    let service = Service::start(root.path()).await?;
    for id in [
        "group/chosen",
        "group/stable",
        "auto/claude-opus-5.5",
        "group/parent",
    ] {
        let reply = client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(&key)
            .body(json!({"model":id,"input":"hello"}).to_string())
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        assert_eq!(
            nextjson::from_slice::<Value>(&reply).map_err(anyhow::Error::msg)?["model"],
            json!("claude-opus-5-5@20260801")
        );
    }
    service.stop().await;
    run(&["group", "set", "chosen", "c/local-name"])?;
    run(&["group", "remove", "chosen"])?;
    assert!(
        !config::load(root.path())?
            .former_groups
            .contains_key("chosen")
    );
    // Restore the fixture before exercising hide/restore and provider removal.
    config::save(root.path(), &config::snapshot(root.path())?, &previous)?;
    run(&["group", "remove", "parent"])?;
    run(&["group", "remove", "auto/claude-opus-5.5"])?;
    assert!(run(&["group", "list"])?.is_empty());
    assert!(
        config::load(root.path())?
            .auto_stand_in("auto/claude-opus-5.5")
            .is_none()
    );
    run(&["group", "restore", name])?;
    assert_eq!(run(&["group", "list"])?, list);
    run(&["group", "auto", "off"])?;
    assert!(run(&["group", "list"])?.is_empty());
    run(&["group", "auto", "on"])?;
    run(&["group", "set", name, "c/local-name", "--rotate"])?;
    run(&["group", "auto", "off"])?;
    assert!(run(&["group", "list"])?.contains("rotate\tc/local-name"));
    let mut cfg = config::load(root.path())?;
    cfg.no_auto_groups = false;
    cfg.groups.clear();
    cfg.rotating_groups.clear();
    cfg.auto_group_skip.insert("a".into());
    assert_eq!(
        cfg.resolved_groups().members[name],
        ["b/claude-opus-5-5@20260801", "c/local-name"]
    );
    cfg.providers.pop();
    assert!(cfg.resolved_groups().members.is_empty());
    fs::write(root.path().join("settings.json"), cfg.to_json()?)?;
    let service = Service::start(root.path()).await?;
    let url = format!("http://{}/v1", service.addr);
    let catalog = client
        .get(format!("{url}/models"))
        .bearer_auth(&key)
        .send()
        .await?
        .text()
        .await?;
    assert!(!catalog.contains("group/auto-"));
    let bytes = client
        .post(format!("{url}/embeddings"))
        .bearer_auth(&key)
        .header("content-type", "application/json")
        .body(json!({"model":"auto/claude-opus-5.5", "input":"hello"}).to_string())
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    assert_eq!(
        nextjson::from_slice::<Value>(&bytes).map_err(anyhow::Error::msg)?["model"],
        json!("claude-opus-5-5@20260801")
    );
    service.stop().await;
    task.abort();
    Ok(())
}

#[test]
fn maker_prefixes_and_reserved_provider_follow_catalog_ownership() -> Result<()> {
    let mut cfg = config::Config::parse(&json!({"providers":[
        {"id":"kimi-code", "models":["k3.1-pro", "k3-256k"], "endpoints":{"chat":"http://localhost/v1"}},
        {"id":"relay", "models":["kimi-k3.1-pro", "kimi-k3"], "endpoints":{"chat":"http://localhost/v1"}}]}).to_string())?;
    let groups = cfg.resolved_groups();
    let id = "group/auto-kimi-k3-1-pro";
    assert_eq!(groups.members.len(), 1);
    assert_eq!(groups.public_refs[id], "auto/kimi-k3.1-pro");
    assert_eq!(groups.aliases["group/auto-k3-1-pro"], id);
    assert_eq!(
        groups.members["auto-kimi-k3-1-pro"],
        ["kimi-code/k3.1-pro", "relay/kimi-k3.1-pro"]
    );
    cfg.providers[1].id = "auto".into();
    let groups = cfg.resolved_groups();
    assert!(groups.public_refs.is_empty());
    assert!(!groups.aliases.keys().any(|name| name.starts_with("auto/")));
    assert!(cfg.canonical_auto_ref("auto/kimi-k3.1-pro").is_none());
    let legacy = tempfile::tempdir()?;
    let saved = cfg.to_json()?;
    assert!(config::save(legacy.path(), &config::snapshot(legacy.path())?, &saved).is_err());
    fs::write(legacy.path().join("settings.json"), &saved)?;
    config::save(legacy.path(), &config::snapshot(legacy.path())?, &saved)?;
    Ok(())
}

#[test]
fn group_spelling_prefers_vendor_ids_and_survives_provider_exclusion() -> Result<()> {
    let mut cfg = config::Config::parse(
        &json!({"providers":[
        {"id":"alias","models":["local"],"endpoints":{"chat":"http://localhost/v1"}},
        {"id":"named","models":["New Model 9.1"],"endpoints":{"chat":"http://localhost/v1"}},
        {"id":"vendor","models":["New-Model-9.1"],"endpoints":{"chat":"http://localhost/v1"}}],
        "model_same_as":{"alias/local":"new_model_9p1"}})
        .to_string(),
    )?;
    let id = "group/auto-new-model-9-1";
    assert_eq!(cfg.resolved_groups().public_refs[id], "auto/New-Model-9.1");
    cfg.groups.insert(
        "auto-new-model-9-1".into(),
        vec!["named/New Model 9.1".into()],
    );
    cfg.auto_group_skip
        .extend(cfg.providers.iter().map(|p| p.id.clone()));
    assert_eq!(cfg.resolved_groups().public_refs[id], "auto/new-model-9-1");
    Ok(())
}

#[test]
fn custom_catalogs_match_only_their_own_models_and_an_existing_group_owns_its_former_id()
-> Result<()> {
    let cfg = config::Config::parse(&json!({"providers":[
        {"id":"custom", "catalog":"unknown, KIMI-CODE-PLAN-CN, CLOUDFLARE-AI-GATEWAY vercel digitalocean ofox", "models":["k3.1-pro","m3.1-flash","namazu","multilingual-v2","m2-her"], "endpoints":{"chat":"http://localhost/v1"}},
        {"id":"vendor", "models":["kimi-k3.1-pro","minimax-m3.1-flash","sakana-namazu","eleven_multilingual_v2","minimax-m2-her"], "endpoints":{"chat":"http://localhost/v1"}},
        {"id":"other-a", "models":["k3.1-pro"], "endpoints":{"chat":"http://localhost/v1"}},
        {"id":"other-b", "models":["k3.1-pro"], "endpoints":{"chat":"http://localhost/v1"}}],
        "groups":{"short":["group/auto-k3-1-pro"],"public":["auto/k3.1-pro"]}}).to_string())?;
    let groups = cfg.resolved_groups();
    for (short, full, slug) in [
        ("k3.1-pro", "kimi-k3.1-pro", "kimi-k3-1-pro"),
        ("m3.1-flash", "minimax-m3.1-flash", "minimax-m3-1-flash"),
        ("namazu", "sakana-namazu", "sakana-namazu"),
        (
            "multilingual-v2",
            "eleven_multilingual_v2",
            "eleven-multilingual-v2",
        ),
        ("m2-her", "minimax-m2-her", "minimax-m2-her"),
    ] {
        assert_eq!(
            groups.members[&format!("auto-{slug}")],
            [format!("custom/{short}"), format!("vendor/{full}")]
        );
    }
    for parent in ["short", "public"] {
        assert_eq!(groups.members[parent], ["group/auto-k3-1-pro"]);
    }
    assert_eq!(
        groups.members["auto-k3-1-pro"],
        ["other-a/k3.1-pro", "other-b/k3.1-pro"]
    );
    let renamed = cfg.renamed_group("auto/kimi-k3.1-pro", "chosen")?;
    assert!(!renamed.former_groups.contains_key("auto-k3-1-pro"));
    let mut cfg = cfg;
    cfg.providers.truncate(2);
    assert_eq!(
        cfg.resolved_groups().members["short"],
        ["group/auto-kimi-k3-1-pro"]
    );
    Ok(())
}
