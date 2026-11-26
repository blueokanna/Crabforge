use anyhow::Result;
use clap::Subcommand;
use crabforge_core::{
    project_instructions::read_markdown,
    rules::{self, Edit, Rule},
};
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub enum Command {
    /// List rules, revisions and native file statuses as JSON.
    List,
    /// Add, edit or rename a rule and apply it to the selected agents.
    Save {
        name: String,
        /// UTF-8 Markdown file containing the rule.
        #[arg(long)]
        from: PathBuf,
        /// Existing name, including when editing without renaming.
        #[arg(long)]
        old: Option<String>,
        /// Require the existing rule to match the revision from list.
        #[arg(long, requires = "old")]
        revision: Option<String>,
        /// Assign to claude, cline or kiro-cli. Repeat for each agent.
        #[arg(long = "agent")]
        agents: Vec<String>,
        /// Limit the rule to this glob. Repeat for each pattern.
        #[arg(long = "path")]
        paths: Vec<String>,
    },
    /// Remove a rule and its unchanged managed copies.
    Remove {
        name: String,
        #[arg(long)]
        revision: Option<String>,
    },
    /// Retry applying the library and print the resulting file statuses.
    Sync,
}

pub fn run(dir: &Path, command: Command) -> Result<()> {
    let edit = match command {
        Command::List => Edit::Read,
        Command::Sync => Edit::Sync,
        Command::Save {
            name,
            from,
            old,
            revision,
            agents,
            paths,
        } => Edit::Save {
            old,
            expected: revision,
            rule: Rule {
                name,
                body: read_markdown(&from)?,
                paths,
                agents,
            },
        },
        Command::Remove { name, revision } => Edit::Remove {
            name,
            expected: revision,
        },
    };
    println!("{}", rules::library(dir, edit)?);
    Ok(())
}
