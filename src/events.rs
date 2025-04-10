use bevy::prelude::*;
use prelude::*;

use crate::backend::log::RefreshLogEvent;

pub mod prelude {
    pub use crate::backend::log::{LogResponseEvent, LogRevsetEvent};
    pub use crate::errors::ErrorEvent;
    pub use crate::frontend::change_buffer::ChangeBufferSelectionEvent;
    pub use crate::frontend::revset_prompt::OpenRevsetPromptEvent;
    pub use crate::frontend::space_menu::OpenSpaceMenuEvent;
}

pub fn plugin(app: &mut App) {
    // Application-wide events
    app.add_event::<ErrorEvent>();

    // Backend events
    app.add_event::<RefreshLogEvent>();
    app.add_event::<LogRevsetEvent>();
    app.add_event::<LogResponseEvent>();

    // Frontend events
    app.add_event::<OpenRevsetPromptEvent>();
    app.add_event::<OpenSpaceMenuEvent>();
    app.add_event::<ChangeBufferSelectionEvent>();
}
