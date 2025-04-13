use bevy::prelude::*;
use prelude::*;

pub mod prelude {
    pub use crate::backend::log::{LogResponseEvent, LogRevsetEvent, RefreshLogEvent};
    pub use crate::errors::ErrorEvent;
    pub use crate::frontend::change_buffer::ChangeBufferSelectionEvent;
    pub use crate::frontend::command_line::NotificationEvent;
}

pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    // Application-wide events
    app.add_event::<ErrorEvent>();

    // Backend events
    app.add_event::<RefreshLogEvent>();
    app.add_event::<LogRevsetEvent>();
    app.add_event::<LogResponseEvent>();

    // Frontend events
    app.add_event::<ChangeBufferSelectionEvent>();
    app.add_event::<NotificationEvent>();

    trace!("Plugin initialized.");
}
