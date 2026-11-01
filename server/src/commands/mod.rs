use std::{io::BufReader, net::TcpListener, path::PathBuf, process::Command};

use crate::{
    State,
    handlers::{entry::handle_entry, idle::handle_idle, prompt::handle_prompt},
};

pub fn init() -> std::io::Result<()> {
    std::fs::create_dir("./.talescribe")?;

    println!("initialized new talescribe directory: .talescribe");

    Ok(())
}

pub fn run(client: Option<PathBuf>) -> std::io::Result<()> {
    if !std::fs::exists("./.talescribe/")? {
        eprintln!("not a talescribe directory, run `talescribe init`");
        std::process::exit(1);
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;

    let child = if let Some(client) = client {
        Some(
            Command::new(client)
                .env("TALESCRIBE_PORT", addr.port().to_string())
                .spawn()?,
        )
    } else {
        println!("{}", addr.port());

        None
    };

    let (stream, _) = listener.accept()?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let mut state = State::Idle;

    loop {
        state = match state {
            State::Idle => handle_idle(&mut reader, &mut writer)?,
            State::Prompt { prompts } => handle_prompt(&mut reader, &mut writer, prompts)?,
            State::Entry { prompts } => handle_entry(&mut reader, &mut writer, prompts)?,
            State::Exit => break,
        }
    }

    if let Some(mut child) = child {
        child.wait()?;
    }

    Ok(())
}
