use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::plugin_bundles::{self, Edit, ProjectEdit, WorkspaceAgent};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List plugin snapshots and their revisions in the shared library.
    List,
    /// Inspect a plugin folder, exported TAR archive or Git source and report its content hash.
    Preview { source: String },
    /// List plugin sources declared by a Claude-compatible marketplace.
    Marketplace { source: String },
    /// Store a reviewed snapshot. Set the existing revision when updating it.
    Save {
        source: String,
        #[arg(long)]
        content: String,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Preview or restore complete plugin snapshots from an encrypted Magpie backup.
    ImportMagpie {
        file: PathBuf,
        /// Select one or more names. Defaults to all plugin snapshots in the backup.
        #[arg(long = "name")]
        names: Vec<String>,
        /// Save snapshots to the shared catalog for project assignment.
        #[arg(long)]
        apply: bool,
    },
    /// Remove a bundle from the shared catalog.
    Remove {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Export a saved bundle as a portable TAR archive.
    Export {
        name: String,
        output: PathBuf,
        #[arg(long)]
        revision: String,
    },
    /// Inspect a bundle component, or copy one listed resource to its library.
    Resources {
        name: String,
        kind: String,
        path: String,
        #[arg(long)]
        revision: String,
        #[arg(long)]
        copy: Option<String>,
    },
    /// List or change Pi package assignments in a project.
    PiProject {
        project: PathBuf,
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// List or change Claude Code plugin assignments in a project.
    ClaudeProject {
        project: PathBuf,
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// List or change Codex plugin assignments in a project.
    CodexProject {
        project: PathBuf,
        #[command(subcommand)]
        command: ProjectCommand,
    },
    /// List or change Antigravity plugin assignments in a project.
    AntigravityProject {
        project: PathBuf,
        #[command(subcommand)]
        command: ProjectCommand,
    },
}

#[derive(Subcommand)]
pub(super) enum ProjectCommand {
    /// List the bundles and their assignment status.
    List,
    /// Assign a reviewed package or sync an existing assignment to its latest saved revision.
    Assign {
        name: String,
        #[arg(long)]
        bundle_revision: String,
        #[arg(long)]
        project_revision: String,
    },
    /// Withdraw the assignment and preserve any project edits to its package entry.
    Withdraw {
        name: String,
        #[arg(long)]
        project_revision: String,
    },
    /// Print a portable JSON document of assigned package names and snapshot hashes.
    Export {
        #[arg(long)]
        project_revision: String,
    },
    /// Export the package snapshot currently assigned to the project as a TAR.
    Archive {
        name: String,
        output: PathBuf,
        #[arg(long)]
        project_revision: String,
    },
    /// Apply a portable JSON document to a project using matching saved snapshots.
    Apply {
        file: PathBuf,
        #[arg(long)]
        project_revision: String,
    },
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    let mut assignments = String::new();
    let edit = match &command {
        Command::List => Edit::Read,
        Command::Preview { source } => Edit::Preview { source },
        Command::Marketplace { source } => Edit::Marketplace { source },
        Command::Save {
            source,
            content,
            revision,
        } => Edit::Save {
            source,
            content,
            revision: revision.as_deref(),
        },
        Command::ImportMagpie { file, names, apply } => {
            let passphrase = super::backup_passphrase()?;
            let report = plugin_bundles::import_magpie(dir, file, &passphrase, names, *apply)?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
            return Ok(());
        }
        Command::Remove { name, revision } => Edit::Remove { name, revision },
        Command::Export {
            name,
            output,
            revision,
        } => Edit::Export {
            name,
            output,
            revision,
        },
        Command::Resources {
            name,
            kind,
            path,
            revision,
            copy,
        } => Edit::Resources {
            name,
            revision,
            kind,
            path,
            copy: copy.as_deref(),
        },
        scope @ (Command::PiProject { project, command }
        | Command::ClaudeProject { project, command }
        | Command::CodexProject { project, command }
        | Command::AntigravityProject { project, command }) => {
            let edit = match command {
                ProjectCommand::List => ProjectEdit::Read,
                ProjectCommand::Assign {
                    name,
                    bundle_revision,
                    project_revision,
                } => ProjectEdit::Set {
                    name,
                    project_revision,
                    bundle_revision: Some(bundle_revision),
                },
                ProjectCommand::Withdraw {
                    name,
                    project_revision,
                } => ProjectEdit::Set {
                    name,
                    project_revision,
                    bundle_revision: None,
                },
                ProjectCommand::Export { project_revision } => {
                    ProjectEdit::Export { project_revision }
                }
                ProjectCommand::Archive {
                    name,
                    output,
                    project_revision,
                } => ProjectEdit::Archive {
                    name,
                    project_revision,
                    output,
                },
                ProjectCommand::Apply {
                    file,
                    project_revision,
                } => {
                    File::open(file)?
                        .take(1024 * 1024 + 1)
                        .read_to_string(&mut assignments)?;
                    ensure!(
                        assignments.len() <= 1024 * 1024,
                        "project assignment document exceeds 1 MiB"
                    );
                    ProjectEdit::Apply {
                        project_revision,
                        assignments: &assignments,
                    }
                }
            };
            match scope {
                Command::PiProject { .. } => Edit::PiProject { project, edit },
                Command::ClaudeProject { .. } => Edit::ClaudeProject { project, edit },
                Command::CodexProject { .. } => Edit::WorkspaceProject {
                    agent: WorkspaceAgent::Codex,
                    project,
                    edit,
                },
                _ => Edit::WorkspaceProject {
                    agent: WorkspaceAgent::Antigravity,
                    project,
                    edit,
                },
            }
        }
    };
    println!("{}", plugin_bundles::edit(dir, edit)?);
    Ok(())
}
