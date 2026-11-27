use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::hooks::{self, Edit, ProjectAssignment};
use std::{
    io::{self, Read},
    path::Path,
};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List saved hooks, native assignments and existing hooks available for adoption.
    List,
    /// Read a hook definition as JSON from standard input and publish it.
    Save {
        /// Existing hook name, including when editing without renaming.
        #[arg(long, requires = "revision")]
        old: Option<String>,
        #[arg(long, requires = "old")]
        revision: Option<String>,
    },
    /// Adopt an unchanged native command hook in place.
    Adopt { key: String, name: String },
    /// Remove the installed hook selected by its current discovery key and keep a backup.
    RemoveInstalled { key: String },
    /// Restore an installed-hook backup if the native file is unchanged since removal.
    Restore { id: String },
    /// Remove a hook and its unchanged managed copies.
    Remove {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Retry the native assignments.
    Sync,
    /// Assign a shared hook to project clients. Omit --agent to withdraw it.
    Assign {
        name: String,
        #[arg(long)]
        revision: String,
        #[arg(long = "agent")]
        agents: Vec<String>,
    },
}

pub(super) fn run(dir: &Path, project: Option<&Path>, command: Command) -> Result<()> {
    let edit = match command {
        Command::List => Edit::Read,
        Command::Save { old, revision } => {
            let mut source = String::new();
            io::stdin()
                .take(hooks::MAX_ENTRY as u64 + 1)
                .read_to_string(&mut source)?;
            ensure!(
                source.len() <= hooks::MAX_ENTRY,
                "hook document exceeds 64 KiB"
            );
            Edit::Save {
                source,
                previous: old.zip(revision),
            }
        }
        Command::Adopt { key, name } => Edit::Adopt { key, name },
        Command::RemoveInstalled { key } => Edit::RemoveInstalled { key },
        Command::Restore { id } => Edit::Restore { id },
        Command::Remove { name, revision } => Edit::Remove { name, revision },
        Command::Sync => Edit::Sync,
        Command::Assign {
            name,
            revision,
            agents,
        } => Edit::Assign(ProjectAssignment {
            name,
            revision,
            agents,
        }),
    };
    let result = match project {
        Some(project) => hooks::edit_project(dir, project, edit),
        None => hooks::edit(dir, edit),
    }?;
    println!("{result}");
    Ok(())
}
