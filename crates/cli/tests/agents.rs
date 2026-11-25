use anyhow::{Context, Result};
use axum::{Router, body::Bytes, http::HeaderMap, routing::post};
use crabforge_core::{config, service::Service};
use serde_json::{Value, json};
use std::{env, fs, process::Stdio, time::Duration};
use tokio::{net::TcpListener, process::Command, time::timeout};

#[tokio::test]
async fn every_advertised_agent_root_forwards_and_records_its_own_name() -> Result<()> {
    let root = tempfile::tempdir()?;
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let base = format!("http://{}/v1", listener.local_addr()?);
    let app = Router::new().route("/v1/embeddings", post(|| async { "fixture" }));
    let upstream = tokio::spawn(async move { axum::serve(listener, app).await });
    let mut cfg = json!({"listen":"127.0.0.1:0", "providers":[{"id":"local","endpoints":{"chat":base},"models":["m"]}]});
    fs::write(root.path().join("settings.json"), cfg.to_string())?;
    let service = Service::start(root.path()).await?;
    cfg["listen"] = json!(service.addr);
    fs::write(root.path().join("settings.json"), cfg.to_string())?;
    let out = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"))
        .arg("--data-dir")
        .arg(root.path())
        .arg("agents")
        .output()
        .await?;
    assert!(out.status.success());
    let advertised = String::from_utf8(out.stdout)?;
    let client = reqwest::Client::builder().no_proxy().build()?;
    let key = config::token(root.path())?;
    let mut ids = std::collections::HashSet::new();
    for row in advertised.lines() {
        let (name, url) = row.split_once('\t').context("invalid agent listing")?;
        assert!(ids.insert(url));
        let response = client
            .post(format!("{url}/v1/embeddings"))
            .header("user-agent", "curl/8.0")
            .bearer_auth(&key)
            .body(r#"{"model":"local/m","input":"fixture"}"#)
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(response.text().await?, "fixture");
        assert_eq!(
            service
                .history
                .lock()
                .unwrap()
                .back()
                .unwrap()
                .agent
                .as_deref(),
            Some(name)
        );
    }
    assert_eq!(ids.len(), 93);
    for (ua, expected) in [
        ("Cursor-Local/1.0", "Cursor Private Inference"),
        ("Pi-Desktop/1.2", "PI-Desktop"),
        ("codex_cli_rs/0.160.0 (Windows)", "Codex"),
        ("DimAgent/0.5.29", "DimCode"),
        ("dimcode-alpha/0.1.0", "Dim"),
        ("githubcopilotchat/1", "VS Code"),
        ("KIMI-CODE-CLI/2.8.0", "Kimi Code"),
        ("Python-urllib/3.12", "Python-urllib"),
    ] {
        client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .header("user-agent", ua)
            .bearer_auth(&key)
            .body(r#"{"model":"local/m","input":"fixture"}"#)
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(
            service
                .history
                .lock()
                .unwrap()
                .back()
                .unwrap()
                .agent
                .as_deref(),
            Some(expected)
        );
    }
    let native = "n".repeat(140);
    for (name, value, session, original) in [
        (
            "x-conversation-id",
            "thread",
            Some("thread"),
            Some("thread"),
        ),
        (
            "x-deepseek-harness-session-id",
            native.as_str(),
            Some(&native[..128]),
            Some(&native[..128]),
        ),
        ("x-magpie-session", "legacy", Some("legacy"), None),
        ("x-crabforge-session", "local", Some("local"), None),
        ("x-conversation-request-id", "only-a-turn", None, None),
    ] {
        let response = client
            .post(format!("http://{}/v1/embeddings", service.addr))
            .bearer_auth(&key)
            .header(name, value)
            .body(r#"{"model":"m","input":"fixture"}"#)
            .send()
            .await?
            .error_for_status()?;
        assert_eq!(
            response
                .headers()
                .get("x-magpie-session")
                .and_then(|v| v.to_str().ok()),
            session
        );
        let history = service.history.lock().unwrap();
        let attempt = history.back().unwrap();
        assert_eq!(attempt.session.as_deref(), session);
        assert_eq!(attempt.native_session.as_deref(), original);
    }
    service.stop().await;
    upstream.abort();
    Ok(())
}

#[tokio::test]
#[ignore = "requires the Codex CLI on PATH"]
async fn codex_exec_receives_assistant_text_through_the_gateway() -> Result<()> {
    client_round(
        "codex",
        "Codex",
        "responses",
        "v1",
        responses_stream(),
        &[
            "exec",
            "--skip-git-repo-check",
            "--sandbox",
            "read-only",
            "Please say hello.",
        ],
    )
    .await
}

fn responses_stream() -> Router {
    Router::new().route("/v1/responses",post(|headers:HeaderMap,bytes:Bytes|async move {
        assert!(!headers.contains_key("authorization"));
        let request:Value=serde_json::from_slice(&bytes).unwrap();
        assert_eq!(request["model"],"m");assert_eq!(request["stream"],true);
        let item=json!({"type":"message","id":"msg_fixture","role":"assistant","status":"completed","content":[{"type":"output_text","text":"CRABFORGE_FIXTURE_REPLY","annotations":[]}]});
        let response=json!({"id":"resp_fixture","object":"response","model":"m","created_at":1700000000,"status":"completed","output":[item],"usage":{"input_tokens":12,"output_tokens":2,"total_tokens":14}});
        let mut frames=String::new();
        for event in [json!({"type":"response.created","response":{"id":"resp_fixture","object":"response","status":"in_progress","output":[]}}),json!({"type":"response.output_item.added","output_index":0,"item":{"type":"message","id":"msg_fixture","role":"assistant","content":[]}}),json!({"type":"response.output_text.delta","item_id":"msg_fixture","output_index":0,"content_index":0,"delta":"CRABFORGE_FIXTURE_REPLY"}),json!({"type":"response.output_item.done","output_index":0,"item":item}),json!({"type":"response.completed","response":response})] {
            frames.push_str(&format!("event: {}\ndata: {event}\n\n",event["type"].as_str().unwrap()));
        }
        ([("content-type","text/event-stream")],frames)
    }))
}

#[tokio::test]
#[ignore = "requires the Gemini CLI on PATH"]
async fn gemini_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "gemini",
        "Gemini CLI",
        "gemini",
        "v1beta",
        gemini_stream(),
        &["--skip-trust", "-p", "Please say hello."],
    )
    .await
}

fn gemini_stream() -> Router {
    Router::new().route("/v1beta/models/m:streamGenerateContent",post(||async {
        let reply=json!({"candidates":[{"content":{"role":"model","parts":[{"text":"CRABFORGE_FIXTURE_REPLY"}]},"index":0,"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":3,"candidatesTokenCount":2,"totalTokenCount":5},"modelVersion":"m"});
        ([("content-type","text/event-stream")],format!("data: {reply}\r\n\r\n"))
    }))
}

#[tokio::test]
#[ignore = "requires the Claude Code CLI on PATH"]
async fn claude_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "claude",
        "Claude Code",
        "anthropic",
        "v1",
        messages_stream(),
        &["-p", "Please say hello.", "--tools", ""],
    )
    .await
}

