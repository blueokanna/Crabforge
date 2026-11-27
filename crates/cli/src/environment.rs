use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::environment::{self, Edit};
use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List library entries, revisions and native publication status.
    List {
        #[arg(long)]
        show_values: bool,
    },
    /// Preview or restore environment variables from an encrypted Magpie backup.
    ImportMagpie {
        file: PathBuf,
        /// Repeat to select several names. The default selects all variables.
        #[arg(long = "name")]
        names: Vec<String>,
        /// Save the definitions to Crabforge's local library.
        #[arg(long)]
        apply: bool,
    },
    /// Read a UTF-8 value from standard input and set its complete agent assignment.
    Save {
        name: String,
        #[arg(long = "agent", value_parser = ["claude", "codex", "pi"])]
        agents: Vec<String>,
        #[arg(long, requires = "revision")]
        rename_from: Option<String>,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Remove a variable from the library and its owned native entries.
    Remove {
        name: String,
        #[arg(long)]
        revision: String,
    },
    /// Retry publishing the current assignments to native configuration.
    Sync,
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    let (edit, reveal) = match command {
        Command::List { show_values } => (Edit::Read, show_values),
        Command::ImportMagpie { file, names, apply } => {
            let passphrase = super::backup_passphrase()?;
            let report = environment::import_magpie(dir, &file, &passphrase, &names, apply)?;
            println!(
                "{}",
                nextjson::to_string_pretty(&report).map_err(anyhow::Error::msg)?
            );
            return Ok(());
        }
        Command::Save {
            name,
            agents,
            rename_from,
            revision,
        } => {
            let mut value = String::new();
            io::stdin()
                .take(environment::MAX_VALUE as u64 + 1)
                .read_to_string(&mut value)?;
            ensure!(
                value.len() <= environment::MAX_VALUE,
                "environment value exceeds 8192 bytes"
            );
            let previous =
                revision.map(|revision| (rename_from.unwrap_or_else(|| name.clone()), revision));
            (
                Edit::Save {
                    name,
                    value,
                    agents,
                    previous,
                },
                false,
            )
        }
        Command::Remove { name, revision } => (Edit::Remove { name, revision }, false),
        Command::Sync => (Edit::Sync, false),
    };
    println!("{}", environment::edit(dir, edit, reveal)?);
    Ok(())
}
