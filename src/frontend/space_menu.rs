use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Rect, *},
    widgets::{Block, Clear, Padding, Paragraph},
};

use super::prelude::*;
use crate::events::prelude::*;

#[derive(Default, Event)]
pub struct OpenSpaceMenuEvent;

pub fn plugin(app: &mut App) {
    app.register_scoped_type::<SpaceMenu>(Screen::Interface);

    app.add_systems(
        Update,
        (
            open_menu.in_set(AppSet::Update),
            prompt_input
                .in_set(AppSet::RecordInput)
                .run_if(in_state(Focus::SpaceMenu)),
        )
            .run_if(in_state(Screen::Interface)),
    );
}

#[derive(Default, Reflect, Resource)]
pub struct SpaceMenu {
    is_visible: bool,
}

fn open_menu(
    mut ev_open_menu: EventReader<OpenSpaceMenuEvent>,
    mut focus: ResMut<NextState<Focus>>,
    mut space_menu: ResMut<SpaceMenu>,
) {
    for _ in ev_open_menu.read() {
        space_menu.is_visible = true;
        focus.set(Focus::SpaceMenu);
        return;
    }
}

fn prompt_input(
    mut ev_revset_prompt: EventWriter<OpenRevsetPromptEvent>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut focus: ResMut<NextState<Focus>>,
    mut space_menu: ResMut<SpaceMenu>,
) {
    for keypress in ev_keypresses.read() {
        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        match keypress.code {
            KeyCode::Esc => {
                space_menu.is_visible = false;
                focus.set(Focus::ChangeBuffer);
            }
            KeyCode::Char('r') => {
                space_menu.is_visible = false;
                ev_revset_prompt.send(default());
            }
            _ => {}
        };
    }
}

impl Widget for &SpaceMenu {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if !self.is_visible {
            return;
        }

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
