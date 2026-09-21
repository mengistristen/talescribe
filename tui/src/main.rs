use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    process::Command,
};

use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use serde::Serialize;
use talescribe_protocol::{entry, idle};
use tempfile::NamedTempFile;

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Entry,
}

enum IdleAction {
    New,
    Quit,
}

enum EntryAction {
    Edit,
    Roll,
    Accept,
    Complete,
}

struct App {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    exit: bool,
    state: State,
}

struct Keybind<T> {
    code: KeyCode,
    action: T,
    name: &'static str,
}

static IDLE_KEYBINDS: &[Keybind<IdleAction>] = &[
    Keybind {
        code: KeyCode::Char('n'),
        action: IdleAction::New,
        name: "new",
    },
    Keybind {
        code: KeyCode::Char('q'),
        action: IdleAction::Quit,
        name: "quit",
    },
];

static ENTRY_KEYBINDS: &[Keybind<EntryAction>] = &[
    Keybind {
        code: KeyCode::Char('e'),
        action: EntryAction::Edit,
        name: "edit",
    },
    Keybind {
        code: KeyCode::Char('r'),
        action: EntryAction::Roll,
        name: "roll",
    },
    Keybind {
        code: KeyCode::Char('a'),
        action: EntryAction::Accept,
        name: "accept",
    },
    Keybind {
        code: KeyCode::Char('c'),
        action: EntryAction::Complete,
        name: "complete",
    },
];

fn build_help<A>(keybinds: &[Keybind<A>]) -> Line<'static> {
    let mut spans = Vec::new();

    for (index, keybind) in keybinds.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" ⋅"));
        }

        spans.push(Span::raw(format!(" {} ", keybind.name)));
        spans.push(keybind.code.to_string().yellow().bold());
    }

    spans.push(Span::raw(" "));

    Line::from(spans)
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

        let help = build_help(IDLE_KEYBINDS);
        let block = Block::bordered()
            .title(Line::from(" idle ").bold().left_aligned())
            .title_bottom(help);

        frame.render_widget(Paragraph::new("You're in idle.").block(block), area);
    }

    fn draw_entry(&self, frame: &mut Frame) {
        let area = frame.area();

        let help = build_help(ENTRY_KEYBINDS);
        let block = Block::bordered()
            .title(Line::from(" entry ").bold().left_aligned())
            .title_bottom(help);

        frame.render_widget(Paragraph::new("You're in entry.").block(block), area);
    }

    fn handle_events(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let event = event::read()?;

        match self.state {
            State::Idle => self.handle_idle_event(&event),
            State::Entry => self.handle_entry_event(&event, terminal),
        }
    }

    fn handle_idle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        if let Event::Key(key) = event
            && key.kind.is_press()
            && let Some(keybind) = IDLE_KEYBINDS.iter().find(|k| k.code == key.code)
        {
            match keybind.action {
                IdleAction::Quit => {
                    self.send(idle::Request::Exit)?;
                    self.exit = true;
                }
                IdleAction::New => {
                    let r = serde_json::from_str(&self.send(idle::Request::Entry)?)?;

                    if matches!(r, idle::Response::Success) {
                        self.state = State::Entry;
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_entry_event(
        &mut self,
        event: &Event,
        terminal: &mut DefaultTerminal,
    ) -> anyhow::Result<()> {
        if let Event::Key(key) = event
            && key.kind.is_press()
            && let Some(keybind) = ENTRY_KEYBINDS.iter().find(|k| k.code == key.code)
        {
            match keybind.action {
                EntryAction::Edit => self.edit_entry(terminal)?,
                EntryAction::Roll => self.roll_entry()?,
                EntryAction::Accept => self.accept_entry()?,
                EntryAction::Complete => {
                    let r = serde_json::from_str(&self.send(entry::Request::Complete {
                        name: "asdf".into(),
                    })?)?;

                    if matches!(r, entry::Response::Success) {
                        self.state = State::Idle;
                    }
                }
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

    fn roll_entry(&mut self) -> anyhow::Result<()> {
        let r: entry::Response = serde_json::from_str(&self.send(entry::Request::Roll)?)?;

        if !matches!(r, entry::Response::Success) {
            // do something
        }

        Ok(())
    }

    fn accept_entry(&mut self) -> anyhow::Result<()> {
        let r: entry::Response = serde_json::from_str(&self.send(entry::Request::Accept)?)?;

        if !matches!(r, entry::Response::Success) {
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
