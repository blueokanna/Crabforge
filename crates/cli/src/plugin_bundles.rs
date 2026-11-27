use anyhow::Result;
use clap::Subcommand;
use crabforge_core::plugin_bundles::{self, Edit};
use std::path::{Path, PathBuf};

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
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
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
    };
    println!("{}", plugin_bundles::edit(dir, edit)?);
    Ok(())
}
