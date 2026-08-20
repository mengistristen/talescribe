use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use talescribe_protocol::entry;

use crate::{State, handlers::send_response};

const WORKING_FILE_PATH: &str = "./.talescribe/EDIT_ENTRY";

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
                let content = if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::read_to_string(WORKING_FILE_PATH)?
                } else {
                    String::new()
                };

                send_response(writer, entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Edit { content } => {
                std::fs::write(WORKING_FILE_PATH, content)?;

                current_roll = None;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Roll => {
                let content = if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::read_to_string(WORKING_FILE_PATH)?
                } else {
                    String::new()
                };

                current_roll = Some(content.clone());

                send_response(writer, entry::Response::Content { content })?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Accept => {
                if let Some(content) = current_roll {
                    std::fs::write(WORKING_FILE_PATH, content)?;

                    current_roll = None;
                }

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Entry { current_roll });
            }
            entry::Request::Complete { name } => {
                if std::fs::exists(WORKING_FILE_PATH)? {
                    std::fs::rename(WORKING_FILE_PATH, format!("./{name}.md"))?;
                }

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Idle);
            }
        }
    }

    send_response(writer, entry::Response::InvalidCommand)?;

    Ok(State::Entry { current_roll })
}
