use anyhow::{Context, Result, bail, ensure};
use courierust::{
    courierust_client::{Client, ClientConfig},
    courierust_http::Method,
};
use nextjson::{NsonDeserialize, NsonSerialize, Value, json};
use std::{
    fs::File,
    path::Path,
    process::Stdio,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    time::{sleep, timeout},
};
use url::Url;

string_enum! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Site {
        DeepSeek => "deepseek",
        ChatGLM => "chatglm",
    }
}

impl Site {
    pub fn model(self) -> &'static str {
        match self {
            Self::DeepSeek => "deepseek-web",
            Self::ChatGLM => "chatglm-web",
        }
    }
    pub fn url(self) -> &'static str {
        match self {
            Self::DeepSeek => "https://chat.deepseek.com/",
            Self::ChatGLM => "https://chatglm.cn/",
        }
    }
    fn reply(self) -> &'static str {
        match self {
            Self::DeepSeek => ".ds-assistant-message-main-content",
            Self::ChatGLM => ".assistant-message-item .markdown-body",
        }
    }
    // Both selectors require a finished turn in a fresh conversation. Text that
    // has merely stopped growing can still be waiting for another token.
    fn finished(self) -> &'static str {
        match self {
            Self::DeepSeek => {
                ".ds-message:has(> :last-child [role=button]):has(.ds-assistant-message-main-content) .ds-assistant-message-main-content"
            }
            Self::ChatGLM => {
                "body:has(.send-button--disabled):not(:has(.send-button--stop)) .assistant-message-item .markdown-body"
            }
        }
    }
}

const ELEMENT: &str = "element-6066-11e4-a52e-4f735466cecf";
const MAX_REPLY: usize = 2 * 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;
    use crabforge_test_support::server::{Router, get};

    #[tokio::test]
    #[ignore = "requires installed Chrome and a matching ChromeDriver"]
    async fn browser_reads_session_headers_only_from_a_listed_origin() -> Result<()> {
        use crate::{config, service::Service};
        let page = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let origin = format!("http://{}", page.local_addr()?);
        let web = Router::new().route(
            "/",
            get(|_| async {
                (
                    [("content-type", "text/html")],
                    "<!doctype html><title>Crabforge CORS test</title>",
                )
            }),
        );
        let task =
            tokio::spawn(async move { crabforge_test_support::server::serve(page, web).await });
        let root = tempfile::tempdir()?;
        let settings =
            nextjson::to_string(&json!({"listen":"127.0.0.1:0", "cors_origins":[origin]}))
                .map_err(anyhow::Error::msg)?;
        config::save(root.path(), &config::snapshot(root.path())?, &settings)?;
        let allowed = Service::start(root.path()).await?;
        let other = tempfile::tempdir()?;
        config::save(
            other.path(),
            &config::snapshot(other.path())?,
            r#"{"listen":"127.0.0.1:0"}"#,
        )?;
        let denied = Service::start(other.path()).await?;
        let browser = Browser::open(root.path(), Site::DeepSeek).await?;
        browser.navigate(&origin).await?;
        for (svc, key, expected) in [
            (
                &allowed,
                config::token(root.path())?,
                json!([200, "browser-turn", "browser-turn"]),
            ),
            (&allowed, "invalid".into(), json!([401, null, null])),
            (&denied, config::token(other.path())?, json!("blocked")),
        ] {
            let script = "const [url,key,done]=arguments; fetch(url,{headers:{Authorization:'Bearer '+key,'X-Crabforge-Session':'browser-turn'}}).then(r=>done([r.status,r.headers.get('X-Crabforge-Session'),r.headers.get('X-Magpie-Session')])).catch(()=>done('blocked'))";
            let result = browser.command(Method::POST, "/execute/async", Some(json!({"script":script,"args":[format!("http://{}/agent/pi/v1/models",svc.addr),key]}))).await?;
            assert_eq!(result, expected);
        }
        browser.close().await;
        denied.stop().await;
        allowed.stop().await;
        task.abort();
        Ok(())
    }

    #[tokio::test]
    #[ignore = "requires installed Chrome and a matching ChromeDriver"]
    async fn browser_submits_text_and_waits_for_the_completed_turn() -> Result<()> {
        const PAGE: &str = r#"<!doctype html><textarea class="el-textarea__inner" placeholder="Message"></textarea><main></main><script>
        const input=document.querySelector('textarea');input.onkeydown=e=>{if(e.key!=='Enter')return;e.preventDefault();const main=document.querySelector('main');
        main.innerHTML='<div class="ds-message assistant-message-item"><div class="ds-assistant-message-main-content markdown-body">Partial</div><footer></footer></div>';
        setTimeout(()=>{main.querySelector('.markdown-body').textContent=input.value;main.querySelector('footer').innerHTML='<button role="button" class="send-button--disabled">Copy</button>'},700)};
        </script>"#;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let url = format!("http://{}/", listener.local_addr()?);
        let app = Router::new().route(
            "/",
            get(|_| async { ([("content-type", "text/html")], PAGE) }),
        );
        let server =
            tokio::spawn(async move { crabforge_test_support::server::serve(listener, app).await });
        let root = tempfile::tempdir()?;
        for site in [Site::DeepSeek, Site::ChatGLM] {
            let browser = Browser::open(root.path(), site).await?;
            let answer = browser.submit(&url, "line one\n第二行").await;
            browser.close().await;
            assert_eq!(answer?, "line one 第二行");
        }
        server.abort();
        let _ = server.await;
        Ok(())
    }
}

