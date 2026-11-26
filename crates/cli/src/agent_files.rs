use anyhow::Result;
use clap::Subcommand;
use crabforge_core::agent_files::{self, Edit, Entry, Kind};
use std::{
    io::{self, Read},
    path::Path,
};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List resources, client IDs and publication statuses as JSON.
    List,
    /// Find existing native files and return their content revisions.
    Discover,
    /// Manage an unchanged native resource at its existing path.
    Adopt {
        agent: String,
        path: String,
        #[arg(long)]
        revision: String,
    },
    /// Publish a resource definition read as UTF-8 JSON from standard input.
    Save {
        /// Previous resource name when editing or renaming an entry.
        #[arg(long, requires = "revision")]
        old: Option<String>,
        #[arg(long, requires = "old")]
        revision: Option<String>,
    },
    /// Remove the resource and its unchanged managed files.
    Remove {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Retry publishing the current assignments.
    Sync,
}

pub(super) fn run(dir: &Path, kind: Kind, project: Option<&Path>, command: Command) -> Result<()> {
    let edit = match command {
        Command::List => Edit::Read,
        Command::Discover => Edit::Discover,
        Command::Adopt {
            agent,
            path,
            revision,
        } => Edit::Adopt {
            agent,
            path,
            revision,
        },
        Command::Save { old, revision } => {
            let mut source = String::new();
            io::stdin()
                .take(agent_files::MAX_DOCUMENT as u64 + 1)
                .read_to_string(&mut source)?;
            Edit::Save {
                previous: old.zip(revision),
                entry: Entry::from_json(&source)?,
            }
        }
        Command::Remove { name, revision } => Edit::Remove { name, revision },
        Command::Sync => Edit::Sync,
    };
    println!("{}", agent_files::library(dir, kind, project, edit)?);
    Ok(())
}
