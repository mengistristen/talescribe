use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use talescribe_protocol::{entry, prompt::Prompt};

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

fn edit_entry(path: &str, content: String) -> std::io::Result<()> {
    std::fs::write(path, content)?;

    Ok(())
}

fn abort_entry(path: &str) -> std::io::Result<()> {
    if std::fs::exists(path)? {
        std::fs::remove_file(path)?;
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
    prompts: Vec<String>,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if let Ok(message) = serde_json::from_str::<entry::Request>(&buf) {
        match message {
            entry::Request::Get => {
                let content = get_entry(WORKING_FILE_PATH)?;

                send_response(
                    writer,
                    entry::Response::Content {
                        content,
                        prompts: prompts.clone(),
                    },
                )?;

                return Ok(State::Entry { prompts });
            }
            entry::Request::Edit { content } => {
                edit_entry(WORKING_FILE_PATH, content)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Entry { prompts });
            }
            entry::Request::Complete { name } => {
                complete_entry(WORKING_FILE_PATH, &name)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Idle);
            }
            entry::Request::Back => {
                let prompts = prompts
                    .into_iter()
                    .map(|prompt| Prompt { prompt, roll: None })
                    .collect();

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Prompt { prompts });
            }
            entry::Request::Abort => {
                abort_entry(WORKING_FILE_PATH)?;

                send_response(writer, entry::Response::Success)?;

                return Ok(State::Idle);
            }
        }
    }

    send_response(writer, entry::Response::InvalidCommand)?;

    Ok(State::Entry { prompts })
}
