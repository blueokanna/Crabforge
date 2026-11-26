use anyhow::{Context, Result, ensure};
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MemberPattern {
    text: String,
    ids: Regex,
    free: bool,
}

impl From<MemberPattern> for String {
    fn from(p: MemberPattern) -> String {
        p.text
    }
}

impl TryFrom<String> for MemberPattern {
    type Error = anyhow::Error;

    fn try_from(input: String) -> Result<Self> {
        let raw = unprefix(&input);
        let (scope, free) = match raw.strip_prefix("free:") {
            Some(s) => (unprefix(s), true),
            None => (raw, false),
        };
        ensure!(
            !scope.is_empty() && !scope.starts_with("free:"),
            "invalid member pattern {input}"
        );
        ensure!(
            free || Self::is_pattern(scope),
            "expected a glob or re: expression: {scope}"
        );
        let expression = if let Some(re) = scope.strip_prefix("re:") {
            ensure!(!re.trim().is_empty(), "empty regular expression");
            re.to_owned()
        } else {
            scope
                .split('*')
                .map(regex::escape)
                .collect::<Vec<_>>()
                .join(".*")
        };
        let ids = fancy_regex::RegexBuilder::new(&format!(r"(?i)\A(?:{expression})\z"))
            .backtrack_limit(100_000)
            .build()
            .with_context(|| format!("invalid member pattern {scope}"))?;
        let text = if free {
            format!("free:{scope}")
        } else {
            scope.to_owned()
        };
        Ok(Self { text, ids, free })
    }
}

fn unprefix(input: &str) -> &str {
    input.trim().strip_prefix("magpie/").unwrap_or(input.trim())
}

impl MemberPattern {
    pub fn is_pattern(input: &str) -> bool {
        let s = unprefix(input);
        s.starts_with("re:") || s.starts_with("free:") || s.contains('*')
    }
    pub fn as_str(&self) -> &str {
        &self.text
    }
    pub(crate) fn matches(&self, id: &str, free: bool) -> bool {
        (!self.free || free) && self.ids.is_match(id).unwrap_or(false)
    }
}

impl super::Config {
    pub fn pattern_hits(&self, rule: &MemberPattern) -> usize {
        self.providers
            .iter()
            .map(|p| {
                p.models
                    .iter()
                    .filter(|m| {
                        rule.matches(&format!("{}/{}", p.id, m), p.free_models.contains(*m))
                    })
                    .count()
            })
            .sum()
    }

    /// Preserve the slots of matched models while applying edits to the named sequence.
    pub fn reorder_named_members(&mut self, group: &str, named: &[String]) {
        let Some(before) = self.group_placed.remove(group) else {
            return;
        };
        let natural = self.expand_patterns(group, named);
        let available: HashSet<_> = natural.iter().collect();
        let explicit: HashSet<_> = named.iter().collect();
        let mut order: Vec<_> = before
            .into_iter()
            .filter(|id| available.contains(id))
            .collect();
        let mut seen: HashSet<_> = order.iter().cloned().collect();
        let after_named = order
            .iter()
            .rposition(|id| explicit.contains(id))
            .map_or(0, |i| i + 1);
        order.splice(
            after_named..after_named,
            named
                .iter()
                .filter(|id| seen.insert((*id).clone()))
                .cloned(),
        );
        order.extend(
            natural
                .iter()
                .filter(|id| seen.insert((*id).clone()))
                .cloned(),
        );
        for (slot, id) in order
            .iter_mut()
            .filter(|id| explicit.contains(&**id))
            .zip(named)
        {
            slot.clone_from(id);
        }
        if order != natural {
            self.group_placed.insert(group.to_owned(), order);
        }
    }

    pub fn set_member_patterns(&mut self, name: &str, patterns: Vec<String>) -> Result<()> {
        let mut seen = BTreeSet::new();
        let mut rules = Vec::new();
        for s in patterns.into_iter().filter(|s| !s.trim().is_empty()) {
            let rule = MemberPattern::try_from(s)?;
            if seen.insert(rule.text.clone()) {
                rules.push(rule);
            }
        }
        if rules.is_empty() {
            self.group_match.remove(name);
            self.group_placed.remove(name);
        } else {
            self.group_match.insert(name.to_owned(), rules);
        }
        Ok(())
    }

    pub fn expand_patterns(&self, name: &str, named: &[String]) -> Vec<String> {
        let mut members = named.to_vec();
        let Some(rules) = self.group_match.get(name) else {
            return members;
        };
        let named: HashSet<_> = named.iter().collect();
        for p in &self.providers {
            for model in &p.models {
                let id = format!("{}/{model}", p.id);
                if !named.contains(&id)
                    && rules
                        .iter()
                        .any(|r| r.matches(&id, p.free_models.contains(model)))
                {
                    members.push(id);
                }
            }
        }
        if let Some(placed) = self.group_placed.get(name) {
            let rank: std::collections::HashMap<_, _> =
                placed.iter().enumerate().map(|(i, id)| (id, i)).collect();
            members.sort_by_key(|id| rank.get(id).copied().unwrap_or(usize::MAX));
        }
        members
    }
}
