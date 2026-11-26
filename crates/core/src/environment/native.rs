use crate::project_mcp::jsonc;
use anyhow::{Context, Result, ensure};
use nextjson::{
    Value,
    formats::{Format, Json5},
    json,
};
use std::collections::BTreeMap;
use toml_edit::{DocumentMut, Item, Table, TableLike};

use super::Values;

const CODEX: &str = "shell_environment_policy";

pub(super) struct Config<'a> {
    original: Option<&'a str>,
    kind: Kind<'a>,
    pub(super) values: BTreeMap<String, Option<String>>,
}

enum Kind<'a> {
    Claude(&'a str),
    Codex(DocumentMut),
    Pi,
}

impl<'a> Config<'a> {
    pub(super) fn read(agent: &str, original: Option<&'a str>, owned: &Values) -> Result<Self> {
        let (kind, values) = match agent {
            "claude" => {
                let text = original.unwrap_or("{}\n");
                let settings: Value = Json5
                    .decode(text.trim_start_matches('\u{feff}').as_bytes())
                    .map_err(|_| anyhow::anyhow!("invalid Claude JSON settings"))?;
                ensure!(
                    settings.is_object(),
                    "Claude settings must contain an object"
                );
                let env = jsonc::parse(text, &["env"])?;
                let values = env
                    .into_iter()
                    .flat_map(|env| env.entries(text))
                    .map(|(key, source)| (key, Json5.decode(source.as_bytes()).ok()))
                    .collect();
                (Kind::Claude(text), values)
            }
            "codex" => {
                let text = original.unwrap_or_default().trim_start_matches('\u{feff}');
                let document = text
                    .parse::<DocumentMut>()
                    .map_err(|_| anyhow::anyhow!("invalid Codex TOML settings"))?;
                let table = table(document.as_table(), CODEX)?
                    .map(|policy| table(policy, "set"))
                    .transpose()?
                    .flatten();
                let values = table
                    .into_iter()
                    .flat_map(TableLike::iter)
                    .map(|(key, item)| (key.to_owned(), item.as_str().map(str::to_owned)))
                    .collect();
                (Kind::Codex(document), values)
            }
            "pi" => {
                let values = if let Some(text) = original {
                    ensure!(
                        pi_extension(owned)?.as_deref() == Some(text),
                        "existing Pi extension was kept"
                    );
                    owned
                        .iter()
                        .map(|(key, value)| (key.clone(), Some(value.clone())))
                        .collect()
                } else {
                    BTreeMap::new()
                };
                (Kind::Pi, values)
            }
            _ => anyhow::bail!("unsupported environment client"),
        };
        Ok(Self {
            original,
            kind,
            values,
        })
    }

    pub(super) fn render(
        self,
        changes: BTreeMap<String, Option<String>>,
        owned: &Values,
    ) -> Result<Option<String>> {
        match self.kind {
            Kind::Claude(text) => {
                let mut text = text.to_owned();
                for (key, replacement) in changes {
                    let value = replacement.map(Value::from);
                    text = match jsonc::parse(&text, &["env"])? {
                        Some(env) => env.edit(&text, &key, value.as_ref())?,
                        None => jsonc::parse(&text, &[])?
                            .context("missing Claude settings object")?
                            .edit(&text, "env", Some(&json!({(key): value})))?,
                    };
                }
                Ok(Some(text))
            }
            Kind::Codex(mut document) => {
                let policy = ensure_table(document.as_table_mut(), CODEX, false);
                let inline = policy.is_inline_table();
                let policy = policy
                    .as_table_like_mut()
                    .context("Codex policy must be a table")?;
                let env = ensure_table(policy, "set", inline)
                    .as_table_like_mut()
                    .context("Codex environment must be a table")?;
                for (key, replacement) in changes {
                    if let Some(replacement) = replacement {
                        // Use an escaped basic string so CRLF in the document cannot
                        // change a newline inside the environment value.
                        let literal = super::encode(&replacement)?.replace('\u{7f}', "\\u007F");
                        let mut replacement =
                            literal.parse::<toml_edit::Value>().map_err(|_| {
                                anyhow::anyhow!("could not encode Codex environment value")
                            })?;
                        if let Some(previous) = env.get(&key).and_then(Item::as_value) {
                            *replacement.decor_mut() = previous.decor().clone();
                        }
                        env.insert(&key, Item::Value(replacement));
                    } else {
                        env.remove(&key);
                    }
                }
                let text = self.original.unwrap_or_default();
                let mut updated = document.to_string();
                if text.contains("\r\n") && !text.replace("\r\n", "").contains('\n') {
                    updated = updated.replace("\r\n", "\n").replace('\n', "\r\n");
                }
                if text.starts_with('\u{feff}') {
                    updated.insert(0, '\u{feff}');
                }
                Ok(Some(updated))
            }
            Kind::Pi => pi_extension(owned),
        }
    }
}

fn table<'a>(parent: &'a dyn TableLike, key: &str) -> Result<Option<&'a dyn TableLike>> {
    parent
        .get(key)
        .map(|item| {
            item.as_table_like()
                .with_context(|| format!("{key} must be a table"))
        })
        .transpose()
}

fn ensure_table<'a>(parent: &'a mut dyn TableLike, key: &str, inline: bool) -> &'a mut Item {
    parent.entry(key).or_insert_with(|| {
        if inline {
            Item::Value(toml_edit::Value::InlineTable(toml_edit::InlineTable::new()))
        } else {
            Item::Table(Table::new())
        }
    })
}

fn pi_extension(values: &Values) -> Result<Option<String>> {
    if values.is_empty() {
        return Ok(None);
    }
    let values = nextjson::to_string(values).map_err(anyhow::Error::msg)?;
    let values = nextjson::to_string(&values)
        .map_err(anyhow::Error::msg)?
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029");
    Ok(Some(format!(
        "// Managed by Crabforge environment variables\nconst environment = JSON.parse({values});\n\nexport default function () {{\n  for (const [name, value] of Object.entries(environment)) {{\n    if (process.env[name] === undefined) process.env[name] = value;\n  }}\n}}\n"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_crlf_inline_values_round_trip_without_claiming_neighbors() -> Result<()> {
        let original =
            "\u{feff}# user comment\r\n[shell_environment_policy]\r\nset = { KEEP = 'yes' }\r\n";
        let expected =
            "first \"line\"\r\nsecond\nbackslash\\ tab\t U+007F:\u{007f} U+0001:\u{0001}";
        let mut values = Values::new();
        let config = Config::read("codex", Some(original), &values)?;
        values.insert("FIXTURE".into(), expected.into());
        let addition = BTreeMap::from([("FIXTURE".into(), Some(expected.into()))]);
        let updated = config.render(addition, &values)?.context("missing TOML")?;
        let config = Config::read("codex", Some(&updated), &values)?;
        assert_eq!(config.values["FIXTURE"].as_deref(), Some(expected));
        assert_eq!(config.values["KEEP"].as_deref(), Some("yes"));
        let removed = config
            .render(BTreeMap::from([("FIXTURE".into(), None)]), &Values::new())?
            .context("missing TOML")?;
        assert!(!removed.contains("FIXTURE"));
        assert!(removed.starts_with("\u{feff}# user comment\r\n"));
        assert!(!removed.replace("\r\n", "").contains('\n'));
        assert_eq!(
            Config::read("codex", Some(&removed), &Values::new())?
                .values
                .len(),
            1
        );
        Ok(())
    }
}
