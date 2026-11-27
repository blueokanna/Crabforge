use super::{AuthHeader, Config, Endpoints, Provider, catalog_url, read_limited, save, snapshot};
use anyhow::{Context, Result, bail, ensure};
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

pub(crate) fn library_section(
    file: &Path,
    passphrase: &str,
    section: &str,
) -> Result<(Vec<Value>, Vec<String>)> {
    let mut backup = open_backup(file, passphrase)?;
    let mut library: BTreeMap<String, Value> = nextjson::from_value(
        backup
            .remove("library")
            .filter(|value| !value.is_null())
            .context("Magpie backup has no library section")?,
    )
    .map_err(|_| anyhow::anyhow!("invalid Magpie library section"))?;
    let entries = match library.remove(section) {
        Some(Value::Array(entries)) => entries,
        None | Some(Value::Null) => anyhow::bail!("Magpie library has no {section} list"),
        Some(_) => anyhow::bail!("invalid Magpie library {section} list"),
    };
    Ok((entries, library.into_keys().collect()))
}

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
    decide: String,
    #[njson(rename = "modelsURL")]
    models_url: String,
    auth_header: AuthHeader,
    models: Vec<String>,
    catalog: String,
    contexts: BTreeMap<String, u32>,
    fallback: Vec<String>,
    patience: Option<i32>,
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
            &self.decide,
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
        let (decisions, systemone) =
            if self.other.get("preset").and_then(Value::as_str) == Some("remote-magpie") {
                if let Some(base) = [&self.chat, &self.responses, &self.anthropic, &self.decide]
                    .into_iter()
                    .find(|base| !base.is_empty())
                {
                    let mut url = super::endpoint(base)?;
                    let path = url.path().trim_end_matches('/');
                    if let Some(base) = [
                        "/chat/completions",
                        "/messages/count_tokens",
                        "/messages",
                        "/responses",
                        "/systemone",
                        "/decisions",
                    ]
                    .into_iter()
                    .find_map(|suffix| path.strip_suffix(suffix))
                    {
                        let base = base.to_owned();
                        url.set_path(&base);
                    }
                    (url.to_string(), url.to_string())
                } else {
                    (String::new(), String::new())
                }
            } else if self.decide.is_empty() {
                (String::new(), String::new())
            } else {
                let mut url = super::endpoint(&self.decide)?;
                let path = url.path().trim_end_matches('/');
                if let Some(endpoint) = crate::gateway::vercel_evaluation_base(&self.decide) {
                    (String::new(), endpoint.to_string())
                } else {
                    match url.host_str() {
                        host if host == Some("api.openai.com") || super::azure(&url) => {
                            (self.decide, String::new())
                        }
                        Some("ai-gateway.vercel.sh") => {
                            let base = path.strip_suffix("/systemone").unwrap_or(path);
                            let base = base.strip_suffix("/models").unwrap_or(base);
                            let base = base.strip_suffix("/v1").unwrap_or(base);
                            let path = if base.is_empty() {
                                "/typesafe/v1".to_owned()
                            } else {
                                format!("{base}/v1")
                            };
                            url.set_path(&path);
                            (String::new(), url.to_string())
                        }
                        Some("generativelanguage.googleapis.com") => {
                            url.set_path("/v1beta/decisions");
                            (String::new(), url.to_string())
                        }
                        Some("api.cloudflare.com") => {
                            bail!(
                                "{} needs a vendor-specific Magpie decision adapter",
                                self.id
                            );
                        }
                        _ => {
                            ensure!(
                                !path.contains("/client/v4"),
                                "{} needs a vendor-specific Magpie decision adapter",
                                self.id
                            );
                            if path.is_empty() {
                                url.set_path("/systemone");
                            }
                            (String::new(), url.to_string())
                        }
                    }
                }
            };
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
                decisions,
                systemone,
                models_url: self.models_url,
                auth_header: self.auth_header,
                ..Default::default()
            },
            key_env,
            models: self.models,
            fallback: self.fallback,
            patience: self.patience.unwrap_or_else(super::disabled_patience),
            no_stand_in: self.no_stand_in,
            free_models,
            model_protocols: BTreeMap::new(),
            model_reasoning_levels: BTreeMap::new(),
            contexts: self.contexts,
        })
    }
}

