use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, ListState, Paragraph},
};
use talescribe_protocol::idle;

use crate::{App, Keybind, State, TuiError, build_help};

enum IdleAction {
    New,
    Quit,
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

pub fn draw_idle(frame: &mut Frame) {
    let area = frame.area();

    let help = build_help(IDLE_KEYBINDS);
    let block = Block::bordered()
        .title(Line::from(" idle ").bold().left_aligned())
        .title_bottom(help);

    frame.render_widget(Paragraph::new("You're in idle.").block(block), area);
}

pub fn handle_idle_event(app: &mut App, event: &Event) -> anyhow::Result<State> {
    let Event::Key(key) = event else {
        return Ok(State::Idle);
    };

    if !key.kind.is_press() {
        return Ok(State::Idle);
    }

    let Some(keybind) = IDLE_KEYBINDS.iter().find(|k| k.code == key.code) else {
        return Ok(State::Idle);
    };

    match keybind.action {
        IdleAction::Quit => {
            app.send(idle::Request::Exit)?;
            app.exit = true;

            Ok(State::Idle)
        }
        IdleAction::New => {
            let r = serde_json::from_str(app.send(idle::Request::NewEntry)?.as_ref())?;

            if !matches!(r, idle::Response::Success) {
                return Err(TuiError::UnexpectedResponse.into());
            }

            Ok(State::Prompt {
                prompts: vec![],
                list_state: ListState::default(),
            })
        }
    }
}
