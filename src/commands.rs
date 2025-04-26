use bevy::prelude::*;
use serde::Deserialize;

pub fn plugin(_app: &mut App) {}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Event, PartialEq)]
#[serde(untagged)]
pub enum Command {
    Static(StaticCommand),
    // TODO: typable commands
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StaticCommand {
    /// Selects the line below the current selection.
    MoveVisualLineDown,

    /// Selects the line above the current selection.
    MoveVisualLineUp,

    /// Extends the current selection to include the next line down.
    ExtendLineDown,

    /// Extends the current selection to include the next line up.
    ExtendLineUp,

    /// Does nothing. Allows to delete a binding.
    #[serde(alias = "no_op")]
    #[serde(alias = "noop")]
    NoOp,
}
