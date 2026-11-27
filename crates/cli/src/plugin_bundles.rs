use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::plugin_bundles::{self, Edit, PiProjectEdit};
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
        command: PiProjectCommand,
    },
}

#[derive(Subcommand)]
pub(super) enum PiProjectCommand {
    /// List the Pi bundles and their assignment status.
    List,
    /// Assign a reviewed Pi package or sync an existing assignment to its latest saved revision.
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
        Command::PiProject { project, command } => Edit::PiProject {
            project,
            edit: match command {
                PiProjectCommand::List => PiProjectEdit::Read,
                PiProjectCommand::Assign {
                    name,
                    bundle_revision,
                    project_revision,
                } => PiProjectEdit::Set {
                    name,
                    project_revision,
                    bundle_revision: Some(bundle_revision),
                },
                PiProjectCommand::Withdraw {
                    name,
                    project_revision,
                } => PiProjectEdit::Set {
                    name,
                    project_revision,
                    bundle_revision: None,
                },
                PiProjectCommand::Export { project_revision } => {
                    PiProjectEdit::Export { project_revision }
                }
                PiProjectCommand::Archive {
                    name,
                    output,
                    project_revision,
                } => PiProjectEdit::Archive {
                    name,
                    project_revision,
                    output,
                },
                PiProjectCommand::Apply {
                    file,
                    project_revision,
                } => {
                    File::open(file)?
                        .take(1024 * 1024 + 1)
                        .read_to_string(&mut assignments)?;
                    ensure!(
                        assignments.len() <= 1024 * 1024,
                        "Pi project assignment document exceeds 1 MiB"
                    );
                    PiProjectEdit::Apply {
                        project_revision,
                        assignments: &assignments,
                    }
                }
            },
        },
    };
    println!("{}", plugin_bundles::edit(dir, edit)?);
    Ok(())
}