// The child driver accepts only local connections. Its one visible browser
// gets a site-specific profile and never opens the user's existing browser.
pub struct Browser {
    driver: Option<(Child, File)>,
    session: String,
    client: Client,
    site: Site,
}

async fn wire(client: &Client, method: Method, url: &str, body: Option<Value>) -> Result<Value> {
    let client = client.clone();
    let url = url.to_owned();
    timeout(
        Duration::from_secs(30),
        tokio::task::spawn_blocking(move || {
            let mut req = client.request(&url, method);
            if let Some(body) = body {
                req = req
                    .header("content-type", "application/json")
                    .body(nextjson::to_vec(&body).map_err(anyhow::Error::msg)?);
            }
            let response = req.send().context("browser driver is unavailable")?;
            let bytes = response.body.collect_limited(MAX_REPLY)?;
            let reply: Value = nextjson::from_slice(&bytes).map_err(anyhow::Error::msg)?;
            let value = reply
                .into_object()
                .and_then(|mut obj| obj.remove("value"))
                .unwrap_or(Value::Null);
            ensure!(
                response.status.is_success(),
                "WebDriver HTTP {} ({})",
                response.status.as_u16(),
                value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("command failed")
            );
            Ok(value)
        }),
    )
    .await
    .context("browser command timed out")??
}

