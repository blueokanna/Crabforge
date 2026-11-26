use super::{Config, merge_name};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct SharedGroup {
    id: String,
    name: String,
    #[serde(deserialize_with = "nullable_members")]
    members: Vec<String>,
    #[serde(rename = "match", skip_serializing_if = "Vec::is_empty")]
    patterns: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    placed: Vec<String>,
    #[serde(skip_serializing)]
    matched: Vec<String>,
    routing: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    note: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pick: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    former: Vec<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    off: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    weights: BTreeMap<String, u32>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    unlists: bool,
    #[serde(flatten, skip_serializing)]
    other: BTreeMap<String, Value>,
}

fn nullable_members<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(Option::<Vec<String>>::deserialize(d)?.unwrap_or_default())
}

#[derive(Default, Serialize)]
pub struct ImportedGroup {
    pub id: String,
    pub name: String,
    pub missing: Vec<String>,
    pub instead: Vec<String>,
    pub replaced: bool,
}

// Account fingerprints in older Magpie shares belong only to their sender.
fn without_account(id: &str) -> &str {
    id.rsplit_once('#')
        .filter(|(m, a)| !m.is_empty() && !a.trim().is_empty() && !a.contains('/'))
        .map_or(id, |(m, _)| m)
}

fn shared_text(text: &str) -> Result<SharedGroup> {
    ensure!(
        text.len() <= crate::config::MAX_CONFIG,
        "shared group exceeds 1 MiB"
    );
    let raw = text
        .find('{')
        .zip(text.rfind('}'))
        .and_then(|(a, b)| text.get(a..=b))
        .context("expected a shared routing group")?;
    let doc: Value = serde_json::from_str(raw)?;
    ensure!(
        doc["magpie"] == "routing-group",
        "expected a Magpie routing-group share"
    );
    ensure!(
        doc["version"] == 1,
        "unsupported routing-group share version {}",
        doc["version"]
    );
    let group: SharedGroup = serde_json::from_value(doc["group"].clone())?;
    for field in group.other.keys() {
        // These fields are local, derived, or an edit timestamp. None is transferred by import.
        ensure!(
            matches!(
                field.as_str(),
                "keys"
                    | "accounts"
                    | "redact"
                    | "seen"
                    | "auto"
                    | "hidden"
                    | "disabled"
                    | "matched"
                    | "routedAt"
            ),
            "unsupported shared group setting: {field}"
        );
    }
    ensure!(
        matches!(
            group.routing.as_str(),
            "" | "order" | "manual" | "rotate" | "weight"
        ),
        "unsupported shared routing strategy: {}",
        group.routing
    );
    Ok(group)
}

impl Config {
    pub fn share_group(&self, reference: &str) -> Result<String> {
        let groups = self.resolved_groups();
        let id = self.group_name(reference);
        let members = self
            .groups
            .get(&id)
            .or_else(|| groups.members.get(&id))
            .context("unknown route group")?;
        let pick = self
            .group_picks
            .get(&id)
            .or_else(|| self.group_last_picks.get(&id))
            .filter(|p| groups.display_members(self, &id).contains(p));
        let routing = if self.group_picks.contains_key(&id) {
            "manual"
        } else if self.group_weights.contains_key(&id) {
            "weight"
        } else if self.rotating_groups.contains(&id) {
            "rotate"
        } else {
            "order"
        };
        let group = SharedGroup {
            name: self.group_labels.get(&id).unwrap_or(&id).clone(),
            members: members.clone(),
            patterns: self
                .group_match
                .get(&id)
                .into_iter()
                .flatten()
                .map(|p| p.as_str().to_owned())
                .collect(),
            placed: self.group_placed.get(&id).cloned().unwrap_or_default(),
            note: self.group_notes.get(&id).cloned().unwrap_or_default(),
            former: self
                .former_groups
                .iter()
                .filter(|(_, to)| *to == &id)
                .map(|(old, _)| old.clone())
                .collect(),
            off: self.group_off.get(&id).cloned().unwrap_or_default(),
            weights: self
                .group_weights
                .get(&id)
                .or_else(|| self.group_last_weights.get(&id))
                .cloned()
                .unwrap_or_default(),
            pick: pick.cloned().unwrap_or_default(),
            routing: routing.into(),
            unlists: self.group_unlists.contains(&id),
            id,
            ..Default::default()
        };
        ensure!(
            group
                .members
                .iter()
                .chain(&group.placed)
                .chain(&group.off)
                .chain(group.weights.keys())
                .chain(std::iter::once(&group.pick))
                .all(|m| without_account(m) == m),
            "a member's literal # suffix cannot be represented in a Magpie share"
        );
        Ok(serde_json::to_string(
            &json!({"magpie":"routing-group","version":1,"group":group}),
        )?)
    }

