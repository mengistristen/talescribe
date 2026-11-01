use std::process::Command;

use crossterm::event::{Event, KeyCode};
use indoc::formatdoc;
use ratatui::{
    DefaultTerminal, Frame,
    style::{Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListState},
};
use regex::regex;
use talescribe_protocol::prompt::{self, Prompt};
use tempfile::NamedTempFile;

use crate::{App, Keybind, State, TuiError, build_help};

enum PromptAction {
    Add,
    Remove,
    Edit,
    Roll,
    StartEntry,
    Up,
    Down,
    Abort,
}

static PROMPT_KEYBINDS: &[Keybind<PromptAction>] = &[
    Keybind {
        code: KeyCode::Char('a'),
        action: PromptAction::Add,
        name: "add",
    },
    Keybind {
        code: KeyCode::Char('x'),
        action: PromptAction::Remove,
        name: "remove",
    },
    Keybind {
        code: KeyCode::Char('e'),
        action: PromptAction::Edit,
        name: "edit",
    },
    Keybind {
        code: KeyCode::Char('r'),
        action: PromptAction::Roll,
        name: "roll",
    },
    Keybind {
        code: KeyCode::Char('s'),
        action: PromptAction::StartEntry,
        name: "start entry",
    },
    Keybind {
        code: KeyCode::Char('k'),
        action: PromptAction::Up,
        name: "up",
    },
    Keybind {
        code: KeyCode::Char('j'),
        action: PromptAction::Down,
        name: "down",
    },
    Keybind {
        code: KeyCode::Char('q'),
        action: PromptAction::Abort,
        name: "abort",
    },
];

pub fn draw_prompt(frame: &mut Frame, prompts: &[Prompt], list_state: &mut ListState) {
    let area = frame.area();

    let help = build_help(PROMPT_KEYBINDS);
    let block = Block::bordered()
        .title(Line::from(" prompt ").bold().left_aligned())
        .title_bottom(help);

    let mut texts = vec![];

    for prompt in prompts {
        let mut lines = vec![];

        let content = if prompt.prompt.is_empty() {
            "<empty>".to_string()
        } else {
            prompt.prompt.clone()
        };
        lines.push(Line::from(vec![Span::styled(content, Style::default())]));

        if let Some(roll) = &prompt.roll {
            lines.push(Line::from(vec![Span::styled(
                roll,
                Style::default().fg(ratatui::style::Color::Cyan),
            )]));
        }

        texts.push(Text::from(lines));
    }

    let list = List::new(texts).highlight_symbol("> ");

    frame.render_stateful_widget(list.block(block), area, list_state);
}

pub fn handle_prompt_event(
    app: &mut App,
    event: &Event,
    terminal: &mut DefaultTerminal,
    prompts: Vec<Prompt>,
    mut list_state: ListState,
) -> anyhow::Result<State> {
    let Event::Key(key) = event else {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    };

    if !key.kind.is_press() {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    }

    let Some(keybind) = PROMPT_KEYBINDS.iter().find(|k| k.code == key.code) else {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    };

    match keybind.action {
        PromptAction::Add => add_prompt(app, list_state),
        PromptAction::Remove => remove_prompt(app, prompts, list_state),
        PromptAction::Edit => edit_prompt(app, terminal, prompts, list_state),
        PromptAction::Roll => roll_prompt(app, prompts, list_state),
        PromptAction::StartEntry => {
            let r = serde_json::from_str(app.send(prompt::Request::StartEntry)?.as_ref())?;

            if !matches!(r, prompt::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Entry)
        }
        PromptAction::Up => {
            list_state.select_previous();

            Ok(State::Prompt {
                prompts,
                list_state,
            })
        }
        PromptAction::Down => {
            list_state.select_next();

            Ok(State::Prompt {
                prompts,
                list_state,
            })
        }
        PromptAction::Abort => {
            let r = serde_json::from_str(app.send(prompt::Request::Abort)?.as_ref())?;

            if !matches!(r, prompt::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Idle)
        }
    }
}

fn add_prompt(app: &mut App, mut list_state: ListState) -> Result<State, anyhow::Error> {
    let mut r = serde_json::from_str(app.send(prompt::Request::Add)?.as_ref())?;

    if !matches!(r, prompt::Response::Success) {
        return Err(TuiError::UnexpectedResponse.into());
    }

    r = serde_json::from_str(app.send(prompt::Request::Get)?.as_ref())?;

    let prompt::Response::Content { content } = r else {
        return Err(TuiError::UnexpectedResponse.into());
    };

    list_state.select_last();

    Ok(State::Prompt {
        prompts: content,
        list_state,
    })
}

fn remove_prompt(
    app: &mut App,
    prompts: Vec<Prompt>,
    list_state: ListState,
) -> Result<State, anyhow::Error> {
    let Some(index) = list_state.selected() else {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    };

    let mut r = serde_json::from_str(app.send(prompt::Request::Remove { index })?.as_ref())?;

    if !matches!(r, prompt::Response::Success) {
        return Err(TuiError::UnexpectedResponse.into());
    }

    r = serde_json::from_str(app.send(prompt::Request::Get)?.as_ref())?;

    let prompt::Response::Content { content } = r else {
        return Err(TuiError::UnexpectedResponse.into());
    };

    Ok(State::Prompt {
        prompts: content,
        list_state,
    })
}

fn edit_prompt(
    app: &mut App,
    terminal: &mut DefaultTerminal,
    prompts: Vec<Prompt>,
    list_state: ListState,
) -> Result<State, anyhow::Error> {
    let Some(index) = list_state.selected() else {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    };
    let content = launch_prompt_editor(app, terminal, prompts[index].prompt.as_ref())?;

    let r = serde_json::from_str(app.send(prompt::Request::Edit { index, content })?.as_ref())?;

    if !matches!(r, prompt::Response::Success) {
        return Err(TuiError::UnexpectedResponse.into());
    }

    let r = serde_json::from_str(app.send(prompt::Request::Get)?.as_ref())?;

    let prompt::Response::Content { content } = r else {
        return Err(TuiError::UnexpectedResponse.into());
    };

    Ok(State::Prompt {
        prompts: content,
        list_state,
    })
}

fn roll_prompt(
    app: &mut App,
    prompts: Vec<Prompt>,
    list_state: ListState,
) -> Result<State, anyhow::Error> {
    let Some(index) = list_state.selected() else {
        return Ok(State::Prompt {
            prompts,
            list_state,
        });
    };

    let r = serde_json::from_str(app.send(prompt::Request::Roll { index })?.as_ref())?;

    if !matches!(r, prompt::Response::Success) {
        return Err(TuiError::UnexpectedResponse.into());
    }

    let r = serde_json::from_str(app.send(prompt::Request::Get)?.as_ref())?;

    let prompt::Response::Content { content } = r else {
        return Err(TuiError::UnexpectedResponse.into());
    };

    Ok(State::Prompt {
        prompts: content,
        list_state,
    })
}

fn launch_prompt_editor(
    _app: &mut App,
    terminal: &mut DefaultTerminal,
    content: &str,
) -> anyhow::Result<String> {
    let file = NamedTempFile::with_suffix(".md")?;

    std::fs::write(
        file.path(),
        formatdoc! {"
        {content}
        <!--
        holy cow, you're writing a prompt
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

    Ok(re.replace(&content, "").trim().to_string())
}
