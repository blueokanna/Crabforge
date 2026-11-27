use super::{AuthHeader, Config, Endpoints, Provider, catalog_url, read_limited, save, snapshot};
use anyhow::{Context, Result, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use nextjson::{NsonDeserialize, NsonSerialize, Value};
use ring::{aead, pbkdf2};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroU32,
    path::Path,
};
use zeroize::Zeroizing;

const MAX_BACKUP: usize = 64 * 1024 * 1024;

#[derive(NsonDeserialize)]
struct Envelope {
    format: String,
    version: u32,
    kdf: String,
    iterations: u32,
    salt: String,
    nonce: String,
    data: String,
}

impl Envelope {
    fn open(&self, passphrase: &str) -> Result<Zeroizing<Vec<u8>>> {
        ensure!(self.format == "magpie-backup", "expected a Magpie backup");
        ensure!(
            self.version == 1 && self.kdf == "pbkdf2-sha256",
            "unsupported Magpie backup envelope"
        );
        let rounds = NonZeroU32::new(self.iterations)
            .filter(|n| (100_000..=10_000_000).contains(&n.get()))
            .context("invalid Magpie backup iteration count")?;
        let salt = STANDARD.decode(&self.salt).context("invalid backup salt")?;
        ensure!(salt.len() >= 16, "invalid backup salt length");
        let nonce = STANDARD
            .decode(&self.nonce)
            .context("invalid backup nonce")?;
        let nonce = aead::Nonce::try_assume_unique_for_key(&nonce)
            .map_err(|_| anyhow::anyhow!("invalid backup nonce length"))?;
        let mut data = Zeroizing::new(STANDARD.decode(&self.data).context("invalid backup data")?);
        let mut key = Zeroizing::new([0; 32]);
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            rounds,
            &salt,
            passphrase.as_bytes(),
            &mut *key,
        );
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &*key)
            .map_err(|_| anyhow::anyhow!("could not initialize backup decryption"))?;
        let header = format!(
            "{}/{}/{}/{}/{}",
            self.format,
            self.version,
            self.kdf,
            self.iterations,
            hex::encode(&salt)
        );
        let length = aead::LessSafeKey::new(key)
            .open_in_place(nonce, aead::Aad::from(header.as_bytes()), &mut data)
            .map_err(|_| anyhow::anyhow!("wrong passphrase or modified Magpie backup"))?
            .len();
        data.truncate(length);
        Ok(data)
    }
}

#[derive(Default, NsonDeserialize)]
#[njson(default, rename_all = "camelCase")]
struct BackupProvider {
    id: String,
    was: Vec<String>,
    key: String,
    chat: String,
    responses: String,
    anthropic: String,
    gemini: String,
    #[njson(rename = "modelsURL")]
    models_url: String,
    auth_header: AuthHeader,
    models: Vec<String>,
    catalog: String,
    fallback: Vec<String>,
    no_stand_in: bool,
    count_free: bool,
    #[njson(flatten)]
    other: BTreeMap<String, Value>,
}

impl BackupProvider {
    fn convert(mut self, key_env: String) -> Result<Provider> {
        let key = Zeroizing::new(std::mem::take(&mut self.key));
        ensure!(
            key.is_empty() || !key_env.is_empty(),
            "{} has an API key; specify --key-env {}=VARIABLE",
            self.id,
            self.id
        );
        for field in self.other.keys() {
            ensure!(
                matches!(
                    field.as_str(),
                    "name"
                        | "icon"
                        | "iconFrom"
                        | "preset"
                        | "baseAPI"
                        | "geminiFilled"
                        | "keyName"
                        | "website"
                        | "keysUrl"
                ),
                "{} has an unsupported Magpie provider setting {field:?}",
                self.id
            );
        }
        for endpoint in [
            &self.chat,
            &self.responses,
            &self.anthropic,
            &self.gemini,
            &self.models_url,
        ] {
            if !endpoint.is_empty() {
                let url = catalog_url(endpoint).context("invalid Magpie provider endpoint")?;
                ensure!(
                    url.query_pairs().all(|(name, _)| name == "api-version"),
                    "Magpie provider endpoint contains an unsupported query parameter"
                );
            }
        }
        let free_models = if self.count_free {
            self.models.iter().cloned().collect()
        } else {
            BTreeSet::new()
        };
        Ok(Provider {
            id: self.id,
            catalog: self.catalog,
            web_site: None,
            endpoints: Endpoints {
                chat: self.chat,
                responses: self.responses,
                anthropic: self.anthropic,
                gemini: self.gemini,
                models_url: self.models_url,
                auth_header: self.auth_header,
                ..Default::default()
            },
            key_env,
            models: self.models,
            fallback: self.fallback,
            no_stand_in: self.no_stand_in,
            free_models,
            model_protocols: BTreeMap::new(),
        })
    }
}

