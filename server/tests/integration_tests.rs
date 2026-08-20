use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::{Child, Command, Stdio},
};

use serde::Serialize;
use talescribe_protocol::{entry, idle};
use tempfile::TempDir;

fn prepare() -> (TempDir, Child, TcpStream, BufReader<TcpStream>) {
    let dir = tempfile::tempdir().unwrap();

    Command::new(env!("CARGO_BIN_EXE_talescribe"))
        .arg("init")
        .current_dir(&dir)
        .status()
        .unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_talescribe"))
        .arg("run")
        .current_dir(&dir)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut port_line = String::new();

    BufReader::new(stdout).read_line(&mut port_line).unwrap();

    let port: u16 = port_line.trim().parse().unwrap();

    let stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
    let writer = stream.try_clone().unwrap();
    let reader = BufReader::new(stream);

    (dir, child, writer, reader)
}

fn send<T: Serialize>(
    stream: &mut TcpStream,
    reader: &mut BufReader<TcpStream>,
    message: T,
) -> std::io::Result<String> {
    let mut buf = String::new();

    stream.write_all(format!("{}\n", serde_json::to_string(&message)?).as_bytes())?;
    reader.read_line(&mut buf)?;

    Ok(buf)
}

fn exit(stream: &mut TcpStream) {
    stream
        .write_all(format!("{}\n", serde_json::to_string(&idle::Request::Exit).unwrap()).as_bytes())
        .unwrap();
}

#[test]
fn test_entry_flow() {
    let (_dir, mut child, mut writer, mut reader) = prepare();

    let r = serde_json::from_str(
        send(&mut writer, &mut reader, idle::Request::Entry)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    assert!(matches!(r, idle::Response::Success));

    let r = serde_json::from_str(
        send(&mut writer, &mut reader, entry::Request::Get)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Content {content } if content.is_empty()));

    let r = serde_json::from_str(
        send(
            &mut writer,
            &mut reader,
            entry::Request::Edit {
                content: "this is the entry".into(),
            },
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Success));

    let r = serde_json::from_str(
        send(&mut writer, &mut reader, entry::Request::Get)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Content { content } if content == "this is the entry"));

    let r = serde_json::from_str(
        send(&mut writer, &mut reader, entry::Request::Roll)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Content { content } if content == "this is the entry"));

    let r = serde_json::from_str(
        send(&mut writer, &mut reader, entry::Request::Accept)
            .unwrap()
            .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Success));

    let r = serde_json::from_str(
        send(
            &mut writer,
            &mut reader,
            entry::Request::Complete {
                name: "my_entry".into(),
            },
        )
        .unwrap()
        .as_str(),
    )
    .unwrap();
    assert!(matches!(r, entry::Response::Success));

    assert!(std::fs::exists(_dir.path().join("my_entry.md")).unwrap());

    exit(&mut writer);

    child.wait().unwrap();
}
