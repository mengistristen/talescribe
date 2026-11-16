use clap::{Parser, Subcommand};
use log::error;
use std::path::PathBuf;
use talescribe_protocol::prompt::Prompt;

use crate::commands::{init, run};

mod commands;
mod handlers;
mod roll;

enum State {
    Idle,
    Prompt { prompts: Vec<Prompt> },
    Entry { prompts: Vec<String> },
    Exit,
}

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Enable logging to a file
    #[arg(short, long)]
    log: bool,
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    #[command(about = "Create a new project in the current directory")]
    Init,
    #[command(about = "Run the project in the current directory")]
    Run {
        /// Path to client executable
        #[arg(short, long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if cli.log {
        simple_logging::log_to_file("info.log", log::LevelFilter::Info)?;
    }

    let result = match cli.command {
        CliCommand::Init => init(),
        CliCommand::Run { client } => run(client),
    };

    if let Err(err) = result {
        error!("oh, no! something went wrong: {err}");

        return Err(err);
    }

    Ok(())
}
