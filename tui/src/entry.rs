use std::process::Command;

use crossterm::event::{Event, KeyCode};
use indoc::indoc;
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, ListState, Paragraph},
};
use regex::regex;
use talescribe_protocol::entry;
use tempfile::NamedTempFile;

use crate::{App, Keybind, State, TuiError, build_help};

enum EntryAction {
    Edit,
    Complete,
    Back,
    Abort,
}

static ENTRY_KEYBINDS: &[Keybind<EntryAction>] = &[
    Keybind {
        code: KeyCode::Char('e'),
        action: EntryAction::Edit,
        name: "edit",
    },
    Keybind {
        code: KeyCode::Char('c'),
        action: EntryAction::Complete,
        name: "complete",
    },
    Keybind {
        code: KeyCode::Char('b'),
        action: EntryAction::Back,
        name: "back",
    },
    Keybind {
        code: KeyCode::Char('q'),
        action: EntryAction::Abort,
        name: "abort",
    },
];

pub fn draw_entry(frame: &mut Frame) {
    let area = frame.area();

    let help = build_help(ENTRY_KEYBINDS);
    let block = Block::bordered()
        .title(Line::from(" entry ").bold().left_aligned())
        .title_bottom(help);

    frame.render_widget(Paragraph::new("You're in entry.").block(block), area);
}

pub fn handle_entry_event(
    app: &mut App,
    event: &Event,
    terminal: &mut DefaultTerminal,
) -> anyhow::Result<State> {
    let Event::Key(key) = event else {
        return Ok(State::Entry);
    };

    if !key.kind.is_press() {
        return Ok(State::Entry);
    }

    let Some(keybind) = ENTRY_KEYBINDS.iter().find(|k| k.code == key.code) else {
        return Ok(State::Entry);
    };

    match keybind.action {
        EntryAction::Edit => {
            edit_entry(app, terminal)?;

            Ok(State::Entry)
        }
        EntryAction::Complete => todo!(),
        EntryAction::Back => {
            let r = serde_json::from_str(app.send(entry::Request::Back)?.as_ref())?;

            if matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Prompt {
                prompts: vec![],
                list_state: ListState::default(),
            })
        }
        EntryAction::Abort => {
            let r = serde_json::from_str(app.send(entry::Request::Abort)?.as_ref())?;

            if matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Idle)
        }
    }
}

fn edit_entry(_app: &mut App, terminal: &mut DefaultTerminal) -> anyhow::Result<String> {
    let file = NamedTempFile::with_suffix(".md")?;

    std::fs::write(
        file.path(),
        indoc! {"

        <!--
        holy cow, you're writing an entry
        -->
        "},
    )?;

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Command::new(std::env::var("EDITOR")?)
        .arg(file.path())
        .status()?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    terminal.clear()?;

    let content = std::fs::read_to_string(file.path())?;
    let re = regex!(r"(?s)<!--.*?-->");

    Ok(re.replace(&content, "").into())
}
