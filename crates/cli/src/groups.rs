use anyhow::{Context, Result};
use crabforge_core::config::Config;

pub(super) fn show(config: &Config, id: &str) -> Result<()> {
    let groups = config.resolved_groups();
    let name = config.group_name(id);
    let name = if groups.members.contains_key(&name) {
        &name
    } else if let Some((name, _)) = config
        .group_labels
        .iter()
        .find(|(_, label)| label.to_lowercase() == id.to_lowercase())
    {
        name
    } else if config.hidden_auto_groups.contains(&name) {
        println!("group/{name}\tremoved");
        println!("restore\tcarbforge-cli group restore {name}");
        return Ok(());
    } else {
        anyhow::bail!("unknown route group {id}");
    };
    let canonical = format!("group/{name}");
    let public = groups.public_refs.get(&canonical).unwrap_or(&canonical);
    let members = groups.display_members(config, name);
    let label = config.group_labels.get(name).unwrap_or(name);
    println!("{label}\t{public}");
    if config.disabled_groups.contains(name) {
        println!("state\toff");
    }
    if !config.groups.contains_key(name) {
        println!("source\tautomatic");
    }
    if let Some(note) = config.group_notes.get(name) {
        println!("note\t{note}");
    }
    let weights = config.group_weights.get(name);
    let pick = config.group_picks.get(name);
    let routing = if pick.is_some() {
        "manual"
    } else if weights.is_some() {
        "weight"
    } else if config.rotating_groups.contains(name) {
        "rotate"
    } else {
        "order"
    };
    println!("routing\t{routing}");
    for rule in config.group_match.get(name).into_iter().flatten() {
        println!("match\t{}\t{}", rule.as_str(), config.pattern_hits(rule));
    }
    if config.group_unlists.contains(name) {
        println!("listed members\tno");
    }
    if let Some(pick) = pick
        && !members.is_empty()
    {
        let selected = members
            .iter()
            .find(|m| *m == pick)
            .or(members.first())
            .context("empty manual group")?;
        println!("pick\t{selected}");
    }
    for (i, member) in members.iter().enumerate() {
        print!("{}\t{member}", i + 1);
        if let Some(weights) = weights {
            print!("\tweight={}", weights.get(member).unwrap_or(&1));
        }
        if config
            .group_off
            .get(name)
            .is_some_and(|off| off.contains(member))
        {
            print!("\toff");
        }
        println!();
    }
    for parent in groups.ordered_names(&config.group_display_order) {
        if groups.members[parent].contains(&canonical) {
            let id = format!("group/{parent}");
            println!("used by\t{}", groups.public_refs.get(&id).unwrap_or(&id));
        }
    }
    Ok(())
}
