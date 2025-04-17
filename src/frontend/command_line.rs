use std::time::Duration;

use bevy::prelude::*;
use ratatui::prelude::{Rect, *};

use super::prelude::*;

#[derive(Deref, Event)]
pub struct NotificationEvent(pub Notification);

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.register_type::<NotificationTimeoutTimer>();
    app.init_resource::<NotificationTimeoutTimer>();

    app.add_systems(
        Update,
        (
            NotificationTimeoutTimer::tick.in_set(AppSet::TickTimers),
            (
                NotificationTimeoutTimer::dismiss_notification.pipe(errors::forward),
                listen_for_notifications.pipe(errors::forward),
            )
                .in_set(AppSet::Update),
        )
            .run_if(in_state(Screen::Interface)),
    );

    trace!("Plugin initialized.");
}

#[derive(Component, Default)]
pub struct CommandLine {
    pub active_notification: Option<Notification>,
}

#[tracing::instrument(skip_all)]
fn listen_for_notifications(
    mut notification_timeout: ResMut<NotificationTimeoutTimer>,
    mut notification_events: EventReader<NotificationEvent>,
    mut command_line: Query<&mut CommandLine>,
) -> Result<()> {
    let mut command_line = command_line.get_single_mut()?;

    for event in notification_events.read() {
        debug!("Received notification: {:?}", event.0);
        command_line.active_notification = Some(event.0.clone());
        notification_timeout.unpause();
        notification_timeout.reset();
    }

    Ok(())
}

const NOTIFICATION_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Deref, DerefMut, Reflect, Resource)]
struct NotificationTimeoutTimer(Timer);

impl NotificationTimeoutTimer {
    pub fn tick(mut timer: ResMut<Self>, time: Res<Time>) {
        timer.tick(time.delta());
    }

    #[tracing::instrument(skip_all)]
    pub fn dismiss_notification(
        mut command_line: Query<&mut CommandLine>,
        timer: Res<Self>,
    ) -> Result<()> {
        let mut command_line = command_line.get_single_mut()?;

        if timer.just_finished() {
            debug!("Dismissed notification.");
            command_line.active_notification = None;
        }

        Ok(())
    }
}

impl Default for NotificationTimeoutTimer {
    fn default() -> Self {
        let mut timer = Timer::new(NOTIFICATION_TIMEOUT, TimerMode::Once);
        timer.pause();
        Self(timer)
    }
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub angry: bool,
}

impl NotificationEvent {
    pub fn new(message: String) -> Self {
        Self(Notification {
            message,
            angry: false,
        })
    }

    pub fn angry(message: String) -> Self {
        Self(Notification {
            message,
            angry: true,
        })
    }
}

impl Widget for &CommandLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if let Some(notification) = &self.active_notification {
            let span = if notification.angry {
                Span::styled(
                    notification.message.clone(),
                    Style::default().fg(Color::Red),
                )
            } else {
                Span::from(notification.message.clone())
            };

            span.render(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn snapshot_command_line() {
        let command_line = CommandLine {
            active_notification: Some(Notification {
                message: "This is a test".into(),
                angry: true,
            }),
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&command_line, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
