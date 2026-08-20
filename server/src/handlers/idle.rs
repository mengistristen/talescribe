use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
};

use crate::{State, handlers::send_response};

pub fn handle_idle(
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
