use bevy::prelude::*;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Clear, Padding, Paragraph},
};

use crate::backend::log::LogRevsetEvent;

use super::prelude::*;

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.add_systems(
        Update,
        (read_keys.pipe(errors::forward))
            .in_set(AppSet::RecordInput)
            .run_if(is_focused::<RevsetPrompt>),
    );

    trace!("Plugin initialized.");
}

#[derive(Component, Debug, Default)]
pub struct RevsetPrompt {
    input: String,
    read_input: bool,
}

#[tracing::instrument(skip_all)]
fn read_keys(
    mut ev_log_request: EventWriter<LogRevsetEvent>,
    mut revset_prompt: Query<&mut RevsetPrompt>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut navigation: Navigation,
) -> Result<()> {
    let mut revset_prompt = revset_prompt.get_single_mut()?;

    // Skip the first frame of input so we don't accidentally read the same
    // keypress that summoned the prompt.
    if !revset_prompt.read_input {
        revset_prompt.read_input = true;
        return Ok(());
    }

    for keypress in ev_keypresses.read() {
        debug!(?keypress.code, ?keypress.kind, revset_prompt.input);

        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        match keypress.code {
            KeyCode::Enter => {
                let revset = revset_prompt.input.clone();
                ev_log_request.send(LogRevsetEvent(revset));
                navigation.go_back()?;

                info!("Switched to revset: {}", revset_prompt.input);
            }
            KeyCode::Esc => {
                navigation.go_back()?;
            }
            KeyCode::Backspace => {
                revset_prompt.input.pop();
            }
            KeyCode::Char(ch) => {
                revset_prompt.input.push(ch);
            }
            _ => {}
        }
    }

    Ok(())
}

impl Widget for &RevsetPrompt {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let [_, center, _] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(64),
                Constraint::Fill(1),
            ])
            .areas(area);

        let [_, prompt_area, _] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(3),
                Constraint::Fill(1),
            ])
            .horizontal_margin(2)
            .areas(center);

        Clear.render(prompt_area, buf);
        Paragraph::new(self.input.as_str())
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" Revset ")
                    .title_bottom(Span::from(" Enter / Esc ").into_right_aligned_line())
                    .padding(Padding::horizontal(1)),
            )
            .render(prompt_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::RunSystemOnce;
    use crossterm::event::{KeyEventState, KeyModifiers};
    use insta::assert_snapshot;
    use ratatui::backend::TestBackend;

    use crate::events;

    use super::*;

    fn to_keypresses(s: &str) -> Vec<KeyEvent> {
        s.chars()
            .map(|ch| {
                KeyEvent(crossterm::event::KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    state: KeyEventState::NONE,
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char(ch),
                })
            })
            .collect()
    }

    #[test]
    fn read_keys() -> Result<()> {
        let mut app = App::new();

        app.add_plugins(events::plugin);
        app.add_event::<KeyEvent>();

        app.add_systems(Update, super::read_keys.pipe(errors::forward));

        (app.world_mut()).run_system_once(
            |mut commands: Commands, mut ev_keypress: EventWriter<KeyEvent>| {
                commands.spawn(RevsetPrompt::default());
                ev_keypress.send_batch(to_keypresses("r"));
            },
        )?;

        app.update();

        // Make sure nothing's input on the first frame
        (app.world_mut()).run_system_once(
            |prompt: Query<&RevsetPrompt>, mut ev_keypress: EventWriter<KeyEvent>| {
                let prompt = prompt.single();

                assert!(prompt.input.is_empty());
                assert!(prompt.read_input);

                ev_keypress.send_batch(to_keypresses("testa"));
            },
        )?;

        app.update();

        // Make sure text input works
        (app.world_mut()).run_system_once(
            |ev_log_revset: EventReader<LogRevsetEvent>,
             mut ev_keypress: EventWriter<KeyEvent>,
             prompt: Query<&RevsetPrompt>| {
                let prompt = prompt.single();

                assert_eq!(prompt.input, "testa".to_string());
                assert!(ev_log_revset.is_empty());

                ev_keypress.send(KeyEvent(crossterm::event::KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    state: KeyEventState::NONE,
                    kind: KeyEventKind::Press,
                    code: KeyCode::Backspace,
                }))
            },
        )?;

        app.update();

        // Make sure backspace works
        (app.world_mut()).run_system_once(
            |ev_log_revset: EventReader<LogRevsetEvent>,
             mut ev_keypress: EventWriter<KeyEvent>,
             prompt: Query<&RevsetPrompt>| {
                let prompt = prompt.single();

                assert_eq!(prompt.input, "test".to_string());
                assert!(ev_log_revset.is_empty());

                ev_keypress.send(KeyEvent(crossterm::event::KeyEvent {
                    modifiers: KeyModifiers::NONE,
                    state: KeyEventState::NONE,
                    kind: KeyEventKind::Press,
                    code: KeyCode::Enter,
                }))
            },
        )?;

        app.update();

        // Make sure submitting with enter works
        (app.world_mut()).run_system_once(
            |mut ev_log_revset: EventReader<LogRevsetEvent>, prompt: Query<&RevsetPrompt>| {
                let prompt = prompt.single();

                assert_eq!(prompt.input, "test".to_string());

                assert_eq!(ev_log_revset.len(), 1);
                for ev in ev_log_revset.read() {
                    assert_eq!(ev.0, "test".to_string());
                }
            },
        )?;

        Ok(())
    }

    #[test]
    fn snapshot_revset_prompt() {
        let prompt = RevsetPrompt {
            input: "this is a test".into(),
            read_input: true,
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&prompt, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
