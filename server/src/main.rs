use std::{io::BufReader, net::TcpListener};

use crate::handlers::{entry::handle_entry, idle::handle_idle};

mod handlers;

enum State {
    Idle,
    Entry { current_roll: Option<String> },
    Exit,
}

fn main() -> std::io::Result<()> {
    if !std::fs::exists("./.talescribe/")? {
        eprintln!("not a talescribe directory, run `talescribe init`");
        std::process::exit(1);
    }

    let listener = TcpListener::bind("127.0.0.1:0")?;

    println!("{}", listener.local_addr()?.port());

    let (stream, _) = listener.accept()?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    let mut state = State::Idle;

    loop {
        state = match state {
            State::Idle => handle_idle(&mut reader, &mut writer)?,
            State::Entry { current_roll } => handle_entry(&mut reader, &mut writer, current_roll)?,
            State::Exit => break,
        }
    }

    Ok(())
}
