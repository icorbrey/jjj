use bevy::prelude::*;
use prelude::*;

pub mod prelude {
    pub use crate::backend::log::{LogRequestEvent, LogResponseEvent};
    pub use crate::errors::ErrorEvent;
    pub use crate::frontend::change_buffer::{ChangeBufferSelectionEvent, PurgeChangeBufferEvent};
}

pub fn plugin(app: &mut App) {
    // Application-wide events
    app.add_event::<ErrorEvent>();

    // Backend events
    app.add_event::<LogRequestEvent>();
    app.add_event::<LogResponseEvent>();

    // Frontend events
    app.add_event::<PurgeChangeBufferEvent>();
    app.add_event::<ChangeBufferSelectionEvent>();
}
