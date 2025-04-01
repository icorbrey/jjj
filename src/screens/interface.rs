//! Rendering logic for the general interface.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::{app::AppSet, components::version::Version};

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        draw.map(bevy::utils::error)
            .in_set(AppSet::Render)
            .run_if(in_state(Screen::Interface)),
    );
}

fn draw(mut context: ResMut<RatatuiContext>) -> Result<()> {
    context.draw(|frame| {
        let [_, version_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());

        frame.render_widget(Version, version_area);
    })?;

    Ok(())
}
