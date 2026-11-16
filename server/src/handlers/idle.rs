use std::net::TcpStream;

use talescribe_protocol::idle;

use crate::{State, handlers::send_response};

pub fn handle_idle(writer: &mut TcpStream, message: &str) -> std::io::Result<State> {
    if let Ok(request) = serde_json::from_str::<idle::Request>(message) {
        match request {
            idle::Request::Exit => {
                send_response(writer, idle::Response::Success)?;

                return Ok(State::Exit);
            }
            idle::Request::NewEntry => {
                send_response(writer, idle::Response::Success)?;

                return Ok(State::Prompt { prompts: vec![] });
            }
        }
    }

    send_response(writer, idle::Response::InvalidCommand)?;

    Ok(State::Idle)
}