fn messages_stream() -> Router {
    Router::new().route("/v1/messages",post(|bytes:Bytes|async move {
        let req:Value=serde_json::from_slice(&bytes).unwrap();
        let part=json!({"type":"text","text":"CRABFORGE_FIXTURE_REPLY"});
        let mut message=json!({"id":"msg_fixture","type":"message","role":"assistant","model":"m","content":[part],"stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":3,"output_tokens":2}});
        if req["stream"]!=true{return ([("content-type","application/json")],message.to_string());}
        message["content"]=json!([]);message["stop_reason"]=Value::Null;message["usage"]["output_tokens"]=json!(0);
        let mut frames=String::new();
        for event in [json!({"type":"message_start","message":message}),json!({"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}),json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"CRABFORGE_FIXTURE_REPLY"}}),json!({"type":"content_block_stop","index":0}),json!({"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":2}}),json!({"type":"message_stop"})]{frames.push_str(&format!("event: {}\ndata: {event}\n\n",event["type"].as_str().unwrap()));}
        ([("content-type","text/event-stream")],frames)
    }))
}

#[tokio::test]
#[ignore = "requires the Goose CLI on PATH"]
async fn goose_receives_text_through_the_gateway() -> Result<()> {
    let app = Router::new().route("/v1/chat/completions", post(|bytes: Bytes| async move {
        let req: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(req["model"], "m");
        if req["stream"] == true {
            let chunk = json!({"id":"chat-goose","object":"chat.completion.chunk","created":1,"model":"m","choices":[{"index":0,"delta":{"role":"assistant","content":"CRABFORGE_FIXTURE_REPLY"},"finish_reason":"stop"}]});
            ([("content-type","text/event-stream")],format!("data: {chunk}\n\ndata: [DONE]\n\n"))
        } else {
            let reply = json!({"id":"chat-goose","object":"chat.completion","created":1,"model":"m","choices":[{"index":0,"message":{"role":"assistant","content":"CRABFORGE_FIXTURE_REPLY"},"finish_reason":"stop"}],"usage":{"prompt_tokens":2,"completion_tokens":1,"total_tokens":3}});
            ([("content-type","application/json")],reply.to_string())
        }
    }));
    client_round(
        "goose",
        "Goose",
        "chat",
        "v1",
        app,
        &[
            "run",
            "--no-profile",
            "--no-session",
            "--max-turns",
            "1",
            "-t",
            "Please say hello.",
        ],
    )
    .await
}

#[tokio::test]
#[ignore = "requires the OpenCode V2 CLI on PATH"]
async fn opencode_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "opencode",
        "OpenCode",
        "chat",
        "v1",
        chat_stream(),
        &["run", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires the GitHub Copilot CLI on PATH"]
async fn copilot_uses_the_gateway_in_offline_mode() -> Result<()> {
    client_round(
        "copilot",
        "Copilot CLI",
        "chat",
        "v1",
        chat_stream(),
        &["-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires the MiMo Code CLI on PATH"]
async fn mimocode_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "mimocode",
        "MiMo Code",
        "chat",
        "v1",
        chat_stream(),
        &["run", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires the omp CLI and its Bun runtime on PATH"]
async fn omp_receives_text_through_the_gateway() -> Result<()> {
    for (protocol, prefix, app) in [
        ("chat", "v1", chat_stream()),
        ("responses", "v1", responses_stream()),
        ("anthropic", "v1", messages_stream()),
        ("gemini", "v1beta", gemini_stream()),
    ] {
        client_round(
            "omp",
            "omp",
            protocol,
            prefix,
            app,
            &["--offline", "--no-session", "-p", "Please say hello."],
        )
        .await
        .with_context(|| format!("omp on {protocol}"))?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires the omp CLI and its Bun runtime on PATH"]
async fn omp_discovers_profiles_and_writes_the_directory_selected_by_native_flags() -> Result<()> {
    let scratch = tempfile::tempdir()?;
    let root = scratch.path();
    let profiles = root.join("custom-omp/profiles");
    for name in ["existing", "Bad", "default"] {
        fs::create_dir_all(profiles.join(name))?;
    }
    fs::write(profiles.join("notes"), "personal")?;
    let cfg = json!({"providers":[{"id":"local", "endpoints":{"chat":"http://127.0.0.1:1/v1"}, "models":["m"]}]});
    fs::write(root.join("settings.json"), cfg.to_string())?;
    let cli = || {
        let mut c = isolated_cli(root);
        c.env("PI_CONFIG_DIR", "custom-omp")
            .env("OMP_PROFILE", "active")
            .env("PI_PROFILE", "ignored")
            .env("PI_CODING_AGENT_DIR", root.join("personal-pi"));
        c
    };
    let listing = cli().arg("omp-profiles").output().await?;
    assert!(
        listing.status.success(),
        "{}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let rows = String::from_utf8(listing.stdout)?;
    assert_eq!(
        rows.lines()
            .map(|r| r.split('\t').take(2).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [
            ["default", "available"],
            ["active", "active"],
            ["existing", "available"]
        ]
    );
    for (args, path) in [
        (
            vec![
                "--profile=wrapper",
                "--",
                "--offline",
                "--profile",
                "existing",
                "--version",
            ],
            profiles.join("existing/agent/models.yml"),
        ),
        (
            vec!["--profile=default", "--", "--version"],
            root.join("agents/omp/models.yml"),
        ),
        (
            vec!["--profile=", "--", "--version"],
            root.join("agents/omp/models.yml"),
        ),
    ] {
        let out = cli()
            .args(["agent", "omp", "local/m"])
            .args(args)
            .output()
            .await?;
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(fs::read_to_string(path)?.contains("local/m"));
    }
    let invalid = cli()
        .args([
            "agent",
            "omp",
            "local/m",
            "--profile=../bad",
            "--",
            "--version",
        ])
        .output()
        .await?;
    assert!(!invalid.status.success());
    assert!(
        !profiles.join("wrapper").exists()
            && !profiles.join("active/agent/models.yml").exists()
            && !root.join("personal-pi/models.yml").exists()
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires the DevEco Code CLI on PATH"]
async fn deveco_loads_the_selected_model_and_authenticated_agent_endpoint() -> Result<()> {
    let scratch = tempfile::tempdir()?;
    let root = scratch.path();
    let cfg = json!({"listen":"127.0.0.1:3425", "providers":[{"id":"local","models":["m"],"endpoints":{"chat":"http://127.0.0.1:1/v1"}}]});
    fs::write(root.join("settings.json"), cfg.to_string())?;
    let output = timeout(
        Duration::from_secs(60),
        isolated_cli(root)
            .args(["agent", "deveco", "local/m", "--", "debug", "config"])
            .output(),
    )
    .await??;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(actual["model"], "crabforge/local/m");
    assert_eq!(actual["small_model"], "crabforge/local/m");
    let provider = &actual["provider"]["crabforge"];
    assert_eq!(
        provider["options"]["baseURL"],
        "http://127.0.0.1:3425/agent/deveco/v1"
    );
    assert!(provider["options"]["apiKey"].as_str() == Some(config::token(root)?.as_str()));
    assert!(provider["models"]["local/m"].is_object());
    assert!(fs::read_dir(root.join("agents/deveco"))?.next().is_none());
    Ok(())
}

#[tokio::test]
#[ignore = "requires the Pi CLI on PATH"]
async fn pi_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "pi",
        "Pi",
        "chat",
        "v1",
        chat_stream(),
        &["--no-session", "--no-tools", "-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires Qwen Code on PATH"]
async fn qwen_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "qwen",
        "Qwen Code",
        "chat",
        "v1",
        chat_stream(),
        &["--approval-mode", "plan", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires Mistral Vibe on PATH"]
async fn vibe_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "vibe",
        "Mistral Vibe",
        "chat",
        "v1",
        chat_stream(),
        &["--max-turns", "1", "-p", "Please say hello."],
    )
    .await
}

fn chat_stream() -> Router {
    Router::new().route("/v1/chat/completions", post(|bytes: Bytes| async move {
        let req: Value = serde_json::from_slice(&bytes).unwrap();
        let model=req["model"].as_str().unwrap();
        assert!(matches!(model,"m"|"n"));
        assert_eq!(req["stream"], true);
        let delta = json!({"id":"chatcmpl-fixture","object":"chat.completion.chunk","model":model,"choices":[{"index":0,"delta":{"role":"assistant","content":format!("CRABFORGE_FIXTURE_REPLY_{model}")},"finish_reason":null}]});
        let stop = json!({"id":"chatcmpl-fixture","object":"chat.completion.chunk","model":model,"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]});
        let mut frames = format!("data: {delta}\n\ndata: {stop}\n\n");
        if req["stream_options"]["include_usage"] == true {
            let usage = json!({"id":"chatcmpl-fixture","object":"chat.completion.chunk","model":model,"choices":[],"usage":{"prompt_tokens":10,"completion_tokens":2,"total_tokens":12}});
            frames.push_str(&format!("data: {usage}\n\n"));
        }
        frames.push_str("data: [DONE]\n\n");
        ([("content-type","text/event-stream")], frames)
    }))
}

#[tokio::test]
#[ignore = "requires omo-ai on PATH"]
async fn omo_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "omo",
        "OmO",
        "chat",
        "v1",
        chat_stream(),
        &["--offline", "--no-session", "-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires mu-agent on PATH"]
async fn mu_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "mu",
        "mu",
        "chat",
        "v1",
        chat_stream(),
        &["--no-session", "-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires prime-agent on PATH"]
async fn prime_agent_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "prime-agent",
        "Prime Agent",
        "chat",
        "v1",
        chat_stream(),
        &["--offline", "--no-session", "-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires Kimi Code on PATH"]
async fn kimi_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "kimi",
        "Kimi Code",
        "chat",
        "v1",
        chat_stream(),
        &["-p", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires Crush on PATH"]
async fn crush_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "crush",
        "Crush",
        "chat",
        "v1",
        chat_stream(),
        &["run", "--quiet", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires nanobot-ai on PATH"]
async fn nanobot_receives_text_through_the_gateway() -> Result<()> {
    client_round(
        "nanobot",
        "nanobot",
        "chat",
        "v1",
        chat_stream(),
        &["--no-markdown", "-m", "Please say hello."],
    )
    .await
}

#[tokio::test]
#[ignore = "requires the Pi CLI on PATH"]
async fn pi_returns_the_result_of_a_streamed_read_call() -> Result<()> {
    use axum::http::StatusCode;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let app = Router::new().route("/v1/chat/completions", post(move |bytes: Bytes| {
        let count = count.clone();
        async move {
            let req: Value = serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?;
            let round = count.fetch_add(1, Ordering::Relaxed);
            let (deltas, finish) = match round {
                0 if req["tools"].as_array().is_some_and(|ts| ts.iter().any(|t| t["function"]["name"] == "read")) => (vec![
                    json!({"role":"assistant","tool_calls":[{"index":0,"id":"call_fixture","type":"function","function":{"name":"read","arguments":"{\"path\":"}}]}),
                    json!({"tool_calls":[{"index":0,"function":{"arguments":"\"settings.json\"}"}}]}),
                ], "tool_calls"),
                1 => {
                    let result = req["messages"].as_array().and_then(|ms| ms.last()).ok_or(StatusCode::BAD_REQUEST)?;
                    if result["role"] != "tool" || result["tool_call_id"] != "call_fixture" || !result["content"].to_string().contains("providers") {
                        return Err(StatusCode::BAD_REQUEST);
                    }
                    (vec![json!({"role":"assistant","content":"CRABFORGE_FIXTURE_REPLY"})], "stop")
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            };
            let mut frames = String::new();
            let chunks = deltas.into_iter().map(|d| (d, Value::Null)).chain([(json!({}), json!(finish))]);
            for (delta, reason) in chunks {
                let chunk = json!({"id":"chatcmpl-fixture","object":"chat.completion.chunk","model":"m","choices":[{"index":0,"delta":delta,"finish_reason":reason}]});
                frames.push_str(&format!("data: {chunk}\n\n"));
            }
            frames.push_str("data: [DONE]\n\n");
            Ok(([("content-type", "text/event-stream")], frames))
        }
    }));
    client_round(
        "pi",
        "Pi",
        "chat",
        "v1",
        app,
        &[
            "--no-session",
            "--tools",
            "read",
            "-p",
            "Read settings.json.",
        ],
    )
    .await?;
    assert_eq!(calls.load(Ordering::Relaxed), 2);
    Ok(())
}

async fn client_round(
    client: &str,
    name: &str,
    protocol: &str,
    prefix: &str,
    app: Router,
    args: &[&str],
) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let upstream = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, app).await });
    let scratch = tempfile::tempdir()?;
    let root = scratch.path();
    let pi_format = matches!(client, "pi" | "mu" | "omo" | "prime-agent");
    let omp_profile = match protocol {
        "responses" => "work",
        "anthropic" => "messages",
        "gemini" => "google",
        _ => "",
    };
    let omp_file = if omp_profile.is_empty() {
        root.join("agents/omp/models.yml")
    } else {
        root.join("omp-test/profiles")
            .join(omp_profile)
            .join("agent/models.yml")
    };
    if client == "omp" {
        fs::create_dir_all(omp_file.parent().unwrap())?;
        fs::write(
            &omp_file,
            "providers:\n  personal:\n    api: openai-completions\n    apiKey: TEST_PERSONAL_KEY\n    baseUrl: http://127.0.0.1:1/v1\n    models:\n      - id: personal\n",
        )?;
    }
    let model_path = root.join("agents").join(client).join("models.json");
    if pi_format {
        fs::create_dir_all(model_path.parent().unwrap())?;
        fs::write(
            &model_path,
            r#"{"providers":{"personal":{"baseUrl":"http://127.0.0.1:1/v1","api":"openai-completions","apiKey":"personal-fixture","models":[{"id":"personal"}]}}}"#,
        )?;
    }
    let nanobot_home = root.join("agents/nanobot");
    let copilot_file = root.join("agents/copilot/providers.json");
    let copilot_personal = r#"{"providers":[{"name":"personal","type":"openai","baseUrl":"http://127.0.0.1:1/v1"}],"models":[{"id":"p","provider":"personal"}]}"#;
    if client == "copilot" {
        fs::create_dir_all(copilot_file.parent().unwrap())?;
        fs::write(&copilot_file, copilot_personal)?;
    }
    let personal = r#"{"agents":{"defaults":{"modelPreset":"personal"}},"modelPresets":{"personal":{"provider":"openai","model":"personal"}}}"#;
    if client == "nanobot" {
        fs::create_dir_all(&nanobot_home)?;
        fs::write(nanobot_home.join("config.json"), personal)?;
    }
    if client == "kimi" {
        fs::create_dir_all(root.join("agents/kimi"))?;
        fs::write(
            root.join("agents/kimi/config.toml"),
            r#"# my Kimi settings
default_model = 'personal'
[providers.personal]
type = 'openai'
base_url = 'http://127.0.0.1:1/v1'
api_key = 'test-only'
[models.personal]
provider = 'personal'
model = 'personal'
max_context_size = 65536
[models.'crabforge/gone']
provider = 'crabforge'
model = 'gone'
max_context_size = 4096
[models.'crabforge/local/m']
provider = 'crabforge'
model = 'local/m'
max_context_size = 32768
"#,
        )?;
    }
    if client == "vibe" {
        fs::create_dir_all(root.join("agents/vibe"))?;
        fs::write(
            root.join("agents/vibe/config.toml"),
            "# personal settings\nactive_model = 'personal'\nmodels = [{ name = 'personal', alias = 'personal', provider = 'other' }, {name = 'gone', alias = 'crabforge/gone', provider = 'crabforge'}]\n[[providers]]\nname = 'other'\napi_base = 'http://127.0.0.1:1/v1'\n",
        )?;
    }
    if client == "crush" {
        fs::create_dir_all(root.join("agents/crush/config"))?;
        fs::write(
            root.join("agents/crush/config/crush.json"),
            r#"{"options":{"disable_metrics":true},"providers":{"personal":{"name":"Personal","type":"completions","base_url":"http://127.0.0.1:1/v1","models":[{"id":"personal","name":"Personal"}]}}}"#,
        )?;
    }
    let home = root.join("codex");
    fs::create_dir(&home)?;
    let parallel = matches!(client, "crush" | "vibe");
    let models = if parallel {
        vec!["m", "n", "unlisted"]
    } else {
        vec!["m", "unlisted"]
    };
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"local","models":models,"endpoints":{protocol:format!("http://{upstream}/{prefix}")}}],"groups":{"first":["local/unlisted"],"last":["local/m"],"paused":["local/m"]},"disabled_groups":["paused"],"group_unlists":["first"],"group_display_order":["paused","last","first"]});
    config::save(root, &config::snapshot(root)?, &cfg.to_string())?;
    let service = Service::start(root).await?;
    let mut cfg = cfg;
    cfg["listen"] = json!(service.addr.to_string());
    config::save(root, &config::snapshot(root)?, &cfg.to_string())?;
    let run = |selection: &'static str| async move {
        let mut command = isolated_cli(root);
        command
            .env("CODEX_HOME", root.join("codex"))
            .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
            .env("GEMINI_TELEMETRY_ENABLED", "false")
            .env("CRUSH_DISABLE_PROVIDER_AUTO_UPDATE", "1")
            .env("COPILOT_OFFLINE", "true")
            .env(
                "COPILOT_PROVIDERS_CONFIG",
                root.join("agents/copilot/providers.json"),
            )
            .env("COPILOT_PROVIDER_BEARER_TOKEN", "personal-test-token")
            .env("COPILOT_PROVIDER_WIRE_MODEL", "unused-test-model")
            .env("MIMOCODE_DISABLE_AUTOUPDATE", "1")
            .env("MIMOCODE_DISABLE_MODELS_FETCH", "1")
            .env("NO_BROWSER", "true")
            .args(["agent", client, selection]);
        if client == "omp" {
            command
                .env("PI_CONFIG_DIR", "omp-test")
                .env("PI_PROFILE", "fallback");
            match protocol {
                "responses" => {
                    command
                        .env("OMP_PROFILE", "unused")
                        .args(["--profile", omp_profile]);
                }
                "anthropic" => {
                    command.env("OMP_PROFILE", omp_profile);
                }
                "gemini" => {
                    command.env("PI_PROFILE", omp_profile);
                }
                _ => {
                    command.env("OMP_PROFILE", "");
                }
            }
        }
        command.arg("--").args(args);
        let out = timeout(Duration::from_secs(90), command.output())
            .await
            .with_context(|| format!("{name} timed out"))??;
        assert!(
            out.status.success(),
            "{} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("CRABFORGE_FIXTURE_REPLY"),
            "missing response from {name} for {selection}, stdout {:?}, stderr {:?}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        if parallel {
            assert!(String::from_utf8_lossy(&out.stdout).contains(&format!(
                "CRABFORGE_FIXTURE_REPLY_{}",
                selection.strip_prefix("local/").unwrap()
            )));
        }
        Ok::<(), anyhow::Error>(())
    };
    if parallel {
        tokio::try_join!(run("local/m"), run("local/n"))?;
        for id in ["local/m", "local/n"] {
            assert!(
                service
                    .history
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|a| a.agent.as_deref() == Some(name) && a.model == id && a.status == 200),
                "missing {name} request for {id}"
            );
        }
    } else {
        run("local/m").await?;
    }
    assert!(
        service
            .history
            .lock()
            .unwrap()
            .iter()
            .any(|a| a.agent.as_deref() == Some(name) && a.status == 200)
    );
    if client == "kimi" {
        let text = fs::read_to_string(root.join("agents/kimi/config.toml"))?;
        let config: toml_edit::DocumentMut = text.parse()?;
        assert_eq!(config["default_model"].as_str(), Some("personal"));
        assert_eq!(
            config["models"]["crabforge/local/m"]["max_context_size"].as_integer(),
            Some(32768)
        );
        assert!(text.contains("# my Kimi settings") && !text.contains("crabforge/gone"));
        assert!(!text.contains(&config::token(root)?));
    }
    if matches!(client, "mimocode" | "opencode") {
        assert!(
            fs::read_dir(root.join("agents").join(client))?
                .next()
                .is_none()
        );
    }
    if client == "copilot" {
        assert_eq!(fs::read_to_string(copilot_file)?, copilot_personal);
    }
    if client == "omp" {
        assert!(
            !root
                .join("omp-test/profiles/fallback/agent/models.yml")
                .exists()
        );
        assert!(
            !root
                .join("omp-test/profiles/unused/agent/models.yml")
                .exists()
        );
        let text = fs::read_to_string(omp_file)?;
        let doc: Value = serde_yaml::from_str(&text)?;
        assert_eq!(doc["providers"]["personal"]["models"][0]["id"], "personal");
        let models = doc["providers"]["crabforge"]["models"].as_array().unwrap();
        let direct = models.iter().find(|m| m["id"] == "local/m").unwrap();
        let expected = match protocol {
            "responses" => "openai-responses",
            "anthropic" => "anthropic-messages",
            "gemini" => "google-generative-ai",
            _ => "openai-completions",
        };
        assert_eq!(
            direct["api"].as_str().unwrap_or("openai-completions"),
            expected
        );
        assert_eq!(
            models[0]["api"].as_str(),
            (protocol == "responses").then_some("openai-responses")
        );
        assert_eq!(
            doc["providers"]["crabforge"]["models"][0]["id"],
            "group/last"
        );
        assert!(!text.contains("group/paused") && !text.contains(&config::token(root)?));
        assert_eq!(
            doc["providers"]["crabforge"]["compat"]["promptCacheSessionHeader"],
            "x-session-affinity"
        );
    }
    if client == "pi" || (client == "omp" && protocol == "chat") {
        assert!(
            service
                .history
                .lock()
                .unwrap()
                .iter()
                .any(|a| a.agent.as_deref() == Some(name)
                    && a.status == 200
                    && a.native_session.is_some()
                    && a.session == a.native_session),
            "{client} did not name its session"
        );
    }
    if client == "qwen" {
        assert!(
            fs::read_dir(root.join("agents/qwen"))?.next().is_some(),
            "Qwen profile is empty"
        );
    }
    if pi_format {
        let text = fs::read_to_string(model_path)?;
        let catalog: Value = serde_json::from_str(&text)?;
        let ours = &catalog["providers"]["crabforge"]["models"];
        assert_eq!(ours[0]["id"], "group/last");
        assert_eq!(ours[1]["id"], "group/first");
        assert!(!ours.as_array().unwrap().iter().any(|m| matches!(
            m["id"].as_str(),
            Some("unlisted" | "local/unlisted" | "group/paused")
        )));
        assert_eq!(
            catalog["providers"]["personal"]["models"][0]["id"],
            "personal"
        );
        assert!(!text.contains(&config::token(root)?));
    }
    if client == "nanobot" {
        assert_eq!(
            fs::read_to_string(nanobot_home.join("config.json"))?,
            personal
        );
        assert!(
            fs::read_dir(nanobot_home.join("sessions"))?
                .next()
                .is_some()
        );
        for entry in fs::read_dir(nanobot_home)? {
            assert!(!entry?.file_name().to_string_lossy().starts_with("launch-"));
        }
    }
    if client == "vibe" {
        let text = fs::read_to_string(root.join("agents/vibe/config.toml"))?;
        let doc: toml_edit::DocumentMut = text.parse()?;
        assert_eq!(doc["active_model"].as_str(), Some("personal"));
        assert_eq!(doc["providers"].as_array_of_tables().unwrap().len(), 2);
        assert!(text.contains("# personal settings") && text.contains("crabforge/local/m"));
        assert!(!text.contains("crabforge/gone"));
        assert!(!text.contains(&config::token(root)?));
    }
    if client == "crush" {
        let text = fs::read_to_string(root.join("agents/crush/config/crush.json"))?;
        let saved: Value = serde_json::from_str(&text)?;
        assert_eq!(saved["providers"]["personal"]["name"], "Personal");
        assert_eq!(saved["options"]["disable_metrics"], true);
        assert!(!text.contains(&config::token(root)?));
    }
    service.stop().await;
    server.abort();
    let _ = server.await;
    Ok(())
}

fn isolated_cli(root: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_carbforge-cli"));
    command.env_clear();
    for k in [
        "PATH",
        "SystemRoot",
        "WINDIR",
        "ComSpec",
        "PATHEXT",
        "SystemDrive",
    ] {
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
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "XDG_STATE_HOME",
    ] {
        command.env(k, root);
    }
    command
        .current_dir(root)
        .env("CRABFORGE_HOME", root)
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("NO_PROXY", "127.0.0.1,localhost")
        .env("DEVECO_DISABLE_AUTOUPDATE", "1")
        .env("DEVECO_DISABLE_MODELS_FETCH", "1")
        .stdin(Stdio::null())
        .kill_on_drop(true);
    command
}
