use anyhow::Result;
use bevy::prelude::*;

#[derive(Deref, Event)]
pub struct ErrorEvent(String);

impl<S: Into<String>> From<S> for ErrorEvent {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

pub fn forward<T>(result: In<Result<T>>, mut ev_errors: EventWriter<ErrorEvent>) {
    if let Err(err) = result.0 {
        ev_errors.send(ErrorEvent::from(format!("{err}")));
    }
}
