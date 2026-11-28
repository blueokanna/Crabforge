use super::{Section, agent_target, identity, optional, read_path, replace, write};
use crate::memory;
use anyhow::{Result, ensure};
use nextjson::{NsonDeserialize, NsonSerialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    time::Duration,
};
use time::OffsetDateTime;

const STATE: &str = "memory-publication.json";
const DAY: i64 = 24 * 60 * 60;

#[derive(NsonSerialize, NsonDeserialize)]
struct Publication {
    target: String,
    digest: String,
    pending: Option<String>,
    updated: i64,
}

type Publications = BTreeMap<String, Publication>;

fn load(dir: &Path) -> Result<Publications> {
    optional(&dir.join(STATE), 16 * 1024)?
        .map(|source| nextjson::from_str(&source).map_err(anyhow::Error::msg))
        .transpose()
        .map(Option::unwrap_or_default)
}

fn save(dir: &Path, publications: &Publications) -> Result<()> {
    replace(
        &dir.join(STATE),
        &nextjson::to_string(publications).map_err(anyhow::Error::msg)?,
    )
}

fn lock(dir: &Path) -> Result<File> {
    fs::create_dir_all(dir)?;
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("memory-publication.lock"))?;
    file.lock()?;
    Ok(file)
}

fn digest_revision(digest: &str) -> String {
    hex::encode(Sha256::digest(digest.trim_end_matches(['\r', '\n'])))
}

pub(super) fn enabled(dir: &Path, agent: &str) -> Result<bool> {
    Ok(load(dir)?.contains_key(agent))
}

pub fn configure(dir: &Path, agent: &str, enabled: bool, revision: &str) -> Result<()> {
    // Resolve the native target before storing an opt-in or accepting an agent ID.
    let target = agent_target(agent)?;
    configure_target(
        dir,
        agent,
        enabled.then_some((target, revision)),
        OffsetDateTime::now_utc().unix_timestamp(),
    )
}

fn configure_target(
    dir: &Path,
    agent: &str,
    enabled: Option<(PathBuf, &str)>,
    now: i64,
) -> Result<()> {
    let _guard = lock(dir)?;
    let mut publications = load(dir)?;
    if let Some((target, revision)) = enabled {
        let view = read_path(dir, target.clone(), Section::AgentMemory)?;
        ensure!(
            view.revision == revision,
            "instructions changed; reload the user file"
        );
        publications.insert(
            agent.into(),
            Publication {
                target: identity(&target),
                digest: digest_revision(&view.memory_digest),
                pending: None,
                updated: now,
            },
        );
    } else {
        publications.remove(agent);
    }
    save(dir, &publications)
}

pub(super) fn publish(
    dir: &Path,
    agent: &str,
    previous: &str,
    preview: Option<&str>,
) -> Result<()> {
    let _guard = lock(dir)?;
    publish_to(
        dir,
        &mut load(dir)?,
        agent,
        agent_target,
        Some((previous, preview)),
        OffsetDateTime::now_utc().unix_timestamp(),
    )
}

fn publish_to(
    dir: &Path,
    publications: &mut Publications,
    agent: &str,
    resolve: impl Fn(&str) -> Result<PathBuf>,
    reviewed: Option<(&str, Option<&str>)>,
    now: i64,
) -> Result<()> {
    let target = resolve(agent)?;
    let view = read_path(dir, target.clone(), Section::AgentMemory)?;
    let target_id = identity(&target);
    let current = digest_revision(&view.memory_digest);
    let tracked = publications
        .get_mut(agent)
        .filter(|p| p.target == target_id);
    if let Some((revision, _)) = reviewed {
        ensure!(
            view.revision == revision,
            "instructions changed; reload the user file"
        );
    } else {
        ensure!(
            tracked
                .as_ref()
                .is_some_and(|p| p.digest == current || p.pending.as_ref() == Some(&current)),
            "the automatic memory target or its published digest changed; review the user instructions"
        );
    }

    let (_folder, _memory_lock) = memory::lock_store(dir)?;
    let digest = memory::agent_digest(dir, Some(agent))?;
    ensure!(
        reviewed
            .and_then(|(_, preview)| preview)
            .is_none_or(|preview| preview == digest),
        "agent memory changed; preview it again"
    );
    let revised = digest_revision(&digest);
    if reviewed.is_none() && current == revised {
        let publication = tracked.expect("automatic publication checked the target");
        publication.digest = revised;
        publication.pending = None;
        publication.updated = now;
        return save(dir, publications);
    }
    let tracked = if let Some(p) = tracked {
        // Keep both revisions until the managed file and publication record agree.
        p.digest = current;
        p.pending = Some(revised.clone());
        save(dir, publications)?;
        true
    } else {
        false
    };
    write(
        dir,
        || {
            let current = resolve(agent)?;
            ensure!(current == target, "user instruction target changed");
            Ok(current)
        },
        &view.revision,
        Some(&digest),
        Section::AgentMemory,
    )?;
    if tracked {
        publications.insert(
            agent.into(),
            Publication {
                target: target_id,
                digest: revised,
                pending: None,
                updated: now,
            },
        );
        save(dir, publications)?;
    }
    Ok(())
}

