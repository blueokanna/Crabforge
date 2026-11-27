use super::{Config, Provider};
use std::collections::{BTreeMap, BTreeSet};
mod pattern;
mod share;
pub use pattern::MemberPattern;
pub use share::ImportedGroup;
include!(concat!(env!("OUT_DIR"), "/catalog_names.rs"));

pub(super) fn default_catalog(id: &str) -> &str {
    match id {
        "kimi-code" => "kimi-code-plan-global",
        "kimi-code-cn" => "kimi-code-plan-cn",
        "moonshot" | "moonshot-cn" => "moonshotai",
        other => other,
    }
}

/// Derived from the configured catalogs. Only explicit edits belong in settings.json.
pub struct ResolvedGroups {
    pub members: BTreeMap<String, Vec<String>>,
    /// Expanded members in display order, before resolving stored aliases.
    pub expanded: BTreeMap<String, Vec<String>>,
    pub aliases: BTreeMap<String, String>,
    /// Public model ID for each internal group/<name> that has a vendor spelling.
    pub public_refs: BTreeMap<String, String>,
}

impl ResolvedGroups {
    pub fn group_reference(&self, cfg: &Config, id: &str) -> Option<String> {
        let canonical = cfg.canonical_auto_ref(id);
        let name = canonical.as_deref().unwrap_or(id);
        let target = self.aliases.get(name).map_or(name, String::as_str);
        self.members
            .contains_key(target.strip_prefix("group/")?)
            .then(|| target.to_owned())
    }
    pub fn display_members<'a>(&'a self, config: &'a Config, name: &str) -> &'a Vec<String> {
        self.expanded
            .get(name)
            .or_else(|| config.groups.get(name))
            .unwrap_or_else(|| &self.members[name])
    }
    pub fn listed_refs(&self, config: &Config) -> Vec<String> {
        self.ordered_names(&config.group_display_order)
            .into_iter()
            .filter(|name| !config.disabled_groups.contains(*name))
            .map(|name| {
                let id = format!("group/{name}");
                self.public_refs.get(&id).cloned().unwrap_or(id)
            })
            .collect()
    }

    /// New groups follow the visible groups whose order has been saved.
    pub fn ordered_names<'a>(&'a self, preferred: &[String]) -> Vec<&'a String> {
        let rank: BTreeMap<_, _> = preferred
            .iter()
            .enumerate()
            .map(|(i, name)| (name, i))
            .collect();
        let mut names: Vec<_> = self.members.keys().collect();
        if !preferred.is_empty() {
            names.sort_by_key(|name| rank.get(*name).copied().unwrap_or(usize::MAX));
        }
        names
    }
}

fn windowless(model: &str) -> &str {
    let Some((name, suffix)) = model.rsplit_once('@') else {
        return model;
    };
    let grok = name.rsplit('/').next().unwrap_or(name).to_ascii_lowercase();
    let digits = suffix.strip_suffix(['k', 'm', 'K', 'M']).or_else(|| {
        grok.strip_prefix("grok-")
            .is_some_and(|v| v.as_bytes().first().is_some_and(u8::is_ascii_digit))
            .then_some(suffix)
    });
    if !name.contains('@')
        && digits.is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
    {
        name
    } else {
        model
    }
}

fn merge_name(model: &str) -> String {
    crate::model_name::same_model(windowless(model.trim()))
}

