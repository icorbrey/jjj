use bevy::prelude::*;
use ratatui::prelude::{Rect, *};

use crate::screens::Screen;

use super::prelude::EmptyBuffer;

pub fn plugin(app: &mut App) {
    app.register_type::<ChangeBuffer>();

    app.add_systems(OnEnter(Screen::Interface), ChangeBuffer::init);
    app.add_systems(OnExit(Screen::Interface), ChangeBuffer::remove);
}

#[derive(Default, Reflect, Resource)]
pub struct ChangeBuffer;

impl ChangeBuffer {
    fn init(mut commands: Commands) {
        commands.init_resource::<Self>();
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }
}

impl Widget for &ChangeBuffer {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        EmptyBuffer.render(area, buf);
    }
}
