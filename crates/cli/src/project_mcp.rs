use anyhow::Result;
use clap::Subcommand;
use crabforge_core::{
    project_instructions::read_markdown,
    project_mcp::{self, Edit},
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
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    let (project, agent, edit) = match command {
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
