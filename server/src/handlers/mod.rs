use std::io::Write;
use std::net::TcpStream;

use serde::Serialize;

pub mod entry;
pub mod idle;
pub mod prompt;

pub fn send_response<T: Serialize>(writer: &mut TcpStream, response: T) -> std::io::Result<()> {
    writeln!(writer, "{}", serde_json::to_string(&response)?)?;

    Ok(())
}
