use super::{Config, merge_name};
use crate::config::{Classifiers, valid_group_name, valid_id};
use anyhow::{Context, Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize, Value, json};
use std::collections::{BTreeMap, BTreeSet, HashSet};

#[derive(Default, NsonSerialize, NsonDeserialize)]
#[njson(default)]
struct SharedGroup {
    id: String,
    name: String,
    members: Option<Vec<String>>,
    #[njson(rename = "match", skip_serializing_if = "Vec::is_empty")]
    patterns: Vec<String>,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    placed: Vec<String>,
    #[njson(skip_serializing)]
    matched: Vec<String>,
    routing: String,
    #[njson(rename = "firstToken", skip_serializing_if = "crate::config::is_zero")]
    first_content: i32,
    #[njson(skip_serializing_if = "crate::config::is_zero")]
    patience: i32,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    rules: Vec<super::rules::RoutingRule>,
    #[njson(skip_serializing_if = "String::is_empty")]
    effort: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    classifier: String,
    #[njson(rename = "classifierMore", skip_serializing_if = "Vec::is_empty")]
    classifier_more: Vec<String>,
    #[njson(rename = "classifyPrompt", skip_serializing_if = "String::is_empty")]
    classify_prompt: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    note: String,
    #[njson(skip_serializing_if = "String::is_empty")]
    pick: String,
    #[njson(skip_serializing_if = "Vec::is_empty")]
    former: Vec<String>,
    #[njson(skip_serializing_if = "BTreeSet::is_empty")]
    off: BTreeSet<String>,
    #[njson(skip_serializing_if = "BTreeMap::is_empty")]
    weights: BTreeMap<String, u32>,
    #[njson(skip_serializing_if = "std::ops::Not::not")]
    unlists: bool,
    #[njson(flatten, skip_serializing)]
    other: BTreeMap<String, Value>,
}

#[derive(Default, nextjson::NsonSerialize)]
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
    let doc: Value = nextjson::from_str(raw).map_err(anyhow::Error::msg)?;
    ensure!(
        doc.get("magpie").and_then(Value::as_str) == Some("routing-group"),
        "expected a Magpie routing-group share"
    );
    let version = doc.get("version").unwrap_or(&Value::Null);
    ensure!(
        crate::json::unsigned(version) == Some(1),
        "unsupported routing-group share version {}",
        version
    );
    let group: SharedGroup =
        nextjson::from_value(doc.get("group").context("missing shared group")?.clone())
            .map_err(anyhow::Error::msg)?;
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
    group.validate_routing()?;
    Ok(group)
}

impl SharedGroup {
    fn take_classifiers(&mut self) -> Classifiers {
        Classifiers {
            effort: std::mem::take(&mut self.effort),
            classifier: std::mem::take(&mut self.classifier),
            classifier_more: std::mem::take(&mut self.classifier_more),
            classify_prompt: std::mem::take(&mut self.classify_prompt),
        }
    }

    fn validate_routing(&self) -> Result<()> {
        ensure!(
            matches!(
                self.routing.as_str(),
                "" | "order" | "manual" | "rotate" | "weight"
            ),
            "unsupported shared routing strategy: {}",
            self.routing
        );
        Ok(())
    }
}

impl Config {
    fn restore_group_routing(
        &mut self,
        id: &str,
        routing: &str,
        pick: Option<String>,
        weights: BTreeMap<String, u32>,
    ) {
        self.group_picks.remove(id);
        self.group_last_picks.remove(id);
        self.group_weights.remove(id);
        self.group_last_weights.remove(id);
        self.rotating_groups.remove(id);
        if routing == "manual" {
            self.group_picks
                .insert(id.to_owned(), pick.unwrap_or_default());
        } else if let Some(pick) = pick {
            self.group_last_picks.insert(id.to_owned(), pick);
        }
        if routing == "weight" {
            self.group_weights.insert(id.to_owned(), weights);
        } else if !weights.is_empty() {
            self.group_last_weights.insert(id.to_owned(), weights);
        }
        if routing == "rotate" {
            self.rotating_groups.insert(id.to_owned());
        }
    }

