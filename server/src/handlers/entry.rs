use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use talescribe_protocol::entry;

use crate::{State, handlers::send_response};

const WORKING_FILE_PATH: &str = "./.talescribe/EDIT_ENTRY";

fn get_entry(path: &str) -> std::io::Result<String> {
    let content = if std::fs::exists(path)? {
        std::fs::read_to_string(path)?
    } else {
        String::new()
    };

    Ok(content)
}

fn edit_entry(
    path: &str,
    content: String,
    current_roll: &mut Option<String>,
) -> std::io::Result<()> {
    std::fs::write(path, content)?;

    *current_roll = None;

    Ok(())
}

fn roll_entry(path: &str, current_roll: &mut Option<String>) -> std::io::Result<String> {
    let content = get_entry(path)?;

    *current_roll = Some(content.clone());

    Ok(content)
}

fn accept_entry(path: &str, current_roll: &mut Option<String>) -> std::io::Result<()> {
    if let Some(content) = current_roll {
        std::fs::write(path, content)?;

        *current_roll = None;
    }

    Ok(())
}

fn complete_entry(path: &str, name: &str) -> std::io::Result<()> {
    if std::fs::exists(path)? {
        std::fs::rename(path, format!("./{name}.md"))?;
    }

    Ok(())
}

pub fn handle_entry(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    mut current_roll: Option<String>,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if let Ok(message) = serde_json::from_str::<entry::Request>(&buf) {
        match message {
            entry::Request::Get => {
                let content = get_entry(WORKING_FILE_PATH)?;

                send_response(writer, entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Edit { content } => {
                edit_entry(WORKING_FILE_PATH, content, &mut current_roll)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Roll => {
                let content = roll_entry(WORKING_FILE_PATH, &mut current_roll)?;

                send_response(writer, entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Accept => {
                accept_entry(WORKING_FILE_PATH, &mut current_roll)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Complete { name } => {
                complete_entry(WORKING_FILE_PATH, &name)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Idle);
            }
        }
    }

    send_response(writer, entry::Response::InvalidCommand)?;

    Ok(State::Entry { current_roll })
}
