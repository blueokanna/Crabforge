use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod catalogs;
#[cfg(windows)]
mod memory;

#[derive(Parser)]
struct Options {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Survey a built CLI with isolated provider configurations and emit CSV.
    Catalogs {
        cli: PathBuf,
        #[arg(long,default_value_t=4,value_parser=clap::value_parser!(u8).range(1..=8))]
        parallel: u8,
    },
    #[cfg(windows)]
    Gateway {
        cli: PathBuf,
        magpie: PathBuf,
        #[arg(value_enum)]
        workload: memory::Workload,
        #[arg(long,default_value_t=3,value_parser=clap::value_parser!(u8).range(1..=100))]
        rounds: u8,
    },
    #[cfg(windows)]
    Desktop {
        gui: PathBuf,
        #[arg(long)]
        magpie_pid: Option<u32>,
        #[arg(long,default_value_t=3,value_parser=clap::value_parser!(u8).range(1..=100))]
        rounds: u8,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    match Options::parse().command {
        Command::Catalogs { cli, parallel } => catalogs::run(&cli.canonicalize()?, parallel).await,
        #[cfg(windows)]
        Command::Gateway {
            cli,
            magpie,
            workload,
            rounds,
        } => {
            memory::gateway(
                &cli.canonicalize()?,
                &magpie.canonicalize()?,
                workload,
                rounds,
            )
            .await
        }
        #[cfg(windows)]
        Command::Desktop {
            gui,
            magpie_pid,
            rounds,
        } => memory::desktop(&gui.canonicalize()?, magpie_pid, rounds).await,
    }
}
