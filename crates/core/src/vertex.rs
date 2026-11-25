use anyhow::{Context, Result, ensure};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::path::Path;

pub(crate) mod auth;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Vertex {
    pub project: String,
    pub location: String,
    pub credentials: String,
    pub impersonate: String,
    #[serde(alias = "baseUrl")]
    pub base_url: String,
}

impl Vertex {
    pub(crate) fn location(&self) -> &str {
        if self.location.is_empty() {
            "global"
        } else {
            &self.location
        }
    }

    pub(crate) fn validate(&self, keyed: bool, models_url: &str) -> Result<()> {
        ensure!(
            keyed || !self.project.is_empty(),
            "Vertex requires a project ID for ADC"
        );
        if !self.project.is_empty() {
            let valid = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
            let id = self.project.as_bytes();
            ensure!(
                id.len() >= 2
                    && valid(id[0])
                    && valid(id[id.len() - 1])
                    && id.iter().all(|&b| valid(b) || b"-.:".contains(&b)),
                "invalid Vertex project ID"
            );
        }
        let mut parts = self.location().split('-');
        ensure!(
            parts
                .next()
                .unwrap()
                .bytes()
                .all(|b| b.is_ascii_lowercase())
                && !self.location().starts_with('-')
                && parts.all(|s| !s.is_empty()
                    && s.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())),
            "invalid Vertex location"
        );
        ensure!(
            self.credentials.is_empty()
                || Path::new(&self.credentials).is_absolute()
                || Path::new(&self.credentials).starts_with("~"),
            "Vertex credentials require an absolute path or ~/path"
        );
        ensure!(
            self.impersonate.is_empty() || account(&self.impersonate),
            "invalid Vertex service account email"
        );
        if !self.base_url.is_empty() {
            ensure!(keyed, "Vertex ADC requires the Google endpoint");
            crate::config::endpoint(&self.base_url)?;
        }
        if !keyed && !models_url.is_empty() {
            let base = self.base()?;
            let catalog = crate::config::catalog_url(models_url)?;
            ensure!(
                catalog.origin() == base.origin()
                    && catalog
                        .path()
                        .strip_prefix(base.path())
                        .is_some_and(|rest| rest.starts_with('/')),
                "a Vertex ADC catalog must be under the project's Google endpoint"
            );
        }
        Ok(())
    }

    pub(crate) fn base(&self) -> Result<Url> {
        let loc = self.location();
        let mut url = if self.base_url.is_empty() {
            let host = match loc {
                _ if self.project.is_empty() => "aiplatform.googleapis.com".to_owned(),
                "global" => "aiplatform.googleapis.com".to_owned(),
                "us" | "eu" => format!("aiplatform.{loc}.rep.googleapis.com"),
                _ => format!("{loc}-aiplatform.googleapis.com"),
            };
            Url::parse(&format!("https://{host}"))?
        } else {
            crate::config::endpoint(&self.base_url)?
        };
        let path = url.path().trim_end_matches('/');
        let path = path
            .strip_suffix("/v1beta1")
            .or_else(|| path.strip_suffix("/v1"))
            .unwrap_or(path)
            .to_string();
        url.set_path(&path);
        let mut seg = url
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("invalid Vertex base"))?;
        seg.pop_if_empty().push("v1");
        if !self.project.is_empty() {
            seg.extend(["projects", &self.project, "locations", loc]);
        }
        drop(seg);
        Ok(url)
    }
}

fn account(s: &str) -> bool {
    s.split_once('@')
        .is_some_and(|(lhs, rhs)| !lhs.is_empty() && !rhs.is_empty() && !rhs.contains('@'))
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"@._+-".contains(&b))
}

pub(crate) fn claude_body(url: &mut Url, model: &str, count: bool, data: &[u8]) -> Result<Vec<u8>> {
    let mut obj: Map<String, Value> =
        serde_json::from_slice(data).context("invalid Vertex Messages request")?;
    obj.entry("anthropic_version")
        .or_insert_with(|| json!("vertex-2023-10-16"));
    if count {
        obj.insert("model".into(), json!(model));
    } else {
        obj.remove("model");
        if obj.get("stream").and_then(Value::as_bool) == Some(true) {
            url.path_segments_mut()
                .map_err(|_| anyhow::anyhow!("invalid Vertex URL"))?
                .pop()
                .push(&format!("{model}:streamRawPredict"));
        }
    }
    Ok(serde_json::to_vec(&obj)?)
}
