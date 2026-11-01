use regex::{Captures, regex};
use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use talescribe_protocol::prompt;

use crate::{Prompt, State, handlers::send_response, roll::parser::parse_roll};

pub fn handle_prompt(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
    mut prompts: Vec<Prompt>,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if let Ok(message) = serde_json::from_str::<prompt::Request>(&buf) {
        match message {
            prompt::Request::Get => {
                send_response(
                    writer,
                    prompt::Response::Content {
                        content: prompts.clone(),
                    },
                )?;

                return Ok(State::Prompt { prompts });
            }
            prompt::Request::Add => {
                prompts.push(Prompt {
                    prompt: String::new(),
                    roll: None,
                });

                send_response(writer, prompt::Response::Success)?;

                return Ok(State::Prompt { prompts });
            }
            prompt::Request::Remove { index } => {
                if index < prompts.len() {
                    prompts.remove(index);

                    send_response(writer, prompt::Response::Success)?;
                } else {
                    send_response(writer, prompt::Response::Missing)?;
                }

                return Ok(State::Prompt { prompts });
            }
            prompt::Request::Edit { index, content } => {
                if index < prompts.len() {
                    prompts[index] = Prompt {
                        prompt: content,
                        roll: None,
                    };

                    send_response(writer, prompt::Response::Success)?;
                } else {
                    send_response(writer, prompt::Response::Missing)?;
                }

                return Ok(State::Prompt { prompts });
            }
            prompt::Request::Roll { index } => {
                if index < prompts.len() {
                    let re = regex!(r"\{([^{}]*)\}");
                    let prompt = &prompts[index];

                    let roll = re.replace_all(&prompt.prompt, |c: &Captures| {
                        if let Ok(("", roll)) = parse_roll(&c[1])
                            && let Some(result) = roll.evaluate()
                        {
                            return result;
                        }

                        format!("{{{}}}", &c[1])
                    });

                    prompts[index].roll = Some(roll.to_string());

                    send_response(writer, prompt::Response::Success)?;
                } else {
                    send_response(writer, prompt::Response::Missing)?;
                }

                return Ok(State::Prompt { prompts });
            }
            prompt::Request::StartEntry => {
                let prompts = prompts
                    .into_iter()
                    .map(|prompt| {
                        if let Some(roll) = prompt.roll {
                            return roll;
                        }

                        prompt.prompt
                    })
                    .collect();

                send_response(writer, prompt::Response::Success)?;

                return Ok(State::Entry { prompts });
            }
            prompt::Request::Abort => {
                send_response(writer, prompt::Response::Success)?;

                return Ok(State::Idle);
            }
        }
    }

    send_response(writer, prompt::Response::InvalidCommand)?;

    Ok(State::Prompt { prompts })
}
