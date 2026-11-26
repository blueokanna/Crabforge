use anyhow::Result;
use clap::{Args, Subcommand};
use crabforge_core::{memory, project_instructions};
use nextjson::NsonSerialize;
use std::path::{Path, PathBuf};

#[derive(Subcommand)]
pub(super) enum McpCommand {
    /// Share scoped memories with a locally configured MCP client.
    Memory {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, conflicts_with = "project")]
        project_key: Option<String>,
        /// Resolve a project key from this folder's Git checkout or its local path.
        #[arg(long)]
        project: Option<PathBuf>,
    },
}

pub(super) fn serve(dir: &Path, command: McpCommand) -> Result<()> {
    let McpCommand::Memory {
        agent,
        project_key,
        project,
    } = command;
    let project_key = match project {
        Some(folder) => Some(memory::project_key(&folder)?),
        None => project_key,
    };
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
    /// Print the scoped memory key for a project folder.
    ProjectKey {
        #[arg(default_value = ".")]
        folder: PathBuf,
    },
    /// Preview the active memories belonging to this project as Markdown.
    Digest { folder: PathBuf },
    /// Preview global memory and optionally the named agent's own memory as Markdown.
    Context {
        #[arg(long)]
        agent: Option<String>,
    },
    /// Inspect, publish or restore memory in Claude Code or Codex user instructions.
    AgentInstructions {
        agent: String,
        #[arg(long, group = "change")]
        publish: bool,
        #[arg(long, group = "change")]
        restore: bool,
        #[arg(long, requires = "change")]
        revision: Option<String>,
        /// Require this previously exported UTF-8 digest to still match.
        #[arg(long, requires = "publish")]
        preview: Option<PathBuf>,
    },
    /// Publish the project's memory digest in the selected agent's instructions.
    Publish {
        folder: PathBuf,
        agent: String,
        #[arg(long)]
        revision: Option<String>,
    },
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
            text: from.map(project_instructions::read_markdown).transpose()?,
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
        Command::ProjectKey { folder } => {
            println!("{}", memory::project_key(&folder)?);
            Ok(())
        }
        Command::Digest { folder } => {
            print!("{}", memory::project_digest(dir, &folder)?);
            Ok(())
        }
        Command::Context { agent } => {
            print!("{}", memory::agent_digest(dir, agent.as_deref())?);
            Ok(())
        }
        Command::AgentInstructions {
            agent,
            publish,
            restore,
            revision,
            preview,
        } => {
            let view = project_instructions::read_agent(dir, &agent)?;
            let revision = revision.as_deref().unwrap_or(&view.revision);
            if publish {
                let preview = preview
                    .as_deref()
                    .map(project_instructions::read_markdown)
                    .transpose()?
                    .map(|text| text.trim_start_matches('\u{feff}').replace("\r\n", "\n"));
                project_instructions::publish_agent_memory(
                    dir,
                    &agent,
                    revision,
                    preview.as_deref(),
                )?;
            } else if restore {
                project_instructions::edit_agent(dir, &agent, revision, None)?;
            } else {
                return print(&view);
            }
            print(&project_instructions::read_agent(dir, &agent)?)
        }
        Command::Publish {
            folder,
            agent,
            revision,
        } => {
            let revision = match revision {
                Some(revision) => revision,
                None => project_instructions::read(dir, &folder, &agent)?.revision,
            };
            project_instructions::publish_memory(dir, &folder, &agent, &revision, None)?;
            print(&project_instructions::read(dir, &folder, &agent)?)
        }
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
