use std::io::Write;
use std::net::TcpStream;

use log::info;
use serde::Serialize;

pub mod entry;
pub mod idle;
pub mod prompt;

pub fn send_response<T: Serialize>(writer: &mut TcpStream, response: T) -> std::io::Result<()> {
    let message = serde_json::to_string(&response)?;

    info!("outgoing --> {message}");
    writeln!(writer, "{message}")?;

    Ok(())
}
