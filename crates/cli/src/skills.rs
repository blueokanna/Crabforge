use anyhow::Result;
use clap::Subcommand;
use crabforge_core::skills::{self, Edit};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List skills, revisions and native folder statuses as JSON.
    List,
    /// Find unmanaged skills in supported agent profiles and list their content hashes.
    Discover,
    /// Manage an installed skill in place after checking its discovered content hash.
    Adopt {
        agent: String,
        name: String,
        path: PathBuf,
        #[arg(long)]
        content: String,
    },
    /// Import a local folder or a Git source for a skill and its supporting files.
    Import {
        source: PathBuf,
        /// Assign to claude, codex, gemini, opencode or pi. Repeat for each agent.
        #[arg(long = "agent")]
        agents: Vec<String>,
        /// Require the revision from list when updating an existing skill.
        #[arg(long)]
        revision: Option<String>,
    },
    /// Compare a remote skill's files with the library snapshot without applying changes.
    Check {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// List available skills in a local collection or at a Git ref.
    Catalog { source: String },
    /// Set aside a candidate using its source from the catalog.
    Dismiss { source: String },
    /// Replace the selected agents, or clear the assignments if none are given.
    Assign {
        name: String,
        #[arg(long = "agent")]
        agents: Vec<String>,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Remove the library entry and unchanged managed copies.
    Remove {
        name: String,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Retry pending changes for the current agent profiles.
    Sync,
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    let edit = match command {
        Command::Discover => {
            println!("{}", skills::discover(dir)?);
            return Ok(());
        }
        Command::Check { name, revision } => {
            println!("{}", skills::check_update(dir, &name, &revision)?);
            return Ok(());
        }
        Command::Catalog { source } => {
            println!("{}", skills::source_catalog(dir, &source)?);
            return Ok(());
        }
        Command::Dismiss { source } => {
            skills::dismiss_candidate(dir, &source)?;
            return Ok(());
        }
        Command::Adopt {
            agent,
            name,
            path,
            content,
        } => Edit::Adopt {
            agent,
            name,
            path,
            content,
        },
        Command::List => Edit::Read,
        Command::Import {
            source,
            agents,
            revision,
        } => Edit::Import {
            source,
            agents,
            expected: revision,
        },
        Command::Assign {
            name,
            agents,
            revision,
        } => Edit::Assign {
            name,
            agents,
            expected: revision,
        },
        Command::Remove { name, revision } => Edit::Remove {
            name,
            expected: revision,
        },
        Command::Sync => Edit::Sync,
    };
    println!("{}", skills::library(dir, edit)?);
    Ok(())
}
