use std::time::Duration;

use bevy::prelude::*;

use crate::{
    app::AppSet,
    backend::log::{LogResponseEvent, RefreshLogEvent},
    screens::Screen,
};

use super::config::Config;

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    app.register_type::<PollTimer>();
    app.add_systems(OnEnter(Screen::Interface), PollTimer::init);
    app.add_systems(OnExit(Screen::Interface), PollTimer::remove);

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

    debug!("Finished loading");
}

#[derive(Deref, DerefMut, Reflect, Resource)]
struct PollTimer(Timer);

impl PollTimer {
    fn init(mut commands: Commands, config: Res<Config>) {
        commands.insert_resource(Self(Timer::new(
            Duration::from_millis(config.log.poll_interval_ms),
            TimerMode::Repeating,
        )));
    }

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
        for _ in ev_log_response.read() {
            poll_timer.reset();
            break;
        }
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }
}
