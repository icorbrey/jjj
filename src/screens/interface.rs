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
    app.add_systems(OnEnter(Screen::Interface), init);

    app.add_systems(
        Update,
        draw.map(bevy::utils::error)
            .in_set(AppSet::Render)
            .run_if(in_state(Screen::Interface)),
    );
}

fn init(
    mut ev_read_log: EventWriter<LogRevsetEvent>,
    mut navigation: Navigation,
    mut commands: Commands,
) {
    let root = commands.spawn(ChangeBuffer::default()).id();
    commands.spawn(StatusLine::default());

    navigation.focus_as_root(root);

    ev_read_log.send(LogRevsetEvent(
        "present(@) | ancestors(immutable_heads().., 2) | present(trunk())".to_string(),
    ));
}

fn draw(
    mut change_buffer: Query<&mut ChangeBuffer>,
    mut context: ResMut<RatatuiContext>,
    revset_prompt: Query<&RevsetPrompt>,
    error_popup: Query<&ErrorPopup>,
    status_line: Query<&StatusLine>,
    space_menu: Query<&SpaceMenu>,
) -> Result<()> {
    let mut change_buffer = change_buffer.get_single_mut()?;
    let status_line = status_line.get_single()?;

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

        frame.render_widget(status_line, status_line_area);

        if let Ok(revset_prompt) = revset_prompt.get_single() {
            frame.render_widget(revset_prompt, buffer_area);
        }

        if let Ok(space_menu) = space_menu.get_single() {
            frame.render_widget(space_menu, buffer_area);
        }

        if let Ok(error_popup) = error_popup.get_single() {
            frame.render_widget(error_popup, buffer_area);
        }
    })?;

    Ok(())
}