fn publish_due(dir: &Path, now: i64, resolve: impl Fn(&str) -> Result<PathBuf>) -> Result<()> {
    if load(dir)?.is_empty() {
        return Ok(());
    }
    let _guard = lock(dir)?;
    let mut publications = load(dir)?;
    let due = publications
        .iter()
        .filter(|(_, p)| now.saturating_sub(p.updated) >= DAY)
        .map(|(agent, _)| agent.clone())
        .collect::<Vec<_>>();
    for agent in due {
        if let Err(error) = publish_to(dir, &mut publications, &agent, &resolve, None, now) {
            eprintln!("Memory publication for {agent}: {error:#}");
        }
    }
    Ok(())
}

pub(crate) async fn maintain(dir: &Path) {
    let mut hourly = tokio::time::interval(Duration::from_secs(3600));
    hourly.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        hourly.tick().await;
        if let Err(error) = publish_due(
            dir,
            OffsetDateTime::now_utc().unix_timestamp(),
            agent_target,
        ) {
            eprintln!("Memory publication: {error:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daily_publication_preserves_native_edits_and_excludes_candidates() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let dir = temp.path();
        let native = dir.join("AGENTS.md");
        let resolve = |_: &str| Ok(native.clone());
        let agent = "codex";
        let original = "\u{feff}# Native instructions\r\n";
        fs::write(&native, original)?;
        let add = |text: &str, status: &str| {
            memory::save(
                dir,
                None,
                None,
                memory::Changes {
                    text: Some(text.into()),
                    status: Some(status.into()),
                    ..Default::default()
                },
            )
        };
        add("Use UTF-8", "active")?;
        let candidate = add("Review the new preference", "candidate")?;
        publish_due(dir, DAY, resolve)?;
        assert_eq!(fs::read_to_string(&native)?, original);

        let view = read_path(dir, native.clone(), Section::AgentMemory)?;
        assert!(configure_target(dir, agent, Some((native.clone(), "stale")), 0).is_err());
        configure_target(dir, agent, Some((native.clone(), &view.revision)), 0)?;
        publish_due(dir, DAY - 1, resolve)?;
        assert_eq!(fs::read_to_string(&native)?, original);
        publish_due(dir, DAY, resolve)?;
        let published = fs::read_to_string(&native)?;
        assert!(published.starts_with(original));
        assert!(published.contains("Use UTF-8\r\n"));
        assert!(!published.contains("Review the new preference"));

        memory::save(
            dir,
            Some(&candidate.id),
            Some(&candidate.revision),
            memory::Changes {
                status: Some("active".into()),
                ..Default::default()
            },
        )?;
        let external = published.replace("# Native instructions", "# Updated outside Crabforge");
        fs::write(&native, &external)?;
        let new_target = dir.join("AGENTS.override.md");
        publish_due(dir, 2 * DAY, |_| Ok(new_target.clone()))?;
        assert!(!new_target.exists());
        let edited_digest = external.replace("Use UTF-8", "An external digest edit");
        fs::write(&native, &edited_digest)?;
        publish_due(dir, 2 * DAY, resolve)?;
        assert_eq!(fs::read_to_string(&native)?, edited_digest);

        // Recreate a shutdown between the document write and the final state write.
        fs::write(&native, &external)?;
        let before = read_path(dir, native.clone(), Section::AgentMemory)?;
        let digest = memory::agent_digest(dir, Some(agent))?;
        let mut state = load(dir)?;
        state.get_mut(agent).unwrap().pending = Some(digest_revision(&digest));
        save(dir, &state)?;
        write(
            dir,
            || resolve(agent),
            &before.revision,
            Some(&digest),
            Section::AgentMemory,
        )?;
        publish_due(dir, 2 * DAY, resolve)?;
        let recovered = &load(dir)?[agent];
        assert!(recovered.pending.is_none());
        assert_eq!(recovered.updated, 2 * DAY);
        let published = fs::read_to_string(&native)?;
        assert!(published.starts_with("\u{feff}# Updated outside Crabforge\r\n"));
        assert!(published.contains("Review the new preference"));

        configure_target(dir, agent, None, 2 * DAY)?;
        assert!(!enabled(dir, agent)?);
        publish_due(dir, 3 * DAY, resolve)?;
        assert_eq!(fs::read_to_string(&native)?, published);
        Ok(())
    }
}
