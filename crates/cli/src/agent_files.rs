use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::agent_files::{self, Edit, Entry, ImportRequest, Kind, ProjectAssignment};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List resources, client IDs and publication statuses as JSON.
    List,
    /// Find existing native files and return their content revisions.
    Discover,
    /// Preview resources in a local file, folder or Git source.
    ProbeSource { source: String },
    /// Import selected resource files from a preview request read as UTF-8 JSON.
    ImportSource,
    /// Preview or restore user library definitions from an encrypted Magpie backup.
    ImportMagpie {
        file: PathBuf,
        /// Repeat to select several names. The default selects the whole section.
        #[arg(long = "name")]
        names: Vec<String>,
        /// Save the definitions to Crabforge's library.
        #[arg(long)]
        apply: bool,
    },
    /// Set a shared subagent's clients in a project. Omit --agent to withdraw it.
    Assign {
        name: String,
        #[arg(long)]
        revision: String,
        #[arg(long = "agent")]
        agents: Vec<String>,
    },
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
        Command::ProbeSource { source } => Edit::ProbeSource { source },
        Command::ImportSource => Edit::ImportSource {
            request: ImportRequest::from_json(&document()?)?,
        },
        Command::ImportMagpie { file, names, apply } => {
            ensure!(
                project.is_none(),
                "import Magpie resources into the user library"
            );
            let passphrase = super::backup_passphrase()?;
            let report = agent_files::import_magpie(dir, kind, &file, &passphrase, &names, apply)?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
            return Ok(());
        }
        Command::Assign {
            name,
            revision,
            agents,
        } => Edit::Assign(ProjectAssignment {
            name,
            revision,
            agents,
        }),
        Command::Adopt {
            agent,
            path,
            revision,
        } => Edit::Adopt {
            agent,
            path,
            revision,
        },
        Command::Save { old, revision } => Edit::Save {
            previous: old.zip(revision),
            entry: Entry::from_json(&document()?)?,
        },
        Command::Remove { name, revision } => Edit::Remove { name, revision },
        Command::Sync => Edit::Sync,
    };
    println!("{}", agent_files::library(dir, kind, project, edit)?);
    Ok(())
}

fn document() -> Result<String> {
    let mut source = String::new();
    io::stdin()
        .take(agent_files::MAX_DOCUMENT as u64 + 1)
        .read_to_string(&mut source)?;
    Ok(source)
}
