use super::{Entry, KINDS, project_key, scan, scope_key};
use anyhow::{Result, ensure};
use std::{cmp::Reverse, path::Path};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const PROJECT_INTRO: &str =
    "## Project memory\n\nUse these notes with the project's instructions.\n";
const GLOBAL_INTRO: &str =
    "## Memory\n\nUse these notes alongside the user's current request and project instructions.\n";
const AGENT_INTRO: &str =
    "## Agent memory\n\nUse these notes with the user's current instructions.\n";
const HEADINGS: [&str; 7] = [
    "About the user",
    "Preferences",
    "Rules",
    "References",
    "Facts",
    "Procedures",
    "Recent decisions",
];

pub fn project_digest(dir: &Path, project: &Path) -> Result<String> {
    let scope = format!("project:{}", project_key(project)?);
    let mut entries = Vec::new();
    active_entries(dir, |entry| {
        if entry.scope == scope {
            entries.push(entry);
        }
    })?;
    Ok(render(entries, PROJECT_INTRO, 1500))
}

/// Preview global notes and, when selected, the specified agent's own notes.
pub fn agent_digest(dir: &Path, agent: Option<&str>) -> Result<String> {
    let scope = agent.map(|key| scope_key("agent", key)).transpose()?;
    let mut global = Vec::new();
    let mut specific = Vec::new();
    active_entries(dir, |entry| {
        if entry.scope == "global" {
            global.push(entry);
        } else if scope.as_ref() == Some(&entry.scope) {
            specific.push(entry);
        }
    })?;
    let mut digest = render(global, GLOBAL_INTRO, 4000);
    let specific = render(specific, AGENT_INTRO, 1000);
    if !digest.is_empty() && !specific.is_empty() {
        digest.push('\n');
    }
    digest.push_str(&specific);
    Ok(digest)
}

fn active_entries(dir: &Path, mut visit: impl FnMut(Entry)) -> Result<()> {
    let mut unreadable = scan(&dir.join("memory"), |entry| {
        if entry.status == "active" {
            visit(entry);
        }
    })?;
    unreadable.sort_unstable();
    ensure!(
        unreadable.is_empty(),
        "cannot build a digest while memory files are unreadable: {}",
        unreadable.join(", ")
    );
    Ok(())
}

fn render(entries: Vec<Entry>, intro: &str, budget: usize) -> String {
    let mut matching: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            let kind = KINDS.iter().position(|kind| *kind == entry.kind).unwrap();
            let updated = OffsetDateTime::parse(&entry.updated, &Rfc3339).ok();
            (Reverse(entry.pinned), kind, Reverse(updated), entry)
        })
        .collect();
    matching.sort_unstable_by(|a, b| (&a.0, &a.1, &a.2, &a.3.id).cmp(&(&b.0, &b.1, &b.2, &b.3.id)));
    let mut groups: [String; HEADINGS.len()] = Default::default();
    let mut used = intro.chars().count();
    for (_, kind, _, entry) in matching {
        let mut lines = entry.text.lines().map(str::trim_end);
        let mut line = format!("- {}", lines.next().unwrap_or_default());
        for content in lines {
            line.push('\n');
            if !content.is_empty() {
                line.push_str("  ");
                line.push_str(content);
            }
        }
        if !entry.why.is_empty() {
            let why = entry.why.split_whitespace().collect::<Vec<_>>().join(" ");
            line.push_str(&format!(" (reason: {why})"));
        }
        line = line.replace("<!-- crabforge:", "<!-- crabforge :");
        let section = &mut groups[kind];
        let heading = HEADINGS[kind];
        let cost = line.chars().count()
            + 1
            + if section.is_empty() {
                heading.chars().count() + 6
            } else {
                0
            };
        if used + cost > budget {
            continue;
        }
        if section.is_empty() {
            section.push_str(&format!("\n### {heading}\n"));
        }
        section.push_str(&line);
        section.push('\n');
        used += cost;
    }
    if groups.iter().all(String::is_empty) {
        return String::new();
    }
    format!("{intro}{}", groups.concat())
}
