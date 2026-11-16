use std::process::Command;

use crossterm::event::{Event, KeyCode};
use indoc::formatdoc;
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, ListState, Paragraph},
};
use regex::regex;
use talescribe_protocol::{entry, prompt};
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

pub fn draw_entry(frame: &mut Frame, content: &str) {
    let area = frame.area();

    let help = build_help(ENTRY_KEYBINDS);
    let block = Block::bordered()
        .title(Line::from(" entry ").bold().left_aligned())
        .title_bottom(help);

    frame.render_widget(Paragraph::new(content).block(block), area);
}

pub fn handle_entry_event(
    app: &mut App,
    event: &Event,
    terminal: &mut DefaultTerminal,
    content: String,
    prompts: Vec<String>,
) -> anyhow::Result<State> {
    let Event::Key(key) = event else {
        return Ok(State::Entry { content, prompts });
    };

    if !key.kind.is_press() {
        return Ok(State::Entry { content, prompts });
    }

    let Some(keybind) = ENTRY_KEYBINDS.iter().find(|k| k.code == key.code) else {
        return Ok(State::Entry { content, prompts });
    };

    match keybind.action {
        EntryAction::Edit => {
            let content = launch_entry_editor(terminal, content.as_ref())?;

            let r = serde_json::from_str(app.send(entry::Request::Edit { content })?.as_ref())?;

            if !matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            let r = serde_json::from_str(app.send(entry::Request::Get)?.as_ref())?;

            let entry::Response::Content { content, prompts } = r else {
                return Err(TuiError::UnexpectedResponse.into());
            };

            Ok(State::Entry { content, prompts })
        }
        EntryAction::Complete => {
            let r = serde_json::from_str(
                app.send(entry::Request::Complete {
                    name: "asdf".to_string(),
                })?
                .as_ref(),
            )?;

            if !matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Idle)
        }
        EntryAction::Back => {
            let r = serde_json::from_str(app.send(entry::Request::Back)?.as_ref())?;

            if !matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            let r = serde_json::from_str(app.send(prompt::Request::Get)?.as_ref())?;

            let prompt::Response::Content { content } = r else {
                return Err(TuiError::UnexpectedResponse.into());
            };
            let mut list_state = ListState::default();

            if !content.is_empty() {
                list_state.select_first();
            }

            Ok(State::Prompt {
                prompts: content,
                list_state,
            })
        }
        EntryAction::Abort => {
            let r = serde_json::from_str(app.send(entry::Request::Abort)?.as_ref())?;

            if !matches!(r, entry::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Idle)
        }
    }
}

fn launch_entry_editor(terminal: &mut DefaultTerminal, content: &str) -> anyhow::Result<String> {
    let file = NamedTempFile::with_suffix(".md")?;

    std::fs::write(
        file.path(),
        formatdoc! {"
        {content}
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
