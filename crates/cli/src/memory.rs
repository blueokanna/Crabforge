use anyhow::Result;
use clap::{Args, Subcommand};
use crabforge_core::{memory, project_instructions::read_markdown};
use nextjson::NsonSerialize;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum McpCommand {
    /// Share scoped memories with a locally configured MCP client.
    Memory {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        project_key: Option<String>,
    },
}

pub(super) fn serve(dir: &Path, command: McpCommand) -> Result<()> {
    let McpCommand::Memory { agent, project_key } = command;
    memory::mcp::serve(
        dir,
        agent.as_deref(),
        project_key.as_deref(),
        std::io::stdin().lock(),
        std::io::stdout().lock(),
    )
}

#[derive(Subcommand)]
pub(super) enum Command {
    /// List or search memories. Includes all statuses unless filtered.
    Search {
        #[arg(default_value = "")]
        text: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long, default_value_t = 0)]
        offset: usize,
    },
    /// Show a memory and its revision.
    Show { id: String },
    /// Create a memory from UTF-8 Markdown.
    Add {
        from: PathBuf,
        #[command(flatten)]
        fields: Fields,
    },
    /// Update selected fields in a memory or change its status.
    Edit {
        id: String,
        #[arg(long)]
        from: Option<PathBuf>,
        #[arg(long)]
        revision: Option<String>,
        #[command(flatten)]
        fields: Fields,
    },
}

#[derive(Args)]
pub(super) struct Fields {
    #[arg(long)]
    kind: Option<String>,
    #[arg(long)]
    scope: Option<String>,
    #[arg(long)]
    status: Option<String>,
    #[arg(long = "tag", group = "tags-action")]
    tags: Vec<String>,
    #[arg(long, group = "tags-action")]
    clear_tags: bool,
    #[arg(long)]
    why: Option<String>,
    #[arg(long)]
    pinned: Option<bool>,
}

impl Fields {
    fn changes(self, from: Option<&Path>) -> Result<memory::Changes> {
        Ok(memory::Changes {
            text: from.map(read_markdown).transpose()?,
            kind: self.kind,
            scope: self.scope,
            status: self.status,
            tags: (self.clear_tags || !self.tags.is_empty()).then_some(self.tags),
            why: self.why,
            pinned: self.pinned,
        })
    }
}

fn print(value: &impl NsonSerialize) -> Result<()> {
    println!(
        "{}",
        nextjson::to_string_pretty(value).map_err(anyhow::Error::msg)?
    );
    Ok(())
}

pub(super) fn run(dir: &Path, command: Command) -> Result<()> {
    match command {
        Command::Show { id } => print(&memory::read(dir, &id)?),
        Command::Search {
            text,
            status,
            scope,
            kind,
            tags,
            limit,
            offset,
        } => print(&memory::search(
            dir,
            memory::Query {
                text,
                status,
                scope,
                kind,
                tags,
                limit,
                offset,
            },
        )?),
        Command::Add { from, fields } => print(&memory::save(
            dir,
            None,
            None,
            fields.changes(Some(&from))?,
        )?),
        Command::Edit {
            id,
            from,
            revision,
            fields,
        } => {
            let revision = match revision {
                Some(revision) => revision,
                None => memory::read(dir, &id)?.revision,
            };
            print(&memory::save(
                dir,
                Some(&id),
                Some(&revision),
                fields.changes(from.as_deref())?,
            )?)
        }
    }
}
