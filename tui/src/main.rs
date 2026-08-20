use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::Command,
};

use crossterm::event::{self, Event};
use protocol::{entry, idle};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph},
};
use serde::Serialize;
use tempfile::NamedTempFile;

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Entry,
}

struct App {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    exit: bool,
    state: State,
}

impl App {
    pub fn new(stream: TcpStream) -> anyhow::Result<Self> {
        let writer = stream.try_clone()?;
        let reader = BufReader::new(stream);

        Ok(Self {
            writer,
            reader,
            exit: false,
            state: State::Idle,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(terminal)?;
        }

        Ok(())
    }

    fn send<T: Serialize>(&mut self, message: T) -> anyhow::Result<String> {
        let mut buf = String::new();

        self.writer
            .write_all(format!("{}\n", serde_json::to_string(&message)?).as_bytes())?;
        self.reader.read_line(&mut buf)?;

        Ok(buf)
    }

    fn draw(&self, frame: &mut Frame) {
        match self.state {
            State::Idle => self.draw_idle(frame),
            State::Entry => self.draw_entry(frame),
        }
    }

    fn draw_idle(&self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(Line::from(" idle ").bold().left_aligned())
            .border_set(border::THICK);

        frame.render_widget(Paragraph::new("You're in idle.").block(block), area);
    }

    fn draw_entry(&self, frame: &mut Frame) {
        let area = frame.area();

        let block = Block::bordered()
            .title(Line::from(" entry ").bold().left_aligned())
            .border_set(border::THICK);

        frame.render_widget(Paragraph::new("You're in entry.").block(block), area);
    }

    fn handle_events(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let event = event::read()?;

        match self.state {
            State::Idle => self.handle_idle_event(event),
            State::Entry => self.handle_entry_event(event, terminal),
        }
    }

    fn handle_idle_event(&mut self, event: Event) -> anyhow::Result<()> {
        if let Event::Key(key) = event
            && key.kind.is_press()
        {
            match key.code {
                event::KeyCode::Char('q') => {
                    self.send(idle::Request::Exit)?;
                    self.exit = true;
                }
                event::KeyCode::Char('e') => {
                    let r = serde_json::from_str(&self.send(idle::Request::Entry)?)?;

                    if matches!(r, idle::Response::Success) {
                        self.state = State::Entry;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn handle_entry_event(
        &mut self,
        event: Event,
        terminal: &mut DefaultTerminal,
    ) -> anyhow::Result<()> {
        if let Event::Key(key) = event
            && key.kind.is_press()
        {
            match key.code {
                event::KeyCode::Char('e') => self.edit_entry(terminal)?,
                event::KeyCode::Char('c') => {
                    let r = serde_json::from_str(&self.send(entry::Request::Complete {
                        name: "asdf".into(),
                    })?)?;

                    if matches!(r, entry::Response::Success) {
                        self.state = State::Idle;
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn edit_entry(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let file = NamedTempFile::new()?;

        let r: entry::Response = serde_json::from_str(&self.send(entry::Request::Get)?)?;

        let content = match r {
            entry::Response::Content { content } => content,
            _ => String::new(),
        };

        std::fs::write(file.path(), content)?;

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

        Command::new(std::env::var("EDITOR")?)
            .arg(file.path())
            .status()?;

        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

        terminal.clear()?;

        let content = std::fs::read_to_string(file.path())?;

        let r = serde_json::from_str(&self.send(entry::Request::Edit { content })?)?;

        if matches!(r, entry::Response::Success) {
            // do something
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", std::env::var("TALESCRIBE_PORT")?))?;
    let mut app = App::new(stream)?;

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
