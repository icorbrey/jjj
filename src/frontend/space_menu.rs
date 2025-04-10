use bevy::prelude::*;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, Clear, Padding, Paragraph},
};

use super::{prelude::*, revset_prompt::RevsetPrompt};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (read_keys.pipe(errors::forward))
            .in_set(AppSet::RecordInput)
            .run_if(is_focused::<SpaceMenu>),
    );
}

#[derive(Component)]
pub struct SpaceMenu;

fn read_keys(mut ev_keypresses: EventReader<KeyEvent>, mut navigation: Navigation) -> Result<()> {
    for keypress in ev_keypresses.read() {
        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        match keypress.code {
            KeyCode::Char('r') => navigation.spawn_popup(RevsetPrompt::default())?,
            KeyCode::Esc => navigation.go_back()?,
            _ => {}
        };
    }

    Ok(())
}

impl Widget for &SpaceMenu {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let prompt = Paragraph::new(vec![Line::from("r  Select revset")])
            .block(Block::bordered().padding(Padding::horizontal(1)));

        let [_, row, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Length(2),
            ])
            .areas(area);

        let [_, prompt_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length((prompt.line_width() + 4) as u16),
            ])
            .areas(row);

        Clear.render(prompt_area, buf);
        prompt.render(prompt_area, buf);
    }
}
