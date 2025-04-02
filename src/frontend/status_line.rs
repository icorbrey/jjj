use bevy::prelude::*;
use ratatui::{
    prelude::{Rect, *},
    widgets::Block,
};

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.register_type::<StatusLine>();

    app.add_systems(OnEnter(Screen::Interface), StatusLine::init);
    app.add_systems(OnExit(Screen::Interface), StatusLine::remove);
}

#[derive(Default, Reflect, Resource)]
pub struct StatusLine {
    pub revset: Option<String>,
}

impl StatusLine {
    fn init(mut commands: Commands) {
        commands.init_resource::<Self>();
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }

    fn revset(&self) -> String {
        self.revset.clone().unwrap_or("-".into())
    }
}

impl Widget for &StatusLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [_, refset_area] = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(self.revset().len().try_into().unwrap()),
            ])
            .areas(area);

        Block::default().on_white().render(area, buf);
        Span::styled(self.revset(), Style::new().black()).render(refset_area, buf);
    }
}
