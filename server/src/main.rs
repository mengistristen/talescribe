use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use serde::Serialize;

const WORKING_FILE_PATH: &str = "./.talescribe/EDIT_ENTRY";

enum State {
    Idle,
    Entry { current_roll: Option<String> },
    Exit,
}

fn send_response<T: Serialize>(writer: &mut TcpStream, response: T) -> std::io::Result<()> {
    writeln!(writer, "{}", serde_json::to_string(&response)?)?;

    Ok(())
}

fn handle_idle(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if let Ok(message) = serde_json::from_str::<protocol::idle::Request>(&buf) {
        match message {
            protocol::idle::Request::Exit => {
                send_response(writer, protocol::idle::Response::Success)?;

                return Ok(State::Exit);
            }
            protocol::idle::Request::Entry => {
                send_response(writer, protocol::idle::Response::Success)?;

                return Ok(State::Entry { current_roll: None });
            }
        }
    }

    send_response(writer, protocol::idle::Response::InvalidCommand)?;

    Ok(State::Idle)
}

fn handle_entry(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    mut current_roll: Option<String>,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if let Ok(message) = serde_json::from_str::<protocol::entry::Request>(&buf) {
        match message {
            protocol::entry::Request::Get => {
                let content = if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::read_to_string(WORKING_FILE_PATH)?
                } else {
                    String::new()
                };

                send_response(writer, protocol::entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            protocol::entry::Request::Edit { content } => {
                std::fs::write(WORKING_FILE_PATH, content)?;

                current_roll = None;

                send_response(writer, protocol::entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            protocol::entry::Request::Roll => {
                let content = if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::read_to_string(WORKING_FILE_PATH)?
                } else {
                    String::new()
                };

                current_roll = Some(content.clone());

                send_response(writer, protocol::entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            protocol::entry::Request::Accept => {
                if let Some(content) = current_roll {
                    std::fs::write(WORKING_FILE_PATH, content)?;

                    current_roll = None;
                }

                send_response(writer, protocol::entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            protocol::entry::Request::Complete { name } => {
                if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::rename(WORKING_FILE_PATH, format!("./{name}.md"))?;
                }

                send_response(writer, protocol::entry::Response::Success)?;

                return Ok(State::Idle);
            }
        }
    }

    send_response(writer, protocol::entry::Response::InvalidCommand)?;

    Ok(State::Entry { current_roll })
}

fn main() -> std::io::Result<()> {
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