#[derive(NsonSerialize)]
pub struct MagpieProviderImport {
    providers: Vec<String>,
    other_sections: Vec<String>,
    applied: bool,
}

pub(crate) fn open_backup(file: &Path, passphrase: &str) -> Result<BTreeMap<String, Value>> {
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

fn take_entries(bundle: &mut BTreeMap<String, Value>, section: &str) -> Result<Option<Vec<Value>>> {
    match bundle.remove(section) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(entries)) => Ok(Some(entries)),
        Some(_) => anyhow::bail!("invalid Magpie backup {section} list"),
    }
}

fn take_order(bundle: &mut BTreeMap<String, Value>, field: &str) -> Result<Vec<String>> {
    Ok(bundle
        .remove(field)
        .map(nextjson::from_value)
        .transpose()
        .map_err(|_| anyhow::anyhow!("invalid Magpie backup order field {field:?}"))?
        .unwrap_or_default())
}

fn other_sections(bundle: BTreeMap<String, Value>) -> Vec<String> {
    bundle
        .into_keys()
        .filter(|field| {
            !matches!(
                field.as_str(),
                "version" | "created" | "app" | "keys" | "providersKeys"
            )
        })
        .collect()
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
    let groups = take_entries(&mut bundle, "groups")?.context("Magpie backup has no group list")?;
    let order = take_order(&mut bundle, "groupOrder")?;
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
    let providers =
        take_entries(&mut bundle, "providers")?.context("Magpie backup has no provider list")?;
    let order = take_order(&mut bundle, "order")?;
    let previous = snapshot(dir)?;
    let mut cfg = Config::parse(&previous)?;
    let providers = restore_providers(&mut cfg, providers, selected, key_envs, &order)?;
    ensure!(
        !providers.is_empty(),
        "Magpie backup has no providers to import"
    );
    finish_import(dir, &previous, &cfg, &providers, apply)?;
    Ok(MagpieProviderImport {
        providers,
        other_sections: other_sections(bundle),
        applied: apply,
    })
}

#[derive(NsonSerialize)]
pub struct MagpieRoutingImport {
    providers: Vec<String>,
    groups: Vec<String>,
    #[njson(skip_serializing_if = "Option::is_none")]
    settings: Option<MagpieSettingsImport>,
    other_sections: Vec<String>,
    applied: bool,
}

#[derive(NsonDeserialize, NsonSerialize)]
#[njson(rename_all = "camelCase")]
struct BackupSettings {
    #[njson(default, skip_serializing_if = "Option::is_none")]
    port: Option<u16>,
    #[njson(
        default,
        rename = "corsOrigins",
        skip_serializing_if = "Option::is_none"
    )]
    cors_origins: Option<BTreeSet<String>>,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    model_same_as: Option<BTreeMap<String, String>>,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    model_efforts: Option<BTreeMap<String, Vec<String>>>,
    #[njson(
        default,
        rename = "groupFirstToken",
        skip_serializing_if = "Option::is_none"
    )]
    group_first_content: Option<i32>,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    group_patience: Option<i32>,
    #[njson(default, skip_serializing_if = "Option::is_none")]
    classify_timeout: Option<i32>,
    #[njson(flatten, skip_serializing)]
    other: BTreeMap<String, Value>,
}

#[derive(NsonSerialize)]
struct MagpieSettingsImport {
    #[njson(flatten)]
    imported: BackupSettings,
    skipped_fields: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MagpieSettingsMode {
    Skip,
    Include,
    Only,
}

pub struct MagpieImportOptions<'a> {
    pub providers: &'a [String],
    pub groups: &'a [String],
    pub key_envs: &'a BTreeMap<String, String>,
    pub settings: MagpieSettingsMode,
    pub apply: bool,
}

