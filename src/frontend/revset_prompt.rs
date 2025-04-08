use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Rect, *},
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

use crate::backend::log::LogRequestEvent;

use super::prelude::*;

#[derive(Default, Event)]
pub struct OpenRevsetPromptEvent;

pub fn plugin(app: &mut App) {
    app.register_scoped_type::<RevsetPrompt>(Screen::Interface);

    app.add_systems(
        Update,
        (
            open_prompt.in_set(AppSet::Update),
            prompt_input
                .in_set(AppSet::RecordInput)
                .run_if(in_state(Focus::RevsetPrompt)),
        )
            .run_if(in_state(Screen::Interface)),
    );
}

#[derive(Default, Reflect, Resource)]
pub struct RevsetPrompt {
    input: Option<String>,
}

fn open_prompt(
    mut ev_open_prompt: EventReader<OpenRevsetPromptEvent>,
    mut revset_prompt: ResMut<RevsetPrompt>,
    mut focus: ResMut<NextState<Focus>>,
) {
    for _ in ev_open_prompt.read() {
        revset_prompt.input = Some(default());
        focus.set(Focus::RevsetPrompt);
        return;
    }
}

fn prompt_input(
    mut ev_log_request: EventWriter<LogRequestEvent>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut revset_prompt: ResMut<RevsetPrompt>,
    mut focus: ResMut<NextState<Focus>>,
) {
    for keypress in ev_keypresses.read() {
        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        match keypress.code {
            KeyCode::Enter => {
                if let Some(revset) = revset_prompt.input.clone() {
                    ev_log_request.send(LogRequestEvent { revset });
                    focus.set(Focus::ChangeBuffer);
                    revset_prompt.input = None;
                }
            }
            KeyCode::Esc => {
                focus.set(Focus::ChangeBuffer);
                revset_prompt.input = None;
            }
            KeyCode::Backspace => {
                if let Some(ref mut revset) = revset_prompt.input {
                    *revset = revset[..revset.len().saturating_sub(1)].to_owned();
                }
            }
            KeyCode::Char(ch) => {
                if let Some(ref mut revset) = revset_prompt.input {
                    *revset = format!("{revset}{ch}");
                }
            }
            _ => {}
        }
    }
}

impl Widget for &RevsetPrompt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let Some(ref input) = self.input else {
            return;
        };

        let [_, center, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(64),
                Constraint::Fill(1),
            ])
            .areas(area);

        let [_, prompt_area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .areas(center);

        Clear.render(prompt_area, buf);
        Paragraph::new(input.as_str())
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" Revset ")
                    .title_bottom(Span::from(" Enter / Esc ").into_right_aligned_line())
                    .padding(Padding::horizontal(1)),
            )
            .render(prompt_area, buf);
    }
}
