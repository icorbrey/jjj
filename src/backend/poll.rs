use std::time::Duration;

use bevy::prelude::*;

use crate::{
    app::AppSet,
    backend::log::{LogResponseEvent, RefreshLogEvent},
    screens::Screen,
    utils::AppExt,
};

use super::config::Config;

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.register_scoped_type::<PollTimer>(Screen::Interface);
    app.add_systems(
        Update,
        (
            PollTimer::tick.in_set(AppSet::TickTimers),
            (PollTimer::debounce, PollTimer::check)
                .chain()
                .in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Interface)),
    );

    trace!("Plugin initialized.");
}

#[derive(Deref, DerefMut, Reflect, Resource)]
struct PollTimer(Timer);

impl FromWorld for PollTimer {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Config>();
        let duration = Duration::from_millis(config.log.poll_interval_ms);
        Self(Timer::new(duration, TimerMode::Repeating))
    }
}

impl PollTimer {
    fn tick(mut poll_timer: ResMut<Self>, time: Res<Time>) {
        poll_timer.tick(time.delta());
    }

    #[tracing::instrument(skip_all)]
    fn check(poll_timer: Res<Self>, mut ev_refresh_log: EventWriter<RefreshLogEvent>) {
        if poll_timer.just_finished() {
            ev_refresh_log.send(default());
        }
    }

    fn debounce(mut poll_timer: ResMut<Self>, mut ev_log_response: EventReader<LogResponseEvent>) {
        if ev_log_response.read().next().is_some() {
            poll_timer.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bevy::ecs::system::RunSystemOnce;

    use crate::{backend::config::LogConfig, events};

    use super::*;

    fn plugin(app: &mut App) {
        app.add_plugins(events::plugin);
        app.insert_resource(Config {
            log: LogConfig {
                poll_interval_ms: 50,
            },
            ..Config::new()
        });

        app.init_resource::<Time>();
        app.init_resource::<PollTimer>();
    }

    #[test]
    fn uses_config_interval() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(plugin);

        (app.world_mut()).run_system_once(|timer: Res<PollTimer>| {
            assert_eq!(timer.duration(), Duration::from_millis(50));
        })?;

        Ok(())
    }

    #[test]
    fn sends_refresh_events() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(plugin);
        app.add_systems(Update, (PollTimer::tick, PollTimer::check).chain());

        app.update();

        // No time has passed, make sure nothing gets produced.
        (app.world_mut()).run_system_once(
            |ev_refresh_log: EventReader<RefreshLogEvent>, mut time: ResMut<Time>| {
                assert!(ev_refresh_log.is_empty());

                time.advance_by(Duration::from_millis(26));
            },
        )?;

        app.update();

        // Not enough time has passed, don't produce anything yet.
        (app.world_mut()).run_system_once(
            |ev_refresh_log: EventReader<RefreshLogEvent>, mut time: ResMut<Time>| {
                assert!(ev_refresh_log.is_empty());

                time.advance_by(Duration::from_millis(26));
            },
        )?;

        app.update();

        // Enough time has passed to fire once.
        (app.world_mut()).run_system_once(
            |ev_refresh_log: EventReader<RefreshLogEvent>, mut time: ResMut<Time>| {
                assert_eq!(ev_refresh_log.len(), 1);

                time.advance_by(Duration::from_millis(50));
            },
        )?;

        app.update();

        // Enough time has passed to fire twice.
        (app.world_mut()).run_system_once(|ev_refresh_log: EventReader<RefreshLogEvent>| {
            assert_eq!(ev_refresh_log.len(), 2);
        })?;

        Ok(())
    }

    #[test]
    fn debounces() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(plugin);

        app.add_systems(
            Update,
            (PollTimer::tick, PollTimer::debounce, PollTimer::check).chain(),
        );

        (app.world_mut()).run_system_once(|mut time: ResMut<Time>| {
            time.advance_by(Duration::from_millis(49));
        })?;

        app.update();

        // Not enough time has passed, don't produce anything yet.
        (app.world_mut()).run_system_once(
            |mut ev_log_response: EventWriter<LogResponseEvent>,
             ev_refresh_log: EventReader<RefreshLogEvent>,
             mut time: ResMut<Time>| {
                assert!(ev_refresh_log.is_empty());

                time.advance_by(Duration::from_millis(49));
                ev_log_response.send(LogResponseEvent(vec![]));
            },
        )?;

        app.update();

        // Enough time has passed, but we just saw a response so don't request another one yet
        (app.world_mut()).run_system_once(
            |ev_refresh_log: EventReader<RefreshLogEvent>, mut time: ResMut<Time>| {
                assert!(ev_refresh_log.is_empty());

                time.advance_by(Duration::from_millis(51));
            },
        )?;

        app.update();

        // Enough time has passed and no response was seen, so request a refresh
        (app.world_mut()).run_system_once(|ev_refresh_log: EventReader<RefreshLogEvent>| {
            assert_eq!(ev_refresh_log.len(), 1);
        })?;

        Ok(())
    }
}
