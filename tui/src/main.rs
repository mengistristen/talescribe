use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

use crossterm::event::{self, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::{Line, Span},
    widgets::ListState,
};
use serde::Serialize;
use talescribe_protocol::prompt::Prompt;

use crate::{
    entry::{draw_entry, handle_entry_event},
    idle::{draw_idle, handle_idle_event},
    prompt::{draw_prompt, handle_prompt_event},
};

mod entry;
mod idle;
mod prompt;

#[derive(Default)]
enum State {
    #[default]
    Idle,
    Prompt {
        prompts: Vec<Prompt>,
        list_state: ListState,
    },
    Entry,
}

struct App {
    writer: TcpStream,
    reader: BufReader<TcpStream>,
    exit: bool,
}

struct Keybind<T> {
    code: KeyCode,
    action: T,
    name: &'static str,
}

#[derive(thiserror::Error, Debug)]
enum TuiError {
    #[error("unexpected response")]
    UnexpectedResponse,
}

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
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        let mut state = State::Idle;

        while !self.exit {
            terminal.draw(|frame| Self::draw(&mut state, frame))?;
            state = self.handle_events(state, terminal)?;
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

    fn draw(state: &mut State, frame: &mut Frame) {
        match state {
            State::Idle => draw_idle(frame),
            State::Prompt {
                prompts,
                list_state,
            } => draw_prompt(frame, prompts, list_state),
            State::Entry => draw_entry(frame),
        }
    }

    fn handle_events(
        &mut self,
        state: State,
        terminal: &mut DefaultTerminal,
    ) -> anyhow::Result<State> {
        let event = event::read()?;

        match state {
            State::Idle => handle_idle_event(self, &event),
            State::Prompt {
                prompts,
                list_state,
            } => handle_prompt_event(self, &event, terminal, prompts, list_state),
            State::Entry => handle_entry_event(self, &event, terminal),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stream = TcpStream::connect(format!("127.0.0.1:{}", std::env::var("TALESCRIBE_PORT")?))?;
    let mut app = App::new(stream)?;

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