#[derive(NsonSerialize)]
pub struct MagpieProviderImport {
    providers: Vec<String>,
    other_sections: Vec<String>,
    applied: bool,
}

fn open_backup(file: &Path, passphrase: &str) -> Result<BTreeMap<String, Value>> {
    ensure!(
        !passphrase.is_empty(),
        "a Magpie backup requires its passphrase"
    );
    let bytes = read_limited(file, MAX_BACKUP).context("could not read Magpie backup")?;
    let envelope: Envelope =
        nextjson::from_slice(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(&bytes))
            .map_err(|_| anyhow::anyhow!("invalid Magpie backup envelope"))?;
    let plain = envelope.open(passphrase)?;
    let bundle: BTreeMap<String, Value> = nextjson::from_slice(&plain)
        .map_err(|_| anyhow::anyhow!("invalid decrypted Magpie backup"))?;
    ensure!(
        matches!(
            bundle.get("version").and_then(crate::json::unsigned),
            Some(1 | 2)
        ),
        "unsupported Magpie backup version"
    );
    Ok(bundle)
}

#[derive(NsonSerialize)]
pub struct MagpieGroupImport {
    groups: Vec<String>,
    applied: bool,
}

pub fn import_magpie_groups(
    dir: &Path,
    previous: &str,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    apply: bool,
) -> Result<MagpieGroupImport> {
    let mut bundle = open_backup(file, passphrase)?;
    let Some(Value::Array(groups)) = bundle.remove("groups") else {
        anyhow::bail!("Magpie backup has no group list");
    };
    let order: Vec<String> = bundle
        .remove("groupOrder")
        .map(nextjson::from_value)
        .transpose()
        .map_err(|_| anyhow::anyhow!("invalid Magpie group order"))?
        .unwrap_or_default();
    let mut cfg = Config::parse(previous)?;
    let groups = cfg.restore_magpie_groups(groups, selected, &order)?;
    let text = cfg.formatted()?;
    Config::parse(&text)?;
    if apply {
        save(dir, previous, &text)?;
    }
    Ok(MagpieGroupImport {
        groups,
        applied: apply,
    })
}