pub fn import_magpie_routing(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    providers: &[String],
    groups: &[String],
    key_envs: &BTreeMap<String, String>,
    apply: bool,
) -> Result<MagpieRoutingImport> {
    import_magpie_backup(
        dir,
        file,
        passphrase,
        MagpieImportOptions {
            providers,
            groups,
            key_envs,
            settings: MagpieSettingsMode::Skip,
            apply,
        },
    )
}

pub fn import_magpie_backup(
    dir: &Path,
    file: &Path,
    passphrase: &str,
    options: MagpieImportOptions<'_>,
) -> Result<MagpieRoutingImport> {
    let mut bundle = open_backup(file, passphrase)?;
    let previous = snapshot(dir)?;
    let mut cfg = Config::parse(&previous)?;
    let settings = if options.settings == MagpieSettingsMode::Skip {
        None
    } else {
        Some(restore_settings(&mut cfg, bundle.remove("settings"))?)
    };
    let (providers, groups) = if options.settings == MagpieSettingsMode::Only {
        ensure!(
            options.providers.is_empty()
                && options.groups.is_empty()
                && options.key_envs.is_empty(),
            "settings-only import cannot select providers or groups"
        );
        bundle.remove("order");
        bundle.remove("groupOrder");
        (Vec::new(), Vec::new())
    } else {
        let definitions = take_entries(&mut bundle, "providers")?.unwrap_or_default();
        let order = take_order(&mut bundle, "order")?;
        let providers = restore_providers(
            &mut cfg,
            definitions,
            options.providers,
            options.key_envs,
            &order,
        )?;
        let definitions = take_entries(&mut bundle, "groups")?.unwrap_or_default();
        let order = take_order(&mut bundle, "groupOrder")?;
        let groups = if definitions.is_empty() && options.groups.is_empty() {
            Vec::new()
        } else {
            cfg.restore_magpie_groups(definitions, options.groups, &order)?
        };
        (providers, groups)
    };
    ensure!(
        !providers.is_empty() || !groups.is_empty() || settings.is_some(),
        "Magpie backup has no providers or groups to import"
    );
    finish_import(dir, &previous, &cfg, &providers, options.apply)?;
    bundle.remove("settingsKeys");
    bundle.remove("libraryKeys");
    Ok(MagpieRoutingImport {
        providers,
        groups,
        settings,
        other_sections: other_sections(bundle),
        applied: options.apply,
    })
}

fn restore_settings(cfg: &mut Config, raw: Option<Value>) -> Result<MagpieSettingsImport> {
    let Some(raw @ Value::Object(_)) = raw else {
        anyhow::bail!("Magpie backup has no settings object");
    };
    let mut imported: BackupSettings =
        nextjson::from_value(raw).map_err(|_| anyhow::anyhow!("invalid Magpie gateway setting"))?;
    ensure!(
        imported.port.is_some()
            || imported.cors_origins.is_some()
            || imported.model_same_as.is_some()
            || imported.model_efforts.is_some()
            || imported.group_first_content.is_some()
            || imported.group_patience.is_some()
            || imported.classify_timeout.is_some(),
        "Magpie backup has no supported settings to import"
    );
    if let Some(port) = &mut imported.port {
        if *port == 0 {
            *port = Config::default().listen.port();
        }
        cfg.listen.set_port(*port);
    }
    if let Some(origins) = &mut imported.cors_origins {
        *origins = super::origins::clean(origins)?;
        cfg.cors_origins.clone_from(origins);
    }
    if let Some(names) = &imported.model_same_as {
        for (id, name) in names {
            ensure!(
                id.split_once('/').is_some_and(|(provider, model)| {
                    super::valid_id(provider)
                        && !matches!(provider, "auto" | "group")
                        && super::valid_model_id(model)
                }) && super::valid_model_id(name),
                "invalid Magpie modelSameAs entry"
            );
            ensure!(
                cfg.model_same_as
                    .get(id)
                    .is_none_or(|current| current == name),
                "conflicting equivalent model for {id:?}"
            );
        }
        cfg.model_same_as.extend(names.clone());
    }
    if let Some(efforts) = &imported.model_efforts {
        for (id, levels) in efforts {
            ensure!(
                super::valid_model_ref(id),
                "invalid Magpie modelEfforts entry {id:?}"
            );
            super::effort::validate_levels(levels, false)?;
            ensure!(
                cfg.model_efforts
                    .get(id)
                    .is_none_or(|saved| saved == levels),
                "conflicting reasoning levels for {id:?}"
            );
        }
        cfg.model_efforts.extend(efforts.clone());
    }
    if let Some(seconds) = imported.group_first_content {
        cfg.default_group_first_content = seconds;
    }
    if let Some(seconds) = imported.group_patience {
        cfg.default_group_patience = seconds;
    }
    if let Some(seconds) = imported.classify_timeout {
        cfg.classify_timeout = seconds;
    }
    let skipped_fields = std::mem::take(&mut imported.other).into_keys().collect();
    Ok(MagpieSettingsImport {
        imported,
        skipped_fields,
    })
}

