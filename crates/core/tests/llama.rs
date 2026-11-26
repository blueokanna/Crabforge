use anyhow::{Context, Result, ensure};
use crabforge_core::{
    catalog,
    config::{self, Endpoints},
    service::Service,
};
use serde_json::{Value, json};
use std::time::Duration;

#[path = "support/llama.rs"]
mod llama;

#[tokio::test]
#[ignore = "requires LLAMA_TEST_SERVER, LLAMA_TEST_GGUF and OLLAMA_TEST_PYTHON with the Ollama SDK"]
async fn official_ollama_client_lists_models_and_streams_real_tokens() -> Result<()> {
    let python = std::env::var_os("OLLAMA_TEST_PYTHON")
        .context("set OLLAMA_TEST_PYTHON to Python with the ollama SDK installed")?;
    let (mut server, addr) = llama::start("smollm2").await?;
    let dir = tempfile::tempdir()?;
    let cfg = json!({"listen":"127.0.0.1:0","providers":[{"id":"llama","models":["smollm2"],"endpoints":{"chat":format!("http://{addr}/v1")}}]});
    config::save(dir.path(), &config::snapshot(dir.path())?, &cfg.to_string())?;
    let gateway = Service::start(dir.path()).await?;
    let test = r#"
import os
from importlib.metadata import version
from ollama import Client, ResponseError

client = Client(host=os.environ['TEST_HOST'], headers={'Authorization':'Bearer ' + os.environ['TEST_KEY']}, timeout=15, trust_env=False)
model = 'llama/smollm2'
assert model in [item.model for item in client.list().models]
assert client.show(model + ':latest').modelinfo['general.basename'] == model
options = {'num_predict': 16, 'temperature': 0}
prompt = 'Complete the sentence with one word: The sky is'
reply = client.chat(model, messages=[{'role':'user', 'content':prompt}], options=options)
chunks = list(client.generate(model, prompt, options=options, stream=True))
assert chunks[-1].done and all(not chunk.done for chunk in chunks[:-1])
answer = ''.join(chunk.response for chunk in chunks)
assert answer.strip() and answer == reply.message.content
assert 0 < chunks[-1].eval_count == reply.eval_count <= 16
try:
    client.show('unconfigured')
except ResponseError as exc:
    assert exc.status_code == 404 and 'unconfigured' in exc.error
else:
    raise AssertionError('unknown model accepted')
print('ollama', version('ollama'), repr(answer), chunks[-1].eval_count, 'tokens')
"#;
    let output = tokio::time::timeout(
        Duration::from_secs(45),
        tokio::process::Command::new(python)
            .args(["-I", "-c", test])
            .env("TEST_HOST", format!("http://{}", gateway.addr))
            .env("TEST_KEY", config::token(dir.path())?)
            .env_remove("OLLAMA_API_KEY")
            .kill_on_drop(true)
            .output(),
    )
    .await??;
    ensure!(
        output.status.success(),
        "Ollama SDK: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
    gateway.stop().await;
    server.kill().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "requires LLAMA_TEST_SERVER and LLAMA_TEST_GGUF"]
async fn local_weights_generate_text_for_each_client_protocol() -> Result<()> {
    let (mut server, addr) = llama::start("smollm2").await?;
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(30))
        .build()?;
    let endpoints = Endpoints {
        chat: format!("http://{addr}/v1"),
        ..Default::default()
    };
    let models = catalog::discover(&endpoints, "").await?;
    ensure!(
        models.iter().any(|m| m == "smollm2"),
        "expected configured llama.cpp model alias"
    );
    let dir = tempfile::tempdir()?;
    let settings = nextjson::json!({"listen":"127.0.0.1:0","providers":[{"id":"llama","models":models,"endpoints":endpoints}]});
    config::save(
        dir.path(),
        &config::snapshot(dir.path())?,
        &settings.to_string(),
    )?;
    let gateway = Service::start(dir.path()).await?;
    let key = config::token(dir.path())?;
    let prompt = "Complete the sentence with one word: The sky is";
    let model = "llama/smollm2";
    for (path, body, text_pointer, tokens_pointer) in [
        (
            "/api/chat",
            json!({"model":model,"messages":[{"role":"user","content":prompt}],"stream":false,"options":{"num_predict":16,"temperature":0}}),
            "/message/content",
            "/eval_count",
        ),
        (
            "/api/generate",
            json!({"model":format!("{model}:latest"),"prompt":prompt,"stream":false,"options":{"num_predict":16,"temperature":0}}),
            "/response",
            "/eval_count",
        ),
        (
            "/v1/chat/completions",
            json!({"model":model,"messages":[{"role":"user","content":prompt}],"max_tokens":16,"temperature":0}),
            "/choices/0/message/content",
            "/usage/completion_tokens",
        ),
        (
            "/v1/responses",
            json!({"model":model,"input":prompt,"max_output_tokens":16,"temperature":0}),
            "/output/0/content/0/text",
            "/usage/output_tokens",
        ),
        (
            "/v1/messages",
            json!({"model":model,"messages":[{"role":"user","content":prompt}],"max_tokens":16,"temperature":0}),
            "/content/0/text",
            "/usage/output_tokens",
        ),
        (
            "/v1beta/models/llama/smollm2:generateContent",
            json!({"contents":[{"role":"user","parts":[{"text":prompt}]}],"generationConfig":{"maxOutputTokens":16,"temperature":0}}),
            "/candidates/0/content/parts/0/text",
            "/usageMetadata/candidatesTokenCount",
        ),
    ] {
        let r = http
            .post(format!("http://{}{path}", gateway.addr))
            .bearer_auth(&key)
            .body(body.to_string())
            .send()
            .await?
            .error_for_status()?;
        let reply: Value = serde_json::from_slice(&r.bytes().await?)?;
        let text = reply
            .pointer(text_pointer)
            .and_then(Value::as_str)
            .context("missing generated text")?;
        let count = reply
            .pointer(tokens_pointer)
            .and_then(Value::as_u64)
            .context("missing output usage")?;
        ensure!(
            !text.trim().is_empty() && (1..=16).contains(&count),
            "{path}: {reply}"
        );
        println!("{path}: {count} output tokens, {text:?}");
    }
    gateway.stop().await;
    server.kill().await?;
    Ok(())
}
