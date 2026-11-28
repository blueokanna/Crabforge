use anyhow::Result;
use clap::Subcommand;
use crabforge_core::{
    project_instructions::read_markdown,
    project_mcp::{self, Edit, library},
};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum Command {
    /// Read a project's native MCP servers and revision.
    List { project: PathBuf, agent: String },
    /// Add or replace one server from a native JSON object or Codex TOML table body.
    Save {
        project: PathBuf,
        agent: String,
        name: String,
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        revision: Option<String>,
        #[arg(long, requires = "revision")]
        replace: bool,
    },
    /// Remove a named server from the inspected revision.
    Remove {
        project: PathBuf,
        agent: String,
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Manage reusable servers and their native project assignments.
    Library {
        #[command(subcommand)]
        command: LibraryCommand,
    },
}

#[derive(Subcommand)]
pub(super) enum LibraryCommand {
    /// List shared servers, revisions, assignments and placement statuses.
    List,
    /// Preview or restore reusable servers from an encrypted backup.
    ImportBackup {
        file: PathBuf,
        /// Repeat to select several names. The default selects all servers.
        #[arg(long = "name")]
        names: Vec<String>,
        /// Save the definitions to the library for explicit project assignment.
        #[arg(long)]
        apply: bool,
    },
    /// Create or update a shared server from a transport JSON object.
    Save {
        name: String,
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Assign or unassign a shared server for one project and agent.
    Assign {
        name: String,
        project: PathBuf,
        agent: String,
        #[arg(long)]
        revision: String,
        #[arg(long)]
        remove: bool,
    },
    /// Remove an entry and its unchanged managed placements.
    Remove {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Apply pending library placements.
    Sync,
}

fn run_library(dir: &Path, command: LibraryCommand) -> Result<()> {
    let edit = match command {
        LibraryCommand::List => library::Edit::Read,
        LibraryCommand::ImportBackup { file, names, apply } => {
            let passphrase = super::backup_passphrase()?;
            let report = library::import_backup(dir, &file, &passphrase, &names, apply)?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
            return Ok(());
        }
        LibraryCommand::Save {
            name,
            from,
            revision,
        } => library::Edit::Save {
            name,
            source: read_markdown(&from)?,
            expected: revision,
        },
        LibraryCommand::Assign {
            name,
            project,
            agent,
            revision,
            remove,
        } => library::Edit::Assign {
            name,
            project,
            agent,
            enabled: !remove,
            expected: revision,
        },
        LibraryCommand::Remove { name, revision } => library::Edit::Remove {
            name,
            expected: revision,
        },
        LibraryCommand::Sync => library::Edit::Sync,
    };
    println!("{}", library::edit(dir, edit)?);
    Ok(())
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    let (project, agent, edit) = match command {
        Command::Library { command } => return run_library(dir, command),
        Command::List { project, agent } => (project, agent, Edit::Read),
        Command::Save {
            project,
            agent,
            name,
            from,
            revision,
            replace,
        } => {
            let source = read_markdown(&from)?;
            (
                project,
                agent,
                Edit::Save {
                    name,
                    source,
                    expected: revision,
                    replace,
                },
            )
        }
        Command::Remove {
            project,
            agent,
            name,
            revision,
        } => (
            project,
            agent,
            Edit::Remove {
                name,
                expected: revision,
            },
        ),
    };
    println!("{}", project_mcp::edit(dir, &project, &agent, edit)?);
    Ok(())
}