    pub(in crate::config) fn restore_magpie_groups(
        &mut self,
        definitions: Vec<Value>,
        selected: &[String],
        order: &[String],
    ) -> Result<Vec<String>> {
        let resolved = self.resolved_groups();
        let mut occupied: BTreeSet<_> = resolved
            .members
            .keys()
            .chain(&self.hidden_auto_groups)
            .chain(self.former_groups.keys())
            .map(String::as_str)
            .chain(
                resolved
                    .aliases
                    .keys()
                    .filter_map(|id| id.strip_prefix("group/")),
            )
            .map(str::to_owned)
            .collect();
        let mut ids = Vec::new();
        for raw in definitions {
            let source = raw
                .get("id")
                .and_then(Value::as_str)
                .context("missing Magpie group ID")?;
            if !selected.is_empty() && !selected.iter().any(|id| id == source) {
                continue;
            }
            let id = source.to_owned();
            let group: SharedGroup = nextjson::from_value(raw)
                .map_err(|_| anyhow::anyhow!("invalid Magpie group definition: {id}"))?;
            ensure!(
                valid_id(&id) && !id.starts_with("auto-"),
                "unsupported Magpie group ID {id:?}"
            );
            ensure!(occupied.insert(id.clone()), "group ID already in use: {id}");
            self.restore_magpie_group(&id, group, &mut occupied)
                .with_context(|| format!("Magpie group {id}"))?;
            ids.push(id);
        }
        for id in selected {
            ensure!(ids.contains(id), "group {id:?} is absent from the backup");
        }
        ensure!(!ids.is_empty(), "Magpie backup has no groups to import");
        let ranks: BTreeMap<_, _> = order.iter().enumerate().map(|(i, id)| (id, i)).collect();
        ids.sort_by_key(|id| ranks.get(id).copied().unwrap_or(usize::MAX));
        self.group_display_order = resolved
            .ordered_names(&self.group_display_order)
            .into_iter()
            .cloned()
            .chain(ids.iter().cloned())
            .collect();
        Ok(ids)
    }

