use super::{Value, catalog_path};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::ErrorKind, path::Path};

const HISTORY: &str = "crabforge-omp-protocol.json";
pub(super) const AUTO: &str = "auto";
const CHAT: &str = "openai-completions";

#[derive(Serialize, Deserialize)]
pub(super) struct Preference {
    choice: String,
    generated: BTreeSet<String>,
    // An explicit choice supersedes the hand-written value present at that moment.
    // Keep it until the catalog replacement succeeds, including across a restart.
    replacing: Option<String>,
}

impl Preference {
    pub(super) fn load(home: &Path) -> Result<Self> {
        match fs::File::open(home.join(HISTORY)) {
            Ok(file) => {
                Ok(serde_json::from_reader(file).context("invalid Oh My Pi protocol history")?)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(Self {
                choice: AUTO.to_owned(),
                generated: BTreeSet::from([CHAT.to_owned()]),
                replacing: None,
            }),
            Err(e) => Err(e.into()),
        }
    }

    pub(super) fn effective<'a>(&'a self, file: Option<&'a str>) -> &'a str {
        file.filter(|api| {
            !api.is_empty()
                && !self.generated.contains(*api)
                && self.replacing.as_deref() != Some(*api)
        })
        .unwrap_or(&self.choice)
    }

    pub(super) fn select(&mut self, choice: &str, file: Option<&str>) -> Result<()> {
        ensure!(
            matches!(
                choice,
                AUTO | CHAT | "openai-responses" | "anthropic-messages" | "google-generative-ai"
            ),
            "unsupported Oh My Pi protocol {choice:?}"
        );
        self.choice = choice.to_owned();
        self.replacing = file.map(str::to_owned);
        self.generated
            .insert(if choice == AUTO { CHAT } else { choice }.to_owned());
        Ok(())
    }

    pub(super) fn applied(&mut self, home: &Path) -> Result<()> {
        if self.replacing.take().is_some() {
            self.persist(home)?;
        }
        Ok(())
    }

    pub(super) fn persist(&self, home: &Path) -> Result<()> {
        let mut tmp = tempfile::NamedTempFile::new_in(home)?;
        serde_json::to_writer(&mut tmp, self)?;
        tmp.as_file().sync_all()?;
        tmp.persist(home.join(HISTORY))?;
        Ok(())
    }
}

pub(super) fn catalog_api(home: &Path) -> Result<Option<String>> {
    let Some(path) = catalog_path(home) else {
        return Ok(None);
    };
    let doc: Value = serde_yaml::from_reader(fs::File::open(path)?)?;
    Ok(doc
        .get("providers")
        .and_then(|p| p.get("crabforge"))
        .and_then(|p| p.get("api"))
        .and_then(Value::as_str)
        .map(str::to_owned))
}
