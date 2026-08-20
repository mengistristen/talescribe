use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

enum State {
    Idle,
    Other,
    Exit,
}

fn handle_idle(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if buf == "other\n" {
        writeln!(writer, "going to the other state")?;

        return Ok(State::Other);
    }

    if buf == "exit\n" {
        writeln!(writer, "exiting")?;

        return Ok(State::Exit);
    }

    writeln!(writer, "got it")?;

    Ok(State::Idle)
}

fn handle_other(
    reader: &mut BufReader<TcpStream>,
    writer: &mut TcpStream,
) -> std::io::Result<State> {
    let mut buf = String::new();

    reader.read_line(&mut buf)?;

    if buf == "back\n" {
        writeln!(writer, "going back to idle")?;

        return Ok(State::Idle);
    }

    writeln!(writer, "other got it")?;

    Ok(State::Other)
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
            State::Other => handle_other(&mut reader, &mut writer)?,
            State::Exit => break,
        }
    }

    Ok(())
}