    fn restore_magpie_group(
        &mut self,
        id: &str,
        mut group: SharedGroup,
        occupied: &mut BTreeSet<String>,
    ) -> Result<()> {
        let disabled: bool = group
            .other
            .remove("disabled")
            .map(nextjson::from_value)
            .transpose()
            .map_err(|_| anyhow::anyhow!("invalid disabled state"))?
            .unwrap_or_default();
        for field in group.other.keys() {
            ensure!(
                field == "routedAt",
                "unsupported Magpie group setting: {field}"
            );
        }
        ensure!(group.matched.is_empty(), "backup contains derived members");
        group.validate_routing()?;
        let classifiers = group.take_classifiers();
        let members = group.members.unwrap_or_default();
        self.set_member_patterns(id, group.patterns)?;
        ensure!(
            group.placed.is_empty() || self.group_match.contains_key(id),
            "member placement requires patterns"
        );
        let expanded = self.expand_patterns(id, &members);
        let available: HashSet<_> = expanded.iter().map(String::as_str).collect();
        for member in group
            .placed
            .iter()
            .chain(&group.off)
            .chain(group.weights.keys())
        {
            ensure!(
                available.contains(member.as_str()),
                "unavailable group member {member:?}"
            );
        }
        let pick = if group.pick.is_empty() {
            None
        } else {
            ensure!(
                available.contains(group.pick.as_str()),
                "unavailable group pick {:?}",
                group.pick
            );
            Some(group.pick)
        };
        ensure!(
            group.routing == "manual"
                || expanded.is_empty()
                || expanded.iter().any(|m| !group.off.contains(m)),
            "all members are switched off"
        );
        for alias in group.former {
            ensure!(
                valid_group_name(&alias),
                "invalid former group ID {alias:?}"
            );
            ensure!(
                occupied.insert(alias.clone()),
                "group ID already in use: {alias}"
            );
            self.former_groups.insert(alias, id.to_owned());
        }
        self.group_labels.insert(
            id.to_owned(),
            if group.name.is_empty() {
                id.to_owned()
            } else {
                group.name
            },
        );
        self.group_notes.insert(id.to_owned(), group.note);
        self.set_group_first_content(id, group.first_content);
        self.set_group_patience(id, group.patience);
        if !group.rules.is_empty() {
            self.group_rules.insert(id.into(), group.rules);
        }
        if !classifiers.is_empty() {
            self.group_classifiers.insert(id.into(), classifiers);
        }
        if disabled {
            self.disabled_groups.insert(id.to_owned());
        }
        if group.unlists {
            self.group_unlists.insert(id.to_owned());
        }
        if !group.placed.is_empty() {
            self.group_placed.insert(id.to_owned(), group.placed);
        }
        if !group.off.is_empty() {
            self.group_off.insert(id.to_owned(), group.off);
        }
        self.restore_group_routing(id, &group.routing, pick, group.weights);
        self.groups.insert(id.to_owned(), members);
        Ok(())
    }

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
        let classifiers = self.group_classifiers.get(&id).cloned().unwrap_or_default();
        let mut group = SharedGroup {
            name: self.group_labels.get(&id).unwrap_or(&id).clone(),
            members: Some(members.clone()),
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
            first_content: self
                .group_first_content
                .get(&id)
                .copied()
                .unwrap_or_default(),
            patience: self.group_patience.get(&id).copied().unwrap_or_default(),
            rules: self.group_rules.get(&id).cloned().unwrap_or_default(),
            effort: classifiers.effort,
            classifier: classifiers.classifier,
            classifier_more: classifiers.classifier_more,
            classify_prompt: classifiers.classify_prompt,
            unlists: self.group_unlists.contains(&id),
            id: id.clone(),
            ..Default::default()
        };
        if !group.rules.is_empty() {
            let displayed = groups.display_members(self, &id);
            for rule in &mut group.rules {
                rule.remap(|target| {
                    let member = displayed
                        .iter()
                        .zip(&groups.members[&id])
                        .find(|(saved, canonical)| *saved == target || *canonical == target)
                        .map_or(target, |(saved, _)| saved);
                    Some(member.to_owned())
                });
            }
        }
        ensure!(
            group
                .members
                .iter()
                .flatten()
                .chain(&group.placed)
                .chain(&group.off)
                .chain(group.weights.keys())
                .chain(std::iter::once(&group.pick))
                .map(String::as_str)
                .chain(group.rules.iter().flat_map(|rule| rule.targets()))
                .chain(std::iter::once(group.classifier.as_str()))
                .chain(group.classifier_more.iter().map(String::as_str))
                .all(|m| without_account(m) == m),
            "a member's literal # suffix cannot be represented in a Magpie share"
        );
        nextjson::to_string(&json!({"magpie":"routing-group","version":1,"group":group}))
            .map_err(anyhow::Error::msg)
    }

    pub fn import_group(&self, text: &str, replace: bool) -> Result<(Self, ImportedGroup)> {
        let mut group = shared_text(text)?;
        let mut classifiers = group.take_classifiers();
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
        let resolve = |raw: &str| {
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
            (candidate.to_owned(), target)
        };
        for member in group.members.iter().flatten() {
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
            let (candidate, target) = resolve(raw);
            if let Some(target) = &target {
                if *target != candidate {
                    report.instead.push(format!("{candidate} → {target}"));
                }
                if added.insert(target.clone()) {
                    members.push(target.clone());
                }
            } else {
                report.missing.push(candidate);
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
        cfg.set_group_first_content(&id, group.first_content);
        cfg.set_group_patience(&id, group.patience);
        cfg.group_classifiers.remove(&id);
        if !classifiers.is_empty() {
            for target in classifiers.targets() {
                ensure!(
                    resolve(without_account(target.trim())).1.is_some(),
                    "shared classifier {target:?} is unavailable"
                );
            }
            classifiers.rename_targets(|target| resolve(without_account(target.trim())).1);
            cfg.group_classifiers.insert(id.clone(), classifiers);
        }
        cfg.group_rules.remove(&id);
        if !group.rules.is_empty() {
            for rule in &mut group.rules {
                for target in rule.targets() {
                    ensure!(
                        here(target).is_some(),
                        "shared routing rule member {target:?} is unavailable"
                    );
                }
                rule.remap(here);
            }
            cfg.group_rules.insert(id.clone(), group.rules);
        }
        cfg.group_unlists.remove(&id);
        if group.unlists {
            cfg.group_unlists.insert(id.clone());
        }
        let off: BTreeSet<_> = group.off.iter().filter_map(|m| here(m)).collect();
        cfg.group_off.remove(&id);
        if !off.is_empty() && (group.routing == "manual" || off.len() < expanded.len()) {
            cfg.group_off.insert(id.clone(), off);
        }
        let pick = here(&group.pick).or_else(|| {
            if group.routing == "manual" {
                cfg.expand_patterns(&id, &members).into_iter().next()
            } else {
                None
            }
        });
        let mut weights = BTreeMap::new();
        for (member, weight) in group.weights {
            if let Some(target) = here(&member)
                && weight > 0
            {
                let entry = weights.entry(target).or_insert(weight);
                *entry = weight.max(*entry);
            }
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
        cfg.restore_group_routing(&id, &group.routing, pick, weights);
        let cfg = Config::parse(&cfg.to_json()?)?;
        Ok((cfg, report))
    }
}
