use bevy::prelude::*;
use ratatui::{
    layout::{Flex, Rect},
    prelude::*,
};

pub mod change_buffer;
pub mod command_line;
pub mod empty_buffer;
pub mod error_popup;
pub mod navigation;
pub mod revset_prompt;
pub mod space_menu;
pub mod status_line;
pub mod viewport;

pub mod prelude {
    pub use super::change_buffer::ChangeBuffer;
    pub use super::command_line::CommandLine;
    pub use super::empty_buffer::EmptyBuffer;
    pub use super::error_popup::ErrorPopup;
    pub use super::navigation::{is_focused, Navigation};
    pub use super::revset_prompt::RevsetPrompt;
    pub use super::space_menu::SpaceMenu;
    pub use super::status_line::StatusLine;

    pub(super) use anyhow::Result;
    pub(super) use bevy_ratatui::event::KeyEvent;
    pub(super) use crossterm::event::KeyEventKind;

    pub(super) use super::center;
    pub(super) use super::viewport;
    pub(super) use crate::app::AppSet;
    pub(super) use crate::errors;
    pub(super) use crate::screens::Screen;
}

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.add_plugins((
        change_buffer::plugin,
        revset_prompt::plugin,
        command_line::plugin,
        error_popup::plugin,
        status_line::plugin,
        navigation::plugin,
        space_menu::plugin,
    ));

    debug!("Finished loading");
}

/// Returns a centered frame within the given area to render to.
pub(super) fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
