use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_state::<Focus>();
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Focus {
    #[default]
    ChangeBuffer,
    ErrorPopup,
    RevsetPrompt,
    SpaceMenu,
    StatusLine,
}
