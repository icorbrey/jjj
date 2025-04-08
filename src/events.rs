use bevy::prelude::*;
use prelude::*;

pub mod prelude {
    pub use crate::backend::log::{LogRequestEvent, LogResponseEvent};
    pub use crate::errors::ErrorEvent;
    pub use crate::frontend::change_buffer::{ChangeBufferSelectionEvent, PurgeChangeBufferEvent};
    pub use crate::frontend::revset_prompt::OpenRevsetPromptEvent;
    pub use crate::frontend::space_menu::OpenSpaceMenuEvent;
}

pub fn plugin(app: &mut App) {
    // Application-wide events
    app.add_event::<ErrorEvent>();

    // Backend events
    app.add_event::<LogRequestEvent>();
    app.add_event::<LogResponseEvent>();

    // Frontend events
    app.add_event::<OpenRevsetPromptEvent>();
    app.add_event::<OpenSpaceMenuEvent>();
    app.add_event::<PurgeChangeBufferEvent>();
    app.add_event::<ChangeBufferSelectionEvent>();
}