impl Browser {
    pub async fn open(root: &Path, site: Site) -> Result<Self> {
        let profile = root.join("web-profiles").join(site.model());
        std::fs::create_dir_all(&profile)?;
        let lock = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(profile.join("crabforge.lock"))?;
        lock.try_lock()
            .context("this web chat profile is already in use")?;
        let executable =
            std::env::var_os("CRABFORGE_CHROMEDRIVER").unwrap_or_else(|| "chromedriver".into());
        let mut driver = Command::new(executable)
            .arg("--port=0")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .context("start ChromeDriver (install it on PATH or set CRABFORGE_CHROMEDRIVER)")?;
        let mut lines =
            BufReader::new(driver.stdout.take().context("missing driver output")?).lines();
        let port = timeout(Duration::from_secs(10), async {
            while let Some(line) = lines.next_line().await? {
                if let Some(n) = line
                    .split_once("was started successfully on port ")
                    .and_then(|(_, s)| s.trim_end_matches('.').parse::<u16>().ok())
                {
                    return Ok(n);
                }
            }
            bail!("ChromeDriver exited before listening")
        })
        .await
        .context("ChromeDriver startup timed out")??;
        let client = Client::with_config(ClientConfig {
            max_body: MAX_REPLY,
            max_redirects: 0,
            read_timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(5)),
            ..Default::default()
        });
        let url = format!("http://127.0.0.1:{port}/session");
        let flags = vec![
            format!(
                "--user-data-dir={}",
                std::path::absolute(profile)?.display()
            ),
            "--no-first-run".into(),
            "--no-default-browser-check".into(),
        ];
        let capabilities = json!({"capabilities":{"alwaysMatch":{"browserName":"chrome","goog:chromeOptions":{"args":flags},"pageLoadStrategy":"eager"}}});
        let value = wire(&client, Method::POST, &url, Some(capabilities)).await?;
        let id = value
            .get("sessionId")
            .and_then(Value::as_str)
            .context("missing browser session id")?;
        let browser = Self {
            driver: Some((driver, lock)),
            client,
            session: format!("{url}/{id}"),
            site,
        };
        Ok(browser)
    }

    async fn command(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value> {
        wire(
            &self.client,
            method,
            &format!("{}{path}", self.session),
            body,
        )
        .await
    }
    async fn navigate(&self, url: &str) -> Result<()> {
        self.command(Method::POST, "/url", Some(json!({"url":url})))
            .await?;
        Ok(())
    }
    pub async fn visit(&self) -> Result<()> {
        self.navigate(self.site.url()).await
    }

    async fn elements(&self, css: &str) -> Result<Vec<String>> {
        let values = self
            .command(
                Method::POST,
                "/elements",
                Some(json!({"using":"css selector","value":css})),
            )
            .await?;
        values
            .as_array()
            .context("invalid element list")?
            .iter()
            .map(|e| {
                e.get(ELEMENT)
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .context("invalid element reference")
            })
            .collect()
    }
    async fn check_origin(&self, expected: &Url) -> Result<()> {
        let current = self.command(Method::GET, "/url", None).await?;
        ensure!(
            Url::parse(current.as_str().context("missing page URL")?)?.origin()
                == expected.origin(),
            "web chat redirected; open its sign-in window"
        );
        Ok(())
    }

    async fn submit(&self, url: &str, prompt: &str) -> Result<String> {
        self.navigate(url).await?;
        let expected = Url::parse(url)?;
        let selector = match self.site {
            Site::DeepSeek => "textarea[placeholder]",
            Site::ChatGLM => "textarea.el-textarea__inner",
        };
        let input = timeout(Duration::from_secs(12), async {
            loop {
                self.check_origin(&expected).await?;
                if let Some(el) = self.elements(selector).await?.into_iter().next() {
                    return Ok::<_, anyhow::Error>(el);
                }
                sleep(Duration::from_millis(250)).await;
            }
        })
        .await
        .context(
            "composer unavailable; sign in to the web provider or check its current layout",
        )??;
        ensure!(
            self.elements(self.site.reply()).await?.is_empty(),
            "web provider did not open a new conversation"
        );
        self.command(
            Method::POST,
            &format!("/element/{input}/click"),
            Some(json!({})),
        )
        .await?;
        self.check_origin(&expected).await?;
        self.command(
            Method::POST,
            "/goog/cdp/execute",
            Some(json!({"cmd":"Input.insertText","params":{"text":prompt}})),
        )
        .await?;
        self.command(
            Method::POST,
            &format!("/element/{input}/value"),
            Some(json!({"text":"\u{e007}"})),
        )
        .await?;
        timeout(Duration::from_secs(180), async {
            loop {
                self.check_origin(&expected).await?;
                if let Some(last) = self.elements(self.site.finished()).await?.last() {
                    let value = self
                        .command(Method::GET, &format!("/element/{last}/text"), None)
                        .await?;
                    if let Some(text) = value.as_str().filter(|s| !s.trim().is_empty()) {
                        return Ok(text.to_owned());
                    }
                }
                sleep(Duration::from_millis(400)).await;
            }
        })
        .await
        .context("no completed web chat reply within 180 seconds")?
    }

    pub async fn close(mut self) {
        if let Some(driver) = self.driver.take() {
            close_driver(self.client.clone(), self.session.clone(), driver).await;
        }
    }
}

