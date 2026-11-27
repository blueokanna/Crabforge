use anyhow::{Result, ensure};
use clap::Subcommand;
use crabforge_core::claude_plugins::{self, Edit, Manage};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum Command {
    /// List plugins, marketplaces and the current settings revision.
    List,
    /// Browse plugins available from Claude Code marketplaces.
    Available,
    /// Enable an installed plugin in the selected scope.
    Enable {
        id: String,
        #[arg(long)]
        revision: String,
    },
    /// Disable a user plugin or withdraw its managed project assignment.
    Disable {
        id: String,
        #[arg(long)]
        revision: String,
    },
    /// Add a local folder, Git repository or URL as a Claude Code marketplace.
    AddMarketplace { source: String },
    /// Refresh a registered marketplace from its source.
    UpdateMarketplace { name: String },
    /// Remove a marketplace from the Claude Code user profile.
    RemoveMarketplace { name: String },
    /// Install a plugin from a registered Claude Code marketplace.
    Install { id: String },
    /// Update a plugin in the Claude Code user profile.
    Update { id: String },
    /// Uninstall a user plugin and retain its persistent data.
    Uninstall { id: String },
    /// Write portable Claude Code plugin settings to stdout as JSON.
    ExportSettings,
    /// Add portable settings from a JSON file to the Claude Code user profile.
    ImportSettings { file: PathBuf },
}

pub(super) fn run(dir: &Path, project: Option<&Path>, command: Command) -> Result<()> {
    let set = |id, revision, enabled| Edit::Set {
        id,
        revision,
        enabled,
    };
    let manage = |action, target: &str| {
        ensure!(
            project.is_none(),
            "manage installations and marketplaces in the user profile"
        );
        claude_plugins::manage(dir, action, target)
    };
    let result = match &command {
        Command::List => claude_plugins::edit(dir, project, Edit::Read),
        Command::Available => {
            ensure!(project.is_none(), "browse marketplaces in the user profile");
            claude_plugins::available(dir)
        }
        Command::Enable { id, revision } => {
            claude_plugins::edit(dir, project, set(id, revision, true))
        }
        Command::Disable { id, revision } => {
            claude_plugins::edit(dir, project, set(id, revision, false))
        }
        Command::AddMarketplace { source } => manage(Manage::AddMarketplace, source),
        Command::UpdateMarketplace { name } => manage(Manage::UpdateMarketplace, name),
        Command::RemoveMarketplace { name } => manage(Manage::RemoveMarketplace, name),
        Command::Install { id } => manage(Manage::Install, id),
        Command::Update { id } => manage(Manage::Update, id),
        Command::Uninstall { id } => manage(Manage::Uninstall, id),
        Command::ExportSettings => {
            ensure!(
                project.is_none(),
                "export plugin settings from the user profile"
            );
            claude_plugins::export_settings(dir)
        }
        Command::ImportSettings { file } => {
            ensure!(
                project.is_none(),
                "import plugin settings into the user profile"
            );
            claude_plugins::import_settings_file(dir, file)?;
            claude_plugins::edit(dir, None, Edit::Read)
        }
    };
    println!("{}", result?);
    Ok(())
}