fn restore_providers(
    cfg: &mut Config,
    providers: Vec<Value>,
    selected: &[String],
    key_envs: &BTreeMap<String, String>,
    order: &[String],
) -> Result<Vec<String>> {
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
    let ranks: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (id, i)).collect();
    imported.sort_by_key(|p| ranks.get(&p.id).copied().unwrap_or(usize::MAX));
    let providers = imported.iter().map(|p| p.id.clone()).collect();
    cfg.providers.extend(imported);
    Ok(providers)
}

fn finish_import(
    dir: &Path,
    previous: &str,
    cfg: &Config,
    providers: &[String],
    apply: bool,
) -> Result<()> {
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
    let imported: BTreeSet<_> = providers.iter().collect();
    for provider in cfg.providers.iter().filter(|p| imported.contains(&p.id)) {
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
        save(dir, previous, &text)?;
    }
    Ok(())
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
                "version": 2, "order": ["backup", "lab", "google", "evaluation"], "groups": [{"id": "example"}],
                "providers": [
                    {"id":"lab", "was":["previous"], "key":"fixture-api-key", "models":["m"],
                     "anthropic":"http://127.0.0.1:1", "authHeader":"bearer", "noStandIn":true,
                     "fallback":["backup/n"], "patience":27},
                    {"id":"backup", "key":"", "models":["n"], "chat":"http://127.0.0.1:1/v1",
                     "modelsURL":"http://127.0.0.1:1/v1/models", "countFree":true,
                     "decide":"http://127.0.0.1:1/v1"},
                     {"id":"off", "off":true, "chat":"http://127.0.0.1:1/v1", "models":["m"]},
                     {"id":"google", "models":["gemini-decision-1"], "decide":"https://generativelanguage.googleapis.com/v1beta"},
                     {"id":"evaluation", "models":["typesafe-ai/jev"], "decide":"https://ai-gateway.vercel.sh/v4/ai"}
                ]
            }),
        );
        let selected = [
            "lab".into(),
            "backup".into(),
            "google".into(),
            "evaluation".into(),
        ];
        let keys = BTreeMap::from([("lab".into(), "LAB_API_KEY".into())]);
        let run = |pass, selected: &[String], apply| {
            import_magpie_providers(dir.path(), &file, pass, selected, &keys, apply)
        };
        assert!(run("a different passphrase", &selected, true).is_err());
        assert!(run(PASSPHRASE, &[], true).is_err());
        assert!(run(PASSPHRASE, &selected[..1], true).is_err());
        let preview = run(PASSPHRASE, &selected, false)?;
        assert_eq!(preview.providers, ["backup", "lab", "google", "evaluation"]);
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
        assert_eq!(
            cfg.providers[0].endpoints.systemone,
            "http://127.0.0.1:1/v1"
        );
        assert_eq!(
            cfg.providers[2].endpoints.systemone,
            "https://generativelanguage.googleapis.com/v1beta/decisions"
        );
        assert_eq!(
            cfg.providers[3].endpoints.systemone,
            "https://ai-gateway.vercel.sh/v4/ai/evaluation-model"
        );
        assert!(run(PASSPHRASE, &selected, true).is_err());
        let tampered = fs::read_to_string(&file)?.replace("100000", "100001");
        fs::write(&file, tampered)?;
        assert!(run(PASSPHRASE, &selected, true).is_err());
        assert_eq!(lab.patience, 27);
        assert_eq!(cfg.providers[0].patience, -1);
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

    #[test]
    fn routing_backup_validates_cross_references_before_writing() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("routing.magpie-backup");
        let settings = dir.path().join("settings.json");
        let original = r#"{"listen":"127.0.0.1:0"}"#;
        fs::write(&settings, original)?;
        seal(
            &file,
            json!({"version":2, "order":["backup","lab"], "groupOrder":["tools","work"],
                "settingsKeys":false, "settings":{"listen":"127.0.0.1:1"}, "usage":{},
                "providers":[
                    {"id":"lab","chat":"http://127.0.0.1:1","models":["m"],
                     "key":"fixture-only","fallback":["group/old-work"]},
                    {"id":"backup","was":["old-backup"],"chat":"http://127.0.0.1:1","models":["m"]}
                ],
                "groups":[
                    {"id":"work","former":["old-work"],"members":["group/old-tools"],"firstToken":15,"patience":30,
                     "rules":[{"use":"group/tools","where":[{"field":"message","op":"contains","value":"review"}]}]},
                    {"id":"tools","former":["old-tools"],"members":["backup/m"],"firstToken":-1,"patience":-1},
                    {"id":"unsupported","members":["lab/m"],"redact":"on"}
                ]
            }),
        );
        let keys = BTreeMap::from([("lab".into(), "LAB_API_KEY".into())]);
        let groups = ["work".into(), "tools".into()];
        let restore = |providers: &[String], groups: &[String], apply| {
            import_magpie_routing(
                dir.path(),
                &file,
                PASSPHRASE,
                providers,
                groups,
                &keys,
                apply,
            )
        };
        assert!(import_magpie_providers(dir.path(), &file, PASSPHRASE, &[], &keys, true).is_err());
        assert!(restore(&[], &groups[1..], true).is_err());
        assert!(restore(&["lab".into()], &groups, true).is_err());
        assert!(restore(&[], &[], true).is_err());
        let preview = restore(&[], &groups, false)?;
        assert!(!preview.applied);
        assert_eq!(preview.providers, ["backup", "lab"]);
        assert_eq!(preview.groups, ["tools", "work"]);
        assert_eq!(preview.other_sections, ["settings", "usage"]);
        assert_eq!(fs::read_to_string(&settings)?, original);
        assert!(restore(&[], &groups, true)?.applied);
        let stored = fs::read_to_string(&settings)?;
        let cfg = Config::parse(&stored)?;
        assert_eq!(cfg.listen.port(), 0);
        assert_eq!(cfg.providers[1].fallback, ["group/old-work"]);
        assert_eq!(cfg.providers[1].key_env, "LAB_API_KEY");
        assert_eq!(
            cfg.renamed_model("old-backup/m").as_deref(),
            Some("backup/m")
        );
        assert_eq!(cfg.group_display_order, ["auto-m", "tools", "work"]);
        assert_eq!(cfg.group_first_content["work"], 15);
        assert_eq!(cfg.group_first_content["tools"], -1);
        assert_eq!(cfg.group_patience["work"], 30);
        assert_eq!(cfg.group_patience["tools"], -1);
        let share = cfg.share_group("work")?;
        let (restored, group) = cfg.import_group(&share, false)?;
        assert_eq!(restored.group_first_content[&group.id], 15);
        assert_eq!(restored.group_patience[&group.id], 30);
        assert_eq!(restored.group_rules[&group.id][0].target, "group/tools");
        let renamed = cfg.renamed_group("tools", "build")?;
        assert_eq!(renamed.group_rules["work"][0].target, "group/build");
        let (copied, name) = renamed.copied_group("work", "Rules example")?;
        assert_eq!(copied.group_rules[&name][0].conditions[0].value, "review");
        assert!(!stored.contains("fixture-only"));
        assert!(restore(&[], &groups, true).is_err());
        assert_eq!(fs::read_to_string(&settings)?, stored);
        Ok(())
    }

    #[test]
    fn settings_backup_preserves_the_loopback_host_and_skips_unselected_sections() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("settings.magpie-backup");
        let settings = dir.path().join("settings.json");
        let original = r#"{"listen":"[::1]:32122","model_same_as":{"old/model":"old-name"}}"#;
        fs::write(&settings, original)?;
        let restore = |apply| {
            import_magpie_backup(
                dir.path(),
                &file,
                PASSPHRASE,
                MagpieImportOptions {
                    providers: &[],
                    groups: &[],
                    key_envs: &BTreeMap::new(),
                    settings: MagpieSettingsMode::Only,
                    apply,
                },
            )
        };
        for invalid in [
            json!({"port":65536}),
            json!({"port":-1}),
            json!({"port":3425, "corsOrigins":["*"]}),
            json!({"port":3425, "modelSameAs":{"old/model":"conflicting-name"}}),
            json!({"modelSameAs":{"missing-provider-prefix":"target"}}),
            json!({"groupFirstToken":-2}),
            json!({"groupPatience":241}),
            json!({"lanKey":"fixture-secret"}),
        ] {
            seal(&file, json!({"version":1, "settings":invalid}));
            assert!(restore(true).is_err());
            assert_eq!(fs::read_to_string(&settings)?, original);
        }
        seal(
            &file,
            json!({"version":1, "providers":[{"id":"unsupported"}], "groups":[{}],
                "settings":{"port":0,"lan":true,"lanKey":"fixture-secret",
                    "corsOrigins":["HTTPS://Example.org:443/"],
                    "modelSameAs":{"lab/vendor/m":"canonical"}, "groupFirstToken":25,"groupPatience":0}}),
        );
        let preview = restore(false)?;
        let preview = nextjson::to_value(&preview).unwrap();
        assert_eq!(
            preview,
            json!({"providers":[], "groups":[], "settings":{"port":3425,
                 "corsOrigins":["https://example.org"], "modelSameAs":{"lab/vendor/m":"canonical"}, "groupFirstToken":25,"groupPatience":0,
                "skipped_fields":["lan","lanKey"]}, "other_sections":["groups","providers"],
                "applied":false})
        );
        assert_eq!(fs::read_to_string(&settings)?, original);
        assert!(restore(true)?.applied);
        let stored = fs::read_to_string(&settings)?;
        let cfg = Config::parse(&stored)?;
        assert_eq!(cfg.listen.to_string(), "[::1]:3425");
        assert_eq!(
            cfg.cors_origins,
            BTreeSet::from(["https://example.org".into()])
        );
        assert_eq!(cfg.model_same_as["lab/vendor/m"], "canonical");
        assert_eq!(cfg.model_same_as["old/model"], "old-name");
        assert_eq!(cfg.default_group_first_content, 25);
        assert_eq!(cfg.default_group_patience, 0);
        assert!(!stored.contains("fixture-secret"));
        Ok(())
    }

    #[test]
    fn settings_and_routes_resolve_equivalent_models_in_the_same_import() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file = dir.path().join("routes.magpie-backup");
        seal(
            &file,
            json!({"version":2, "settings":{"modelSameAs":{"lab/wire-name":"canonical"}},
                "providers":[
                    {"id":"lab","chat":"http://127.0.0.1:1","models":["wire-name"]},
                    {"id":"direct","chat":"http://127.0.0.1:1","models":["m"],
                     "fallback":["auto/canonical"]}
                ], "groups":[{"id":"work","members":["lab/wire-name"]}]}),
        );
        let apply = |settings| {
            import_magpie_backup(
                dir.path(),
                &file,
                PASSPHRASE,
                MagpieImportOptions {
                    providers: &[],
                    groups: &[],
                    key_envs: &BTreeMap::new(),
                    settings,
                    apply: true,
                },
            )
        };
        assert!(apply(MagpieSettingsMode::Skip).is_err());
        assert!(!dir.path().join("settings.json").exists());
        let report = apply(MagpieSettingsMode::Include)?;
        assert!(report.applied && report.other_sections.is_empty());
        assert_eq!(report.providers, ["lab", "direct"]);
        assert_eq!(report.groups, ["work"]);
        let cfg = super::super::load(dir.path())?;
        assert_eq!(
            cfg.auto_stand_in("auto/canonical").as_deref(),
            Some("lab/wire-name")
        );
        Ok(())
    }

    #[test]
    fn memory_backup_keeps_states_and_evidence_and_refuses_file_conflicts() -> Result<()> {
        use crate::memory;

        let dir = tempfile::tempdir()?;
        let file = dir.path().join("memory.magpie-backup");
        let store = dir.path().join("crabforge");
        let stamp = "2026-11-26T04:09:42.123456789Z";
        let sample = |id, status| {
            json!({
                "id":id,"text":"记住这个项目的发行流程。\n\n保留证据。","kind":"procedure",
                "scope":"project:example/app","origin":"inferred","status":status,
                "created":stamp,"updated":stamp,"tags":["Windows","release"], "pinned":true,
                "sources":[{"agent":"pi","session":"original","quote":"保留证据。","at":stamp}]
            })
        };
        let backup = |items: Vec<Value>| {
            json!({"version":2,"memory":{
                "memories":items,"removed":{"m_gone":stamp},
                "projects":{"example/app":{"key":"example/app","name":"app"}},
                "dismissed":["other fixture memory"],"declined":["fixture suggestion"]
            }})
        };
        let original = vec![
            sample("m_claimed", "candidate"),
            sample("m_gone", "archived"),
        ];
        seal(&file, backup(original.clone()));
        let report = |apply| -> Result<Value> {
            let report = memory::import_magpie(&store, &file, PASSPHRASE, apply)?;
            nextjson::to_value(&report).map_err(anyhow::Error::msg)
        };
        assert!(memory::import_magpie(&store, &file, "wrong passphrase", true).is_err());
        assert_eq!(
            report(false)?,
            json!({
                "memories":[{"id":"m_claimed","status":"candidate","scope":"project:example/app","action":"create"}],
                "removed":["m_gone"],"skipped_fields":["declined","dismissed","projects"],"applied":false
            })
        );
        assert!(!store.exists());
        assert_eq!(report(true)?["applied"], json!(true));
        let saved = memory::read(&store, "m_claimed")?;
        assert_eq!(saved.status, "candidate");
        assert_eq!(saved.updated, stamp);
        assert_eq!(saved.source_quote.as_deref(), Some("保留证据。"));
        assert_eq!(saved.tags, ["release", "windows"]);
        assert!(saved.pinned);
        assert_eq!(report(true)?["memories"][0]["action"], json!("identical"));
        assert!(memory::read(&store, "m_gone").is_err());

        let mut items = vec![
            sample("m_second", "active"),
            sample("m_claimed", "archived"),
        ];
        seal(&file, backup(items.clone()));
        assert_eq!(report(false)?["memories"][1]["action"], json!("conflict"));
        assert!(report(true).is_err());
        assert_eq!(memory::read(&store, "m_claimed")?.revision, saved.revision);
        assert!(memory::read(&store, "m_second").is_err());

        items[1] = sample("../outside", "active");
        seal(&file, backup(items));
        assert!(report(true).is_err());
        assert!(memory::read(&store, "m_second").is_err());
        Ok(())
    }

    #[test]
    fn resource_backup_import_is_read_only_until_apply_and_checks_the_whole_batch() -> Result<()> {
        use crate::agent_files::{Kind, import_magpie};

        let dir = tempfile::tempdir()?;
        let file = dir.path().join("resources.magpie-backup");
        let store = dir.path().join("crabforge");
        let command = json!({"name":"inspect","description":"Inspect","prompt":"Read $1","agents":["claude"]});
        let subagent = json!({"name":"reviewer","description":"Review","prompt":"Check the report.",
            "agents":["codex"],"effort":"high","fromHash":"fixture"});
        seal(
            &file,
            json!({"version":2,"library":{"commands":[command],"subagents":[subagent]}}),
        );
        let report = |kind, apply| -> Result<Value> {
            let report = import_magpie(&store, kind, &file, PASSPHRASE, &[], apply)?;
            nextjson::to_value(&report).map_err(anyhow::Error::msg)
        };
        assert_eq!(
            report(Kind::Command, false)?["resources"][0]["action"],
            json!("create")
        );
        let subagents = report(Kind::Subagent, false)?;
        assert_eq!(
            subagents["resources"][0]["skipped_fields"],
            json!(["fromHash"])
        );
        assert_eq!(subagents["other_library_sections"], json!(["commands"]));
        assert!(!store.exists());
        report(Kind::Command, true)?;
        report(Kind::Subagent, true)?;
        let library = store.join("commands/library.json");
        let before = std::fs::read(&library)?;
        assert_eq!(
            report(Kind::Command, true)?["resources"][0]["action"],
            json!("identical")
        );

        seal(
            &file,
            json!({"version":2,"library":{"commands":[
                {"name":"ahead","description":"New","prompt":"Keep me pending","agents":null},
                {"name":"inspect","description":"Conflict","prompt":"Different text","agents":null},
                {"name":"unsupported","description":"Unsupported","prompt":"Read $1","agents":["gemini"]}
            ]}}),
        );
        assert!(report(Kind::Command, true).is_err());
        let selected = ["ahead".to_owned(), "inspect".to_owned()];
        let preview = import_magpie(&store, Kind::Command, &file, PASSPHRASE, &selected, false)?;
        let preview = nextjson::to_value(&preview).map_err(anyhow::Error::msg)?;
        assert_eq!(preview["resources"][0]["action"], json!("create"));
        assert_eq!(preview["resources"][1]["action"], json!("conflict"));
        assert!(import_magpie(&store, Kind::Command, &file, PASSPHRASE, &selected, true).is_err());
        assert_eq!(std::fs::read(&library)?, before);
        Ok(())
    }

    #[test]
    fn environment_backup_masks_values_and_rejects_a_conflicting_batch() -> Result<()> {
        use crate::environment::import_magpie;

        let dir = tempfile::tempdir()?;
        let file = dir.path().join("environment.magpie-backup");
        let store = dir.path().join("crabforge");
        let value = "fixture-value-原文\nsecond line";
        let variable = json!({"name":"EXISTING","value":value,"agents":["claude","pi"]});
        seal(&file, json!({"version":2,"library":{"env":[variable]}}));
        let report = |apply| -> Result<Value> {
            let report = import_magpie(&store, &file, PASSPHRASE, &[], apply)?;
            nextjson::to_value(&report).map_err(anyhow::Error::msg)
        };
        let preview = report(false)?;
        assert_eq!(preview["variables"][0]["has_value"], json!(true));
        assert!(preview.pointer("/variables/0/value").is_none());
        assert!(!store.exists());

        let applied = report(true)?;
        assert!(applied.pointer("/variables/0/value").is_none());
        let library = store.join("environment/library.json");
        let before = std::fs::read(&library)?;
        let stored: Value = nextjson::from_slice(&before).map_err(anyhow::Error::msg)?;
        assert_eq!(stored["variables"]["EXISTING"]["value"], json!(value));
        assert_eq!(report(true)?["variables"][0]["action"], json!("identical"));

        seal(
            &file,
            json!({"version":2,"library":{"env":[
                {"name":"AHEAD","value":"","agents":null},
                {"name":"EXISTING","value":"different","agents":["claude","pi"]}
            ]}}),
        );
        let preview = report(false)?;
        assert_eq!(preview["variables"][0]["action"], json!("create"));
        assert_eq!(preview["variables"][0]["has_value"], json!(false));
        assert_eq!(preview["variables"][1]["action"], json!("conflict"));
        assert!(report(true).is_err());
        assert_eq!(std::fs::read(library)?, before);
        Ok(())
    }
}