async fn close_driver(client: Client, session: String, (mut driver, _lock): (Child, File)) {
    let _ = timeout(
        Duration::from_secs(5),
        wire(&client, Method::DELETE, &session, None),
    )
    .await;
    let _ = driver.kill().await;
}

impl Drop for Browser {
    fn drop(&mut self) {
        if let Some(driver) = self.driver.take()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            runtime.spawn(close_driver(
                self.client.clone(),
                self.session.clone(),
                driver,
            ));
        }
    }
}

#[derive(NsonDeserialize)]
#[njson(deny_unknown_fields)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[njson(default)]
    stream: bool,
    #[njson(rename = "stream_options")]
    _stream_options: Option<StreamOptions>,
}

#[derive(NsonDeserialize)]
#[njson(deny_unknown_fields)]
struct StreamOptions {
    #[njson(rename = "include_usage")]
    _include_usage: Option<bool>,
}
#[derive(NsonSerialize, NsonDeserialize)]
#[njson(deny_unknown_fields)]
struct Message {
    role: String,
    content: String,
}

pub struct WebPrompt {
    prompt: String,
    model: String,
    stream: bool,
}
impl WebPrompt {
    pub fn parse(body: &[u8]) -> Result<Self> {
        let request: ChatRequest = nextjson::from_slice(body)
            .map_err(anyhow::Error::msg)
            .context("web chat accepts text messages and the stream flag")?;
        ensure!(
            !request.messages.is_empty()
                && request.messages.last().is_some_and(|m| m.role == "user"),
            "web chat expects a final user message"
        );
        ensure!(
            request.messages.iter().all(|m| matches!(
                m.role.as_str(),
                "user" | "assistant" | "system" | "developer"
            )),
            "unsupported web chat role"
        );
        let prompt = if request.messages.len() == 1 {
            request.messages.into_iter().next().unwrap().content
        } else {
            format!(
                "Continue this conversation and answer the last user message.\n{}",
                nextjson::to_string(&request.messages).map_err(anyhow::Error::msg)?
            )
        };
        ensure!(
            !prompt.trim().is_empty() && prompt.len() <= 128 * 1024,
            "web chat prompt must be 1 to 131072 bytes"
        );
        Ok(Self {
            prompt,
            model: request.model,
            stream: request.stream,
        })
    }
    pub async fn answer(self, root: &Path, site: Site) -> Result<axum_core::response::Response> {
        use axum_core::response::IntoResponse;
        let browser = Browser::open(root, site).await?;
        let answer = browser.submit(site.url(), &self.prompt).await;
        browser.close().await;
        let text = answer?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let id = format!("chatcmpl-web-{}", now.as_nanos());
        let reply = |object: &str, choices: Value| {
            nextjson::to_string(&json!({"id":id,"created":now.as_secs(),"model":self.model,"object":object,"choices":choices})).map_err(anyhow::Error::msg)
        };
        if self.stream {
            let first = reply(
                "chat.completion.chunk",
                json!([{"index":0,"delta":{"role":"assistant","content":text},"finish_reason":null}]),
            )?;
            let last = reply(
                "chat.completion.chunk",
                json!([{"index":0,"delta":{},"finish_reason":"stop"}]),
            )?;
            return Ok((
                [("content-type", "text/event-stream")],
                format!("data: {first}\n\ndata: {last}\n\ndata: [DONE]\n\n"),
            )
                .into_response());
        }
        let body = reply(
            "chat.completion",
            json!([{"index":0,"message":{"role":"assistant","content":text},"finish_reason":"stop"}]),
        )?;
        Ok(([("content-type", "application/json")], body).into_response())
    }
}