pub fn import_magpie_providers(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    selected: &[String],
    key_envs: &BTreeMap<String, String>,
    apply: bool,
) -> Result<MagpieProviderImport> {
    let mut bundle = open_backup(file, passphrase)?;
    let Some(Value::Array(providers)) = bundle.remove("providers") else {
        anyhow::bail!("Magpie backup has no provider list");
    };
    let order: Vec<String> = bundle
        .remove("order")
        .map(nextjson::from_value)
        .transpose()
        .map_err(|_| anyhow::anyhow!("invalid Magpie provider order"))?
        .unwrap_or_default();
    let other_sections = bundle
        .keys()
        .filter(|field| {
            !matches!(
                field.as_str(),
                "version" | "created" | "app" | "keys" | "providersKeys"
            )
        })
        .cloned()
        .collect();
    let previous = snapshot(dir)?;
    let mut cfg = Config::parse(&previous)?;
    let mut occupied: BTreeSet<_> = cfg
        .providers
        .iter()
        .map(|p| p.id.clone())
        .chain(cfg.former_providers.keys().cloned())
        .collect();
    let mut found = BTreeSet::new();
    let mut imported = Vec::new();
    for raw in providers {
        let id = raw
            .get("id")
            .and_then(Value::as_str)
            .context("missing Magpie provider ID")?
            .to_owned();
        if !selected.is_empty() && !selected.contains(&id) {
            continue;
        }
        ensure!(
            super::valid_id(&id) && !matches!(id.as_str(), "auto" | "group"),
            "invalid or reserved Magpie provider ID {id:?}"
        );
        ensure!(
            occupied.insert(id.clone()),
            "provider ID already in use: {id}"
        );
        let mut source: BackupProvider = nextjson::from_value(raw)
            .map_err(|_| anyhow::anyhow!("invalid Magpie provider definition: {id}"))?;
        let former = std::mem::take(&mut source.was);
        let provider = source.convert(key_envs.get(&id).cloned().unwrap_or_default())?;
        for alias in former {
            ensure!(
                super::valid_id(&alias) && !matches!(alias.as_str(), "auto" | "group"),
                "invalid or reserved former Magpie provider ID {alias:?}"
            );
            ensure!(
                occupied.insert(alias.clone()),
                "provider ID already in use: {alias}"
            );
            cfg.former_providers.insert(alias, id.clone());
        }
        found.insert(id);
        imported.push(provider);
    }
    for id in selected.iter().chain(key_envs.keys()) {
        ensure!(
            found.contains(id),
            "provider {id:?} is absent from the selected backup entries"
        );
    }
    ensure!(
        !imported.is_empty(),
        "Magpie backup has no providers to import"
    );
    let ranks: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (id, i)).collect();
    imported.sort_by_key(|p| ranks.get(&p.id).copied().unwrap_or(usize::MAX));
    let providers = imported.iter().map(|p| p.id.clone()).collect();
    cfg.providers.extend(imported);
    let text = cfg.formatted()?;
    let cfg = Config::parse(&text)?;
    let groups = cfg.resolved_groups();
    let known: BTreeSet<_> = cfg
        .providers
        .iter()
        .flat_map(|provider| {
            provider
                .models
                .iter()
                .map(|model| format!("{}/{model}", provider.id))
        })
        .collect();
    for provider in cfg.providers.iter().filter(|p| found.contains(&p.id)) {
        for target in &provider.fallback {
            ensure!(
                target.contains('/')
                    && (known.contains(target)
                        || groups.group_reference(&cfg, target).is_some()
                        || cfg.renamed_model(target).is_some()
                        || cfg.auto_stand_in(target).is_some()),
                "{} references an unavailable fallback {target:?}",
                provider.id
            );
        }
    }
    if apply {
        save(dir, &previous, &text)?;
    }
    Ok(MagpieProviderImport {
        providers,
        other_sections,
        applied: apply,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nextjson::json;
    use std::fs;

    const PASSPHRASE: &str = "fixture only 口令";

    fn seal(file: &Path, payload: Value) {
        let salt = [0x23; 16];
        let nonce = [0x45; 12];
        let mut key = [0; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(),
            &salt,
            PASSPHRASE.as_bytes(),
            &mut key,
        );
        let key = aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM, &key).unwrap());
        let mut bytes = nextjson::to_vec(&payload).unwrap();
        key.seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::from(
                b"magpie-backup/1/pbkdf2-sha256/100000/23232323232323232323232323232323",
            ),
            &mut bytes,
        )
        .unwrap();
        fs::write(file, format!("\u{feff}{}", json!({
            "format": "magpie-backup", "version": 1, "kdf": "pbkdf2-sha256", "iterations": 100_000,
            "salt": STANDARD.encode(salt), "nonce": STANDARD.encode(nonce), "data": STANDARD.encode(bytes),
        }))).unwrap();
    }

    #[test]
    fn provider_backup_preview_and_apply_preserve_ids_and_keep_secrets_out_of_settings()
    -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("test.magpie-backup");
        let settings = dir.path().join("settings.json");
        seal(
            &file,
            json!({
                "version": 2, "order": ["backup", "lab"], "groups": [{"id": "example"}],
                "providers": [
                    {"id":"lab", "was":["previous"], "key":"fixture-api-key", "models":["m"],
                     "anthropic":"http://127.0.0.1:1", "authHeader":"bearer", "noStandIn":true,
                     "fallback":["backup/n"]},
                    {"id":"backup", "key":"", "models":["n"], "chat":"http://127.0.0.1:1/v1",
                     "modelsURL":"http://127.0.0.1:1/v1/models", "countFree":true},
                    {"id":"off", "off":true, "chat":"http://127.0.0.1:1/v1", "models":["m"]}
                ]
            }),
        );
        let selected = ["lab".into(), "backup".into()];
        let keys = BTreeMap::from([("lab".into(), "LAB_API_KEY".into())]);
        let run = |pass, selected: &[String], apply| {
            import_magpie_providers(dir.path(), &file, pass, selected, &keys, apply)
        };
        assert!(run("a different passphrase", &selected, true).is_err());
        assert!(run(PASSPHRASE, &[], true).is_err());
        assert!(run(PASSPHRASE, &selected[..1], true).is_err());
        let preview = run(PASSPHRASE, &selected, false)?;
        assert_eq!(preview.providers, ["backup", "lab"]);
        assert_eq!(preview.other_sections, ["groups"]);
        assert!(!preview.applied);
        assert!(!settings.exists());
        assert!(run(PASSPHRASE, &selected, true)?.applied);
        let before = fs::read_to_string(&settings)?;
        assert!(!before.contains("fixture-api-key"));
        let cfg = Config::parse(&before)?;
        assert_eq!(cfg.renamed_model("previous/m").as_deref(), Some("lab/m"));
        let lab = &cfg.providers[1];
        assert_eq!(lab.id, "lab");
        assert_eq!(lab.key_env, "LAB_API_KEY");
        assert_eq!(lab.endpoints.auth_header, AuthHeader::Bearer);
        assert_eq!(lab.fallback, ["backup/n"]);
        assert!(lab.no_stand_in);
        assert!(cfg.providers[0].free_models.contains("n"));
        assert!(run(PASSPHRASE, &selected, true).is_err());
        let tampered = fs::read_to_string(&file)?.replace("100000", "100001");
        fs::write(&file, tampered)?;
        assert!(run(PASSPHRASE, &selected, true).is_err());
        assert_eq!(fs::read_to_string(&settings)?, before);
        Ok(())
    }

    #[test]
    fn group_backup_preserves_addresses_and_rejects_incomplete_batches() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("groups.magpie-backup");
        let settings = dir.path().join("settings.json");
        let original = r#"{"providers":[{"id":"lab","endpoints":{"chat":"http://127.0.0.1:1"},"models":["m","n"]}]}"#;
        fs::write(&settings, original)?;
        seal(
            &file,
            json!({"version":1, "groupOrder":["team","tools"], "groups":[
                {"id":"tools","name":"Tool models","routing":"weight","members":["lab/m"],
                 "match":["lab/n*"],"placed":["lab/n","lab/m"],"weights":{"lab/n":3},
                 "former":["old-tools"]},
                {"id":"team","name":"Team routes","members":["group/old-tools"],"note":"测试",
                 "disabled":true,"routing":"manual","pick":"group/old-tools"},
                {"id":"blocked","members":["lab/n"],"redact":"on"},
                {"id":"self","members":["group/self"]}
            ]}),
        );
        let previous = snapshot(dir.path())?;
        let import = |ids: &[String], apply| {
            import_magpie_groups(dir.path(), &previous, &file, PASSPHRASE, ids, apply)
        };
        let selected = ["team".into(), "tools".into()];
        assert!(import(&selected[..1], true).is_err());
        assert!(import(&["self".into()], true).is_err());
        assert!(import(&[], true).is_err());
        assert!(!import(&selected, false)?.applied);
        assert_eq!(fs::read_to_string(&settings)?, original);
        let report = import(&selected, true)?;
        assert!(report.applied);
        assert_eq!(report.groups, ["team", "tools"]);
        let cfg = super::super::load(dir.path())?;
        assert_eq!(cfg.group_display_order, report.groups);
        assert_eq!(cfg.former_groups["old-tools"], "tools");
        assert_eq!(cfg.resolved_groups().members["team"], ["group/tools"]);
        assert_eq!(cfg.group_picks["team"], "group/old-tools");
        assert!(cfg.disabled_groups.contains("team"));
        assert_eq!(cfg.group_notes["team"], "测试");
        assert_eq!(cfg.group_placed["tools"], ["lab/n", "lab/m"]);
        assert_eq!(cfg.group_weights["tools"]["lab/n"], 3);
        let stored = fs::read(&settings)?;
        assert!(import(&selected, true).is_err());
        assert_eq!(fs::read(&settings)?, stored);
        Ok(())
    }
}
