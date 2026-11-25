use flate2::read::GzDecoder;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    env,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

#[path = "src/model_name.rs"]
mod model_name;
use model_name::same_model;

#[derive(Deserialize)]
struct Catalog {
    models: BTreeMap<String, Model>,
}
#[derive(Deserialize)]
struct Model {
    canonical_model_id: Option<String>,
    provider: Option<ModelProvider>,
}

#[derive(Deserialize)]
struct ModelProvider {
    npm: Option<String>,
}

fn bare(id: &str) -> String {
    id.rsplit('/').next().unwrap().to_lowercase()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const SOURCE: &str = "data/models.dev.json.gz";
    println!("cargo:rerun-if-changed={SOURCE}");
    println!("cargo:rerun-if-changed=src/model_name.rs");
    let catalogs: BTreeMap<String, Catalog> =
        serde_json::from_reader(GzDecoder::new(File::open(SOURCE)?))?;
    let mut votes = BTreeMap::new();
    let mut canonical = BTreeMap::new();
    for catalog in catalogs.values() {
        for (name, model) in &catalog.models {
            *votes.entry(bare(name)).or_insert(0usize) += 1;
            if let Some(id) = &model.canonical_model_id
                && id.contains('/')
            {
                *canonical.entry(bare(id)).or_insert(0usize) += 1;
            }
        }
    }
    for (id, count) in canonical {
        votes.entry(id).or_insert(count);
    }
    let mut spellings = BTreeMap::new();
    for (id, count) in &votes {
        if matches!(id.as_str(), "auto" | "default" | "free") {
            continue;
        }
        // BTreeMap iteration resolves tied votes by lexical order.
        let best = spellings.entry(same_model(id)).or_insert((id, count));
        if count > best.1 {
            *best = (id, count)
        }
    }
    let mut aliases = BTreeMap::new();
    for (provider, catalog) in &catalogs {
        for (id, model) in &catalog.models {
            let Some(canonical) = model
                .canonical_model_id
                .as_ref()
                .filter(|s| s.contains('/'))
            else {
                continue;
            };
            let name = same_model(id);
            let canonical = same_model(canonical);
            for (idx, _) in canonical.match_indices('-') {
                let (maker, rest) = (&canonical[..idx], &canonical[idx + 1..]);
                if !maker.bytes().all(|c| c.is_ascii_lowercase() || c == b'-') {
                    break;
                }
                let full = if rest == name {
                    canonical.clone()
                } else if name
                    .strip_prefix(rest)
                    .is_some_and(|tail| tail.starts_with('-'))
                {
                    let candidate = format!("{maker}-{name}");
                    if !spellings.contains_key(&candidate) {
                        continue;
                    }
                    candidate
                } else {
                    continue;
                };
                let canonical_name = spellings.get(&full).map(|v| v.0).unwrap_or(&full);
                let entry = aliases
                    .entry((provider, name.clone()))
                    .or_insert_with(|| canonical_name.clone());
                if canonical_name < entry {
                    entry.clone_from(canonical_name)
                }
                break;
            }
        }
    }
    let path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("catalog_names.rs");
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "const SPELLINGS: &[(&str, &str)] = &[")?;
    for (key, (name, _)) in spellings {
        writeln!(out, "({key:?}, {name:?}),")?
    }
    writeln!(out, "];\nconst UNPREFIXED: &[((&str, &str), &str)] = &[")?;
    for ((provider, key), name) in aliases {
        writeln!(out, "(({provider:?}, {key:?}), {name:?}),")?
    }
    writeln!(out, "];")?;
    out.flush()?;
    let path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("opencode_models.rs");
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "const MODELS: &[((&str, &str), Format)] = &[")?;
    for plan in ["opencode", "opencode-go"] {
        for (model, info) in &catalogs[plan].models {
            let format = match info.provider.as_ref().and_then(|p| p.npm.as_deref()) {
                Some("@ai-sdk/openai") => "Responses",
                Some("@ai-sdk/anthropic") => "Messages",
                Some("@ai-sdk/google") => "Gemini",
                None | Some("@ai-sdk/openai-compatible") => "Chat",
                _ => continue,
            };
            writeln!(out, "(({plan:?}, {model:?}), Format::{format}),")?;
        }
    }
    writeln!(out, "];")?;
    out.flush()?;
    Ok(())
}