    pub fn import_group(&self, text: &str, replace: bool) -> Result<(Self, ImportedGroup)> {
        let mut group = shared_text(text)?;
        let resolved = self.resolved_groups();
        let shared_id = group.id.trim().to_ascii_lowercase();
        let replacing =
            (replace && self.groups.contains_key(&shared_id)).then_some(shared_id.as_str());
        let label = group.name.trim();
        let (name, id) = self.free_group_name(
            &resolved,
            if label.is_empty() {
                "Shared group"
            } else {
                label
            },
            replacing,
        );
        let mut report = ImportedGroup {
            id: id.clone(),
            name: name.clone(),
            replaced: replacing.is_some(),
            ..Default::default()
        };
        let mut local = HashSet::new();
        let mut equivalents = BTreeMap::new();
        for provider in &self.providers {
            for model in &provider.models {
                let target = format!("{}/{model}", provider.id);
                equivalents
                    .entry(self.merge_key(provider, model))
                    .or_insert_with(|| target.clone());
                local.insert(target);
            }
        }
        let mut members = Vec::new();
        let mut added = HashSet::new();
        let mut mapped = BTreeMap::new();
        for member in &group.members {
            if group.matched.contains(member) {
                continue;
            }
            if super::MemberPattern::is_pattern(member) {
                group.patterns.push(member.clone());
                continue;
            }
            let raw = without_account(member.trim());
            if mapped.contains_key(raw) {
                continue;
            }
            let candidate = if local.contains(raw) {
                raw
            } else {
                raw.strip_prefix("magpie/").unwrap_or(raw)
            };
            let target = if local.contains(candidate) {
                Some(candidate.to_owned())
            } else if candidate.starts_with("group/")
                || self.canonical_auto_ref(candidate).is_some()
            {
                resolved.group_reference(self, candidate)
            } else {
                candidate.split_once('/').and_then(|(pid, model)| {
                    let catalog = self
                        .providers
                        .iter()
                        .find(|p| p.id == pid)
                        .map_or("", |p| p.catalog.as_str());
                    equivalents
                        .get(&merge_name(self.equivalent_model(pid, catalog, model)))
                        .cloned()
                })
            };
            if let Some(target) = &target {
                if target != candidate {
                    report.instead.push(format!("{candidate} → {target}"));
                }
                if added.insert(target.clone()) {
                    members.push(target.clone());
                }
            } else {
                report.missing.push(candidate.to_owned());
            }
            mapped.insert(raw.to_owned(), target);
        }
        let mut cfg = self.clone();
        cfg.set_member_patterns(&id, group.patterns)?;
        cfg.group_placed.remove(&id);
        ensure!(
            !members.is_empty() || cfg.group_match.contains_key(&id),
            "no members available here: {}",
            report.missing.join(", ")
        );
        let expanded = cfg.expand_patterns(&id, &members);
        let here = |m: &str| {
            let m = without_account(m.trim());
            mapped
                .get(m)
                .and_then(Option::as_ref)
                .cloned()
                .or_else(|| expanded.iter().find(|id| id.as_str() == m).cloned())
        };
        let placed: Vec<_> = group.placed.iter().filter_map(|m| here(m)).collect();
        if cfg.group_match.contains_key(&id) && !placed.is_empty() {
            cfg.group_placed.insert(id.clone(), placed);
        }
        if replacing.is_none() {
            cfg.group_display_order = resolved
                .ordered_names(&self.group_display_order)
                .into_iter()
                .cloned()
                .collect();
            cfg.group_display_order.push(id.clone());
        }
        cfg.group_labels.insert(id.clone(), name);
        cfg.group_notes.insert(id.clone(), group.note);
        cfg.group_unlists.remove(&id);
        if group.unlists {
            cfg.group_unlists.insert(id.clone());
        }
        let off: BTreeSet<_> = group.off.iter().filter_map(|m| here(m)).collect();
        cfg.group_off.remove(&id);
        if !off.is_empty() && (group.routing == "manual" || off.len() < expanded.len()) {
            cfg.group_off.insert(id.clone(), off);
        }
        cfg.group_picks.remove(&id);
        cfg.group_last_picks.remove(&id);
        cfg.group_weights.remove(&id);
        cfg.group_last_weights.remove(&id);
        cfg.rotating_groups.remove(&id);
        let pick = here(&group.pick);
        if group.routing == "manual" {
            cfg.group_picks.insert(
                id.clone(),
                pick.unwrap_or_else(|| {
                    cfg.expand_patterns(&id, &members)
                        .into_iter()
                        .next()
                        .unwrap_or_default()
                }),
            );
        } else if let Some(pick) = pick {
            cfg.group_last_picks.insert(id.clone(), pick);
        }
        let mut weights = BTreeMap::new();
        for (member, weight) in group.weights {
            if let Some(target) = here(&member)
                && weight > 0
            {
                let entry = weights.entry(target).or_insert(weight);
                *entry = weight.max(*entry);
            }
        }
        if group.routing == "weight" {
            cfg.group_weights.insert(id.clone(), weights);
        } else if !weights.is_empty() {
            cfg.group_last_weights.insert(id.clone(), weights);
        }
        if group.routing == "rotate" {
            cfg.rotating_groups.insert(id.clone());
        }
        cfg.groups.insert(id.clone(), members);
        for old in group.former {
            if !cfg.groups.contains_key(&old)
                && !resolved.members.contains_key(&old)
                && !cfg.hidden_auto_groups.contains(&old)
            {
                cfg.former_groups.entry(old).or_insert_with(|| id.clone());
            }
        }
        let cfg = Config::parse(&serde_json::to_string(&cfg)?)?;
        Ok((cfg, report))
    }
}
