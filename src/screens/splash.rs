//! Rendering logic for the splash screen.

use std::time::Duration;

use bevy::prelude::*;
use bevy_ratatui::{event::KeyEvent, terminal::RatatuiContext};
use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Flex, Rect},
    prelude::*,
    widgets::{Clear, Paragraph},
};

use crate::app::AppSet;

use super::Screen;

/// Renders the splash screen when entering [`Screen::Splash`].
pub fn plugin(app: &mut App) {
    app.register_type::<SplashLineTimer>();
    app.register_type::<SplashCursor>();
    app.register_type::<SplashTimer>();

    app.add_systems(
        OnEnter(Screen::Splash),
        (SplashLineTimer::init, SplashCursor::init, SplashTimer::init),
    );

    app.add_systems(
        Update,
        (
            (SplashLineTimer::tick, SplashTimer::tick).in_set(AppSet::TickTimers),
            skip_to_interface.in_set(AppSet::RecordInput),
            (
                SplashTimer::continue_to_interface,
                SplashLineTimer::advance_cursor,
            )
                .in_set(AppSet::Update),
            draw.map(bevy::utils::error).in_set(AppSet::Render),
        )
            .run_if(in_state(Screen::Splash)),
    );

    app.add_systems(
        OnExit(Screen::Splash),
        (
            SplashLineTimer::remove,
            SplashCursor::remove,
            SplashTimer::remove,
        ),
    );
}

/// The total max duration the splash screen should be displayed for.
const SPLASH_DURATION: Duration = Duration::from_millis(1950);

/// The duration between splash screen animation frames.
const SPLASH_LINE_INTERVAL: Duration = Duration::from_millis(150);

/// The splash screen.
const SPLASH_LINES: [&'static str; 8] = [
    "███████╗███████╗███████╗",
    "╚════██║╚════██║╚════██║",
    "     ██║     ██║     ██║",
    "██╗  ██║██╗  ██║██╗  ██║",
    "╚█████╔╝╚█████╔╝╚█████╔╝",
    " ╚════╝  ╚════╝  ╚════╝ ",
    "------------------------",
    "       Jujutsu VCS      ",
];

/// Draws the current animation frame of the splash screen.
fn draw(mut context: ResMut<RatatuiContext>, cursor: Res<SplashCursor>) -> Result<()> {
    context.draw(|frame| {
        let area = center(frame.area(), Constraint::Length(24), Constraint::Length(8));
        let splash = Paragraph::new(
            SPLASH_LINES[..cursor.0]
                .iter()
                .map(|l| Line::from(*l))
                .collect::<Vec<Line>>(),
        );

        frame.render_widget(Clear, area);
        frame.render_widget(splash, area);
    })?;

    Ok(())
}

/// Returns a centered frame within the given area to render to.
fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

/// Skips the splash screen if the user presses the space bar.
fn skip_to_interface(
    mut events: EventReader<KeyEvent>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    for event in events.read() {
        if let KeyCode::Char(' ') = event.code {
            next_screen.set(Screen::Interface);
        }
    }
}

/// Controls the progress of the splash screen animation.
#[derive(Default, Deref, DerefMut, Reflect, Resource)]
struct SplashCursor(usize);

impl SplashCursor {
    fn init(mut commands: Commands) {
        commands.init_resource::<Self>();
    }

    fn advance(&mut self) {
        self.0 = usize::min(self.0 + 1, SPLASH_LINES.len());
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }
}

/// Controls timing for the overall splash screen.
#[derive(Deref, DerefMut, Reflect, Resource)]
struct SplashTimer(Timer);

impl Default for SplashTimer {
    fn default() -> Self {
        Self(Timer::new(SPLASH_DURATION, TimerMode::Once))
    }
}

impl SplashTimer {
    fn init(mut commands: Commands) {
        commands.init_resource::<Self>();
    }

    fn tick(time: Res<Time>, mut timer: ResMut<Self>) {
        timer.tick(time.delta());
    }

    fn continue_to_interface(timer: Res<Self>, mut next_screen: ResMut<NextState<Screen>>) {
        if timer.finished() {
            next_screen.set(Screen::Interface);
        }
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }
}

/// Controls timing for the splash screen animation.
#[derive(Deref, DerefMut, Reflect, Resource)]
struct SplashLineTimer(Timer);

impl Default for SplashLineTimer {
    fn default() -> Self {
        Self(Timer::new(SPLASH_LINE_INTERVAL, TimerMode::Repeating))
    }
}

impl SplashLineTimer {
    fn init(mut commands: Commands) {
        commands.init_resource::<Self>();
    }

    fn tick(time: Res<Time>, mut timer: ResMut<Self>) {
        timer.tick(time.delta());
    }

    fn advance_cursor(timer: Res<Self>, mut cursor: ResMut<SplashCursor>) {
        if timer.finished() {
            cursor.advance()
        }
    }

    fn remove(mut commands: Commands) {
        commands.remove_resource::<Self>();
    }
}
