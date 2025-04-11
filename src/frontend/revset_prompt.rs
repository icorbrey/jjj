use bevy::prelude::*;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

use crate::backend::log::LogRevsetEvent;

use super::prelude::*;

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (read_keys.pipe(errors::forward))
            .in_set(AppSet::RecordInput)
            .run_if(is_focused::<RevsetPrompt>),
    );

    debug!("Finished loading");
}

#[derive(Component, Debug, Default)]
pub struct RevsetPrompt {
    input: String,
    read_input: bool,
}

#[tracing::instrument(skip_all)]
fn read_keys(
    mut ev_log_request: EventWriter<LogRevsetEvent>,
    mut revset_prompt: Query<&mut RevsetPrompt>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut navigation: Navigation,
) -> Result<()> {
    let mut revset_prompt = revset_prompt.get_single_mut()?;

    // Skip the first frame of input so we don't accidentally read the same
    // keypress that summoned the prompt.
    if !revset_prompt.read_input {
        revset_prompt.read_input = true;
        return Ok(());
    }

    for keypress in ev_keypresses.read() {
        debug!(?keypress.code, ?keypress.kind, revset_prompt.input);

        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        match keypress.code {
            KeyCode::Enter => {
                let revset = revset_prompt.input.clone();
                ev_log_request.send(LogRevsetEvent(revset));
                navigation.go_back()?;

                info!("Switched to revset: {}", revset_prompt.input);
            }
            KeyCode::Esc => {
                navigation.go_back()?;
            }
            KeyCode::Backspace => {
                revset_prompt.input.pop();
            }
            KeyCode::Char(ch) => {
                revset_prompt.input.push(ch);
            }
            _ => {}
        }
    }

    Ok(())
}

impl Widget for &RevsetPrompt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
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
        Paragraph::new(self.input.as_str())
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
