use super::{KINDS, project_key, scan};
use anyhow::{Result, ensure};
use std::{cmp::Reverse, path::Path};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

const MAX_CHARS: usize = 1500;
const INTRO: &str = "## Project memory\n\nUse these notes with the project's instructions.\n";
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
    render(dir, &scope)
}

fn render(dir: &Path, scope: &str) -> Result<String> {
    let mut matching = Vec::new();
    let mut unreadable = scan(&dir.join("memory"), |entry| {
        if entry.scope == scope && entry.status == "active" {
            let kind = KINDS.iter().position(|kind| *kind == entry.kind).unwrap();
            let updated = OffsetDateTime::parse(&entry.updated, &Rfc3339).ok();
            matching.push((Reverse(entry.pinned), kind, Reverse(updated), entry));
        }
    })?;
    unreadable.sort_unstable();
    ensure!(
        unreadable.is_empty(),
        "cannot publish while memory files are unreadable: {}",
        unreadable.join(", ")
    );
    matching.sort_unstable_by(|a, b| (&a.0, &a.1, &a.2, &a.3.id).cmp(&(&b.0, &b.1, &b.2, &b.3.id)));
    let mut groups: [String; HEADINGS.len()] = Default::default();
    let mut used = INTRO.chars().count();
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
        line = line
            .replace("<!-- magpie:", "<!-- magpie :")
            .replace("<!-- crabforge:", "<!-- crabforge :");
        let section = &mut groups[kind];
        let heading = HEADINGS[kind];
        let cost = line.chars().count()
            + 1
            + if section.is_empty() {
                heading.chars().count() + 6
            } else {
                0
            };
        if used + cost > MAX_CHARS {
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
        return Ok(String::new());
    }
    Ok(format!("{INTRO}{}", groups.concat()))
}
