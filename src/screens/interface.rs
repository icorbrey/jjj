//! Rendering logic for the general interface.

use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use color_eyre::eyre::Result;
use ratatui::prelude::*;

use crate::app::AppSet;
use crate::frontend::prelude::*;

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        draw.map(bevy::utils::error)
            .in_set(AppSet::Render)
            .run_if(in_state(Screen::Interface)),
    );
}

fn draw(
    mut context: ResMut<RatatuiContext>,
    change_buffer: Res<ChangeBuffer>,
    status_line: Res<StatusLine>,
) -> Result<()> {
    context.draw(|frame| {
        let [buffer_area, status_line_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_widget(change_buffer.into_inner(), buffer_area);
        frame.render_widget(status_line.into_inner(), status_line_area);
    })?;

    Ok(())
}