fn slug(key: &str) -> String {
    key.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

impl Config {
    /// Catalog for agent profiles, including bare and provider-qualified model IDs.
    pub fn agent_model_ids(&self, groups: &ResolvedGroups) -> Vec<String> {
        let mut ids = BTreeSet::new();
        for p in &self.providers {
            for m in &p.models {
                ids.insert(m.clone());
                ids.insert(format!("{}/{m}", p.id));
            }
        }
        let mut group_ids = groups.listed_refs(self);
        if self.group_display_order.is_empty() {
            ids.extend(group_ids);
            return ids.into_iter().collect();
        }
        for id in &group_ids {
            ids.remove(id);
        }
        group_ids.extend(ids);
        group_ids
    }

    pub fn unlisted_model_ids(&self, resolved: &ResolvedGroups) -> BTreeSet<String> {
        let mut hidden: BTreeSet<String> = self
            .group_unlists
            .iter()
            .filter(|name| !self.disabled_groups.contains(*name))
            .filter_map(|name| resolved.members.get(name))
            .flatten()
            .filter(|id| !id.starts_with("group/"))
            .cloned()
            .collect();
        if hidden.is_empty() {
            return hidden;
        }
        // A bare spelling can also be another provider's qualified model ID.
        // Only hide it as an alias when every provider serving that spelling is unlisted.
        let mut hidden_bare: BTreeSet<&str> = hidden
            .iter()
            .filter_map(|id| id.split_once('/').map(|(_, model)| model))
            .collect();
        for provider in &self.providers {
            for model in &provider.models {
                let qualified = format!("{}/{model}", provider.id);
                if !hidden.contains(&qualified) {
                    hidden_bare.remove(model.as_str());
                    hidden_bare.remove(qualified.as_str());
                }
            }
        }
        for public in resolved.public_refs.values() {
            hidden_bare.remove(public.as_str());
        }
        let aliases: Vec<_> = hidden_bare
            .into_iter()
            .filter(|id| !id.starts_with("group/"))
            .map(str::to_owned)
            .collect();
        hidden.extend(aliases);
        hidden
    }

    fn free_group_name(
        &self,
        resolved: &ResolvedGroups,
        requested: &str,
        replacing: Option<&str>,
    ) -> (String, String) {
        let labels: BTreeSet<_> = resolved
            .members
            .keys()
            .filter(|id| Some(id.as_str()) != replacing)
            .map(|id| self.group_labels.get(id).unwrap_or(id).to_lowercase())
            .collect();
        let mut label = requested.to_owned();
        for n in 2.. {
            if !labels.contains(&label.to_lowercase()) {
                break;
            }
            label = format!("{requested} {n}");
        }
        if let Some(id) = replacing {
            return (label, id.to_owned());
        }
        let slug = label
            .to_ascii_lowercase()
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '.')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let mut base = String::new();
        for ch in slug.trim_matches(['-', '.']).chars() {
            if ch != '.' || !base.ends_with('.') {
                base.push(ch);
            }
        }
        if base.is_empty() {
            base.push_str("group");
        }
        // Leave room for a numeric suffix within the persisted ID limit.
        let id = (1usize..)
            .map(|n| {
                let suffix = if n == 1 {
                    String::new()
                } else {
                    format!("-{n}")
                };
                let stem = &base[..base.len().min(96 - suffix.len())];
                format!("{}{suffix}", stem.trim_end_matches(['-', '.']))
            })
            .find(|id| {
                !resolved.members.contains_key(id)
                    && !resolved.aliases.contains_key(&format!("group/{id}"))
                    && !self.hidden_auto_groups.contains(id)
                    && !self.former_groups.contains_key(id)
            })
            .unwrap();
        (label, id)
    }

    /// Copy the configured members and strategy. A discovered group becomes an explicit snapshot.
    pub fn copied_group(&self, from: &str, name: &str) -> anyhow::Result<(Self, String)> {
        let resolved = self.resolved_groups();
        let mut from = self.group_name(from);
        if !resolved.members.contains_key(&from)
            && let Some((id, _)) = self
                .group_labels
                .iter()
                .find(|(_, label)| label.to_lowercase() == from.to_lowercase())
        {
            from.clone_from(id);
        }
        let effective = resolved
            .members
            .get(&from)
            .ok_or_else(|| anyhow::anyhow!("unknown route group {from}"))?;
        let source_label = self.group_labels.get(&from).unwrap_or(&from);
        let requested = if name.trim().is_empty() {
            format!("{source_label} copy")
        } else {
            name.trim().to_owned()
        };
        let (label, id) = self.free_group_name(&resolved, &requested, None);
        let mut cfg = self.clone();
        cfg.groups.insert(
            id.clone(),
            self.groups.get(&from).unwrap_or(effective).clone(),
        );
        cfg.group_labels.insert(id.clone(), label);
        if let Some(rules) = self.group_match.get(&from) {
            cfg.group_match.insert(id.clone(), rules.clone());
        }
        if let Some(order) = self.group_placed.get(&from) {
            cfg.group_placed.insert(id.clone(), order.clone());
        }
        if self.group_unlists.contains(&from) {
            cfg.group_unlists.insert(id.clone());
        }
        if let Some(off) = self.group_off.get(&from) {
            cfg.group_off.insert(id.clone(), off.clone());
        }
        if let Some(note) = self.group_notes.get(&from) {
            cfg.group_notes.insert(id.clone(), note.clone());
        }
        if let Some(&seconds) = self.group_first_content.get(&from) {
            cfg.group_first_content.insert(id.clone(), seconds);
        }
        if let Some(weights) = self.group_weights.get(&from) {
            cfg.group_weights.insert(id.clone(), weights.clone());
        }
        if let Some(weights) = self.group_last_weights.get(&from) {
            cfg.group_last_weights.insert(id.clone(), weights.clone());
        }
        if self.rotating_groups.contains(&from) {
            cfg.rotating_groups.insert(id.clone());
        }
        if let Some(pick) = self.group_picks.get(&from) {
            cfg.group_picks.insert(id.clone(), pick.clone());
        }
        if let Some(pick) = self.group_last_picks.get(&from) {
            cfg.group_last_picks.insert(id.clone(), pick.clone());
        }
        cfg.group_display_order = resolved
            .ordered_names(&self.group_display_order)
            .into_iter()
            .cloned()
            .collect();
        let position = cfg
            .group_display_order
            .iter()
            .position(|g| g == &from)
            .expect("source is visible");
        cfg.group_display_order.insert(position + 1, id.clone());
        cfg.validate()?;
        Ok((cfg, id))
    }

    /// Update a running manual group's selection without changing its members or strategy.
    pub fn pick_member(&mut self, group: &str, member: &str) -> anyhow::Result<()> {
        let resolved = self.resolved_groups();
        anyhow::ensure!(
            resolved.members.contains_key(group)
                && resolved
                    .display_members(self, group)
                    .iter()
                    .any(|m| m == member),
            "unknown member {member} in group {group}"
        );
        let pick = self
            .group_picks
            .get_mut(group)
            .ok_or_else(|| anyhow::anyhow!("group {group} does not use manual selection"))?;
        *pick = member.to_owned();
        Ok(())
    }

    /// Build the complete edit before saving, including parents that use a public alias.
    pub fn renamed_group(&self, from: &str, to: &str) -> anyhow::Result<Self> {
        let from = self.group_name(from);
        let to = to.trim();
        anyhow::ensure!(super::valid_group_name(to), "invalid route group name");
        let resolved = self.resolved_groups();
        let members = resolved
            .members
            .get(&from)
            .ok_or_else(|| anyhow::anyhow!("unknown route group {from}"))?;
        let mut cfg = self.clone();
        if from == to {
            return Ok(cfg);
        }
        anyhow::ensure!(
            !resolved.members.contains_key(to) && !self.hidden_auto_groups.contains(to),
            "group {to} already exists"
        );
        let old = format!("group/{from}");
        let new = format!("group/{to}");
        cfg.group_display_order.retain(|name| name != to);
        for name in &mut cfg.group_display_order {
            if name == &from {
                *name = to.to_owned();
            }
        }
        let references_group = |id: &str| {
            (id.starts_with("group/") || self.canonical_auto_ref(id).is_some())
                && resolved.group_reference(self, id).as_deref() == Some(&old)
        };
        for provider in &mut cfg.providers {
            for id in &mut provider.fallback {
                if references_group(id) {
                    id.clone_from(&new);
                }
            }
        }
        for (name, saved) in &mut cfg.groups {
            for id in saved.iter_mut() {
                if references_group(id) {
                    if let Some(placed) = cfg.group_placed.get_mut(name) {
                        for m in placed.iter_mut().filter(|m| *m == id) {
                            m.clone_from(&new);
                        }
                    }
                    if let Some(off) = cfg.group_off.get_mut(name)
                        && off.remove(id)
                    {
                        off.insert(new.clone());
                    }
                    for choices in [&mut cfg.group_picks, &mut cfg.group_last_picks] {
                        if let Some(pick) = choices.get_mut(name)
                            && pick == id
                        {
                            pick.clone_from(&new);
                        }
                    }
                    for strategy in [&mut cfg.group_weights, &mut cfg.group_last_weights] {
                        if let Some(weights) = strategy.get_mut(name)
                            && let Some(weight) = weights.remove(id)
                        {
                            weights.insert(new.clone(), weight);
                        }
                    }
                    id.clone_from(&new);
                }
            }
        }
        let saved = cfg.groups.remove(&from).unwrap_or_else(|| members.clone());
        cfg.groups.insert(to.into(), saved);
        if let Some(rules) = cfg.group_match.remove(&from) {
            cfg.group_match.insert(to.into(), rules);
        }
        if let Some(order) = cfg.group_placed.remove(&from) {
            cfg.group_placed.insert(to.into(), order);
        }
        if cfg.disabled_groups.remove(&from) {
            cfg.disabled_groups.insert(to.into());
        }
        if cfg.group_unlists.remove(&from) {
            cfg.group_unlists.insert(to.into());
        }
        if let Some(off) = cfg.group_off.remove(&from) {
            cfg.group_off.insert(to.into(), off);
        }
        if let Some(label) = cfg.group_labels.remove(&from) {
            cfg.group_labels.insert(to.into(), label);
        }
        if let Some(note) = cfg.group_notes.remove(&from) {
            cfg.group_notes.insert(to.into(), note);
        }
        if let Some(seconds) = cfg.group_first_content.remove(&from) {
            cfg.group_first_content.insert(to.into(), seconds);
        }
        if cfg.rotating_groups.remove(&from) {
            cfg.rotating_groups.insert(to.into());
        }
        for strategy in [&mut cfg.group_weights, &mut cfg.group_last_weights] {
            if let Some(weights) = strategy.remove(&from) {
                strategy.insert(to.into(), weights);
            }
        }
        for choices in [&mut cfg.group_picks, &mut cfg.group_last_picks] {
            if let Some(pick) = choices.remove(&from) {
                choices.insert(to.into(), pick);
            }
        }
        if from.starts_with("auto-") {
            cfg.hidden_auto_groups.insert(from.clone());
        }
        cfg.former_groups.remove(to);
        for current in cfg.former_groups.values_mut().filter(|g| **g == from) {
            *current = to.to_owned();
        }
        for (alias, target) in &resolved.aliases {
            if target == &old
                && let Some(former) = alias.strip_prefix("group/")
                && former != to
                && !cfg.groups.contains_key(former)
            {
                cfg.former_groups.insert(former.into(), to.into());
            }
        }
        cfg.former_groups.insert(from, to.into());
        cfg.validate()?;
        Ok(cfg)
    }

    pub fn set_group_first_content(&mut self, id: &str, seconds: i32) {
        if seconds == 0 {
            self.group_first_content.remove(id);
        } else {
            self.group_first_content.insert(id.to_owned(), seconds);
        }
    }

    pub fn group_name(&self, reference: &str) -> String {
        let canonical = self.canonical_auto_ref(reference);
        let id = canonical.as_deref().unwrap_or(reference);
        if let Some(name) = id.strip_prefix("group/") {
            let groups = self.resolved_groups();
            if !groups.members.contains_key(name)
                && let Some(target) = groups
                    .aliases
                    .get(id)
                    .and_then(|s| s.strip_prefix("group/"))
            {
                return target.to_string();
            }
            return name.to_string();
        }
        id.to_owned()
    }
    fn equivalent_name<'a>(&'a self, provider: &Provider, model: &'a str) -> &'a str {
        self.equivalent_model(&provider.id, &provider.catalog, model)
    }

    fn equivalent_model<'a>(&'a self, id: &str, catalog: &str, model: &'a str) -> &'a str {
        if !self.model_same_as.is_empty()
            && let Some(other) = self.model_same_as.get(&format!("{id}/{model}"))
        {
            return other;
        }
        if (id == "workbuddy-ai" || catalog == "workbuddy-ai") && model == "deepseek-v4.1-flash-sg"
        {
            return "deepseek-v4.1-flash";
        }
        let catalogs = match catalog.trim() {
            "" => default_catalog(id),
            custom => custom,
        };
        let key = crate::model_name::same_model(model);
        catalogs
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|s| !s.is_empty())
            .find_map(|catalog| {
                let name = catalog.to_ascii_lowercase();
                UNPREFIXED
                    .binary_search_by_key(&(name.as_str(), key.as_str()), |e| e.0)
                    .ok()
                    .map(|i| UNPREFIXED[i].1)
            })
            .unwrap_or(model)
    }

    fn merge_key(&self, provider: &Provider, model: &str) -> String {
        merge_name(self.equivalent_name(provider, model))
    }

    fn group_spelling(&self, name: &str, members: &[String]) -> Option<String> {
        let suffix = name.strip_prefix("auto-")?;
        let mut own = None;
        let mut equivalent = None;
        let mut matched = false;
        for (provider, model) in members
            .iter()
            .filter_map(|m| m.split_once('/'))
            .filter(|(p, _)| *p != "group")
        {
            let Some(provider) = self.providers.iter().find(|p| p.id == provider) else {
                continue;
            };
            let normalized = self.equivalent_name(provider, model);
            let key = merge_name(normalized);
            if slug(&key) != suffix {
                continue;
            }
            matched = true;
            if let Ok(i) = SPELLINGS.binary_search_by_key(&key.as_str(), |entry| entry.0) {
                let vendor = SPELLINGS[i].1;
                if !vendor.chars().any(char::is_whitespace) {
                    return Some(vendor.to_owned());
                }
            }
            let leaf = windowless(normalized.trim()).rsplit('/').next().unwrap();
            if !leaf.chars().any(char::is_whitespace) {
                if normalized == model {
                    own.get_or_insert(leaf);
                } else {
                    equivalent.get_or_insert(leaf);
                }
            }
        }
        own.or(equivalent)
            .or(matched.then_some(suffix))
            .map(str::to_owned)
    }

    pub fn canonical_auto_ref(&self, requested: &str) -> Option<String> {
        let model = requested.trim().strip_prefix("auto/")?;
        if self.providers.iter().any(|p| p.id == "auto") {
            return None;
        }
        let suffix = slug(&merge_name(model.trim_end_matches("[1m]")));
        (!suffix.is_empty()).then(|| format!("group/auto-{suffix}"))
    }

    /// A conversation may still name a group after only one provider remains.
    /// Keep such IDs callable without advertising them in the model catalog.
    pub fn auto_stand_in(&self, requested: &str) -> Option<String> {
        let canonical = self.canonical_auto_ref(requested);
        let suffix = canonical
            .as_deref()
            .unwrap_or(requested)
            .strip_prefix("group/auto-")?;
        let group = format!("auto-{suffix}");
        if suffix.is_empty()
            || (!self.no_auto_groups && self.hidden_auto_groups.contains(&group))
            || self.groups.contains_key(&group)
        {
            return None;
        }
        let mut first = None;
        let mut preferred = None;
        let mut normalized = None;
        for p in &self.providers {
            for model in &p.models {
                let key = self.merge_key(p, model);
                if slug(&key) != suffix {
                    continue;
                }
                if normalized.as_ref().is_some_and(|previous| previous != &key) {
                    return None;
                }
                normalized = Some(key);
                first.get_or_insert_with(|| format!("{}/{model}", p.id));
                if !self.auto_group_skip.contains(&p.id) {
                    preferred.get_or_insert_with(|| format!("{}/{model}", p.id));
                }
            }
        }
        preferred.or(first)
    }

    pub fn resolved_groups(&self) -> ResolvedGroups {
        let mut groups = ResolvedGroups {
            members: self.groups.clone(),
            expanded: BTreeMap::new(),
            aliases: BTreeMap::new(),
            public_refs: BTreeMap::new(),
        };
        for name in self.group_match.keys() {
            if let Some(members) = groups.members.get_mut(name) {
                *members = self.expand_patterns(name, members);
                groups.expanded.insert(name.clone(), members.clone());
            }
        }
        if (self.no_auto_groups || self.providers.len() < 2)
            && !self.groups.keys().any(|g| g.starts_with("auto-"))
            && self.former_groups.is_empty()
        {
            return groups;
        }
        let mut by_model: BTreeMap<String, Vec<(&str, &str)>> = BTreeMap::new();
        let mut spellings: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
        let public = !self.providers.iter().any(|p| p.id == "auto");
        for p in &self.providers {
            if self.auto_group_skip.contains(&p.id) {
                continue;
            }
            for model in &p.models {
                let key = self.merge_key(p, model);
                let names = spellings.entry(key.clone()).or_default();
                names.insert(model);
                let members = by_model.entry(key).or_default();
                // Each provider contributes once. Prefer its unsized model.
                if let Some((last, prev)) = members.last_mut()
                    && *last == p.id
                {
                    if windowless(prev) != *prev && windowless(model) == model {
                        *prev = model
                    }
                } else {
                    members.push((&p.id, model))
                }
            }
        }
        let mut used = BTreeSet::new();
        let mut ambiguous = BTreeSet::new();
        for key in by_model.keys() {
            let id = slug(key);
            if !used.insert(id.clone()) {
                ambiguous.insert(id);
            }
        }
        for (key, entries) in by_model {
            let suffix = slug(&key);
            if suffix.is_empty() || ambiguous.contains(&suffix) {
                continue;
            }
            let name = format!("auto-{suffix}");
            if !groups.members.contains_key(&name) {
                if self.no_auto_groups
                    || self.hidden_auto_groups.contains(&name)
                    || entries.len() < 2
                {
                    continue;
                }
                groups.members.insert(
                    name.clone(),
                    entries.iter().map(|(p, m)| format!("{p}/{m}")).collect(),
                );
            }
            let target = format!("group/{name}");
            if self.groups.contains_key(&name)
                && self.group_spelling(&name, &groups.members[&name]).is_none()
            {
                continue;
            }
            for (provider, model) in &entries {
                let former = format!("group/auto-{}", slug(&crate::model_name::same_model(model)));
                if former != target
                    && !self
                        .model_same_as
                        .contains_key(&format!("{provider}/{model}"))
                    && (*provider, *model) != ("workbuddy-ai", "deepseek-v4.1-flash-sg")
                {
                    groups
                        .aliases
                        .entry(former)
                        .or_insert_with(|| target.clone());
                }
            }
            for model in &spellings[&key] {
                groups
                    .aliases
                    .entry(model.to_string())
                    .or_insert_with(|| target.clone());
                if public {
                    let leaf = windowless(model).rsplit('/').next().unwrap();
                    groups
                        .aliases
                        .entry(format!("auto/{leaf}"))
                        .or_insert_with(|| target.clone());
                }
            }
        }
        if public {
            for (name, members) in &groups.members {
                if let Some(model) = self.group_spelling(name, members) {
                    let target = format!("group/{name}");
                    let reference = format!("auto/{model}");
                    groups.public_refs.insert(target.clone(), reference.clone());
                    groups.aliases.insert(reference, target);
                }
            }
        }
        for (old, current) in &self.former_groups {
            if !groups.members.contains_key(old) && groups.members.contains_key(current) {
                groups
                    .aliases
                    .insert(format!("group/{old}"), format!("group/{current}"));
            }
        }
        // A real group owns its address even when its name was once a shorter
        // vendor spelling for another group. This also applies in saved parents.
        groups.aliases.retain(|alias, _| {
            alias
                .strip_prefix("group/")
                .is_none_or(|name| !groups.members.contains_key(name))
        });
        for (alias, target) in &mut groups.aliases {
            if let Some(canonical) = self.canonical_auto_ref(alias)
                && groups.members.contains_key(&canonical["group/".len()..])
            {
                *target = canonical;
            }
        }
        // References stored by older clients may use a former group ID or a
        // spelling which no configured provider currently advertises.
        for members in self.groups.values() {
            for id in members
                .iter()
                .filter(|id| public && id.starts_with("auto/"))
            {
                if !groups.aliases.contains_key(id)
                    && let Some(canonical) = self.canonical_auto_ref(id)
                {
                    let target = groups.aliases.get(&canonical).cloned().unwrap_or(canonical);
                    if target
                        .strip_prefix("group/")
                        .is_some_and(|name| groups.members.contains_key(name))
                    {
                        groups.aliases.insert(id.clone(), target);
                    }
                }
            }
        }
        for members in groups.members.values_mut() {
            for id in members {
                if ((public && id.starts_with("auto/"))
                    || (id.starts_with("group/") && !self.groups.contains_key(&id[6..])))
                    && let Some(target) = groups.aliases.get(id)
                {
                    id.clone_from(target);
                }
            }
        }
        groups
    }
}
