use std::{env, time::Duration};

use bevy::prelude::*;
use lazy_static::lazy_static;

use crate::{
    app::AppSet,
    backend::log::{LogResponseEvent, RefreshLogEvent},
    screens::Screen,
    utils::AppExt,
};

lazy_static! {
    static ref POLL_INTERVAL_MS: String = env::var("POLL_INTERVAL_MS").unwrap_or("1000".into());
}

pub fn plugin(app: &mut App) {
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
}

#[derive(Deref, DerefMut, Reflect, Resource)]
struct PollTimer(Timer);

impl PollTimer {
    fn tick(mut poll_timer: ResMut<Self>, time: Res<Time>) {
        poll_timer.tick(time.delta());
    }

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
}

impl Default for PollTimer {
    fn default() -> Self {
        let interval = (POLL_INTERVAL_MS.parse::<u64>())
            .expect("POLL_INTERVAL_MS should have been a valid 64 bit integer");

        let duration = Duration::from_millis(interval);
        Self(Timer::new(duration, TimerMode::Repeating))
    }
}
