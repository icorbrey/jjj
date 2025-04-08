//! Rendering logic for the general interface.

use anyhow::Result;
use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use ratatui::prelude::*;
use ratatui::widgets::Clear;

use crate::app::AppSet;
use crate::events::prelude::*;
use crate::frontend::prelude::*;

use super::Screen;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Interface), get_log);

    app.add_systems(
        Update,
        draw.map(bevy::utils::error)
            .in_set(AppSet::Render)
            .run_if(in_state(Screen::Interface)),
    );
}

fn get_log(mut ev_read_log: EventWriter<LogRequestEvent>) {
    ev_read_log.send(LogRequestEvent::from(
        "present(@) | ancestors(immutable_heads().., 2) | present(trunk())",
    ));
}

fn draw(
    mut change_buffer: ResMut<ChangeBuffer>,
    mut context: ResMut<RatatuiContext>,
    revset_prompt: Res<RevsetPrompt>,
    error_popup: Res<ErrorPopup>,
    status_line: Res<StatusLine>,
    space_menu: Res<SpaceMenu>,
) -> Result<()> {
    context.draw(|frame| {
        frame.render_widget(Clear, frame.area());

        let [buffer_area, status_line_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(frame.area());

        frame.render_stateful_widget(
            change_buffer.clone(),
            buffer_area,
            &mut change_buffer.viewport_y,
        );

        frame.render_widget(status_line.into_inner(), status_line_area);

        // Render after everything else
        frame.render_widget(revset_prompt.into_inner(), frame.area());
        frame.render_widget(space_menu.into_inner(), frame.area());
        frame.render_widget(error_popup.into_inner(), frame.area());
    })?;

    Ok(())
}
