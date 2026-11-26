use anyhow::Result;
use clap::Subcommand;
use crabforge_core::skills::{self, Edit};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List skills, revisions and native folder statuses as JSON.
    List,
    /// Import a folder containing SKILL.md and its supporting files.
    Import {
        source: PathBuf,
        /// Assign to claude, codex, gemini, opencode or pi. Repeat for each agent.
        #[arg(long = "agent")]
        agents: Vec<String>,
        /// Require the revision from list when updating an existing skill.
        #[arg(long)]
        revision: Option<String>,
    },
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
