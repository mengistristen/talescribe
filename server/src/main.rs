use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::commands::{init, run};

mod commands;
mod handlers;

enum State {
    Idle,
    Entry { current_roll: Option<String> },
    Exit,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(about = "Create a new project in the current directory")]
    Init,
    #[command(about = "Run the project in the current directory")]
    Run {
        #[arg(short, long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        CliCommand::Init => init()?,
        CliCommand::Run { client } => run(client)?,
    }

    Ok(())
}
