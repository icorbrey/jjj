use anyhow::Result;
use bevy::prelude::*;

#[derive(Debug, Deref, Event, PartialEq)]
pub struct ErrorEvent(String);

impl<S: Into<String>> From<S> for ErrorEvent {
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

pub fn forward<T>(result: In<Result<T>>, mut ev_errors: EventWriter<ErrorEvent>) {
    if let Err(err) = result.0 {
        ev_errors.send(ErrorEvent::from(format!("{err}")));
        error!("{err}");
    }
}
#[cfg(test)]
mod tests {
    use anyhow::anyhow;

    use super::*;

    #[test]
    fn ignores_ok_results() {
        let mut app = App::new();

        fn success() -> Result<()> {
            Ok(())
        }

        app.add_event::<ErrorEvent>();
        app.add_systems(Update, success.pipe(forward));

        app.update();

        let error_events = app.world().resource::<Events<ErrorEvent>>();
        let mut error_reader = error_events.get_cursor();

        assert_eq!(error_reader.read(error_events).next(), None);
    }

    #[test]
    fn sends_events_for_err_results() {
        let mut app = App::new();

        let error = "Something went wrong.";
        let failure = move || Result::<()>::Err(anyhow!(error));

        app.add_event::<ErrorEvent>();
        app.add_systems(Update, failure.pipe(forward));

        app.update();

        let error_events = app.world().resource::<Events<ErrorEvent>>();
        let mut error_reader = error_events.get_cursor();

        assert_eq!(
            error_reader.read(error_events).next(),
            Some(&ErrorEvent(error.to_string()))
        );
    }
}
