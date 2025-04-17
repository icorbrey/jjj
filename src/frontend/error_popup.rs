use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::{errors::ErrorEvent, logger};

use super::prelude::*;

#[mutants::skip]
#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.add_systems(
        Update,
        (
            listen.in_set(AppSet::Update),
            read_keys
                .in_set(AppSet::RecordInput)
                .run_if(is_focused::<ErrorPopup>),
        ),
    );

    trace!("Plugin initialized.");
}

#[derive(Component, Debug, Default)]
pub struct ErrorPopup {
    pub errors: Vec<String>,
}

fn listen(
    mut error_popup: Query<&mut ErrorPopup>,
    mut ev_errors: EventReader<ErrorEvent>,
    mut navigation: Navigation,
) {
    for error in ev_errors.read() {
        let Ok(mut error_popup) = error_popup.get_single_mut() else {
            navigation
                .spawn_popup(ErrorPopup {
                    errors: vec![error.to_string()],
                })
                .unwrap();
            continue;
        };

        error_popup.errors.push(error.to_string());
    }
}

fn read_keys(
    mut ev_keypresses: EventReader<KeyEvent>,
    mut error_popup: Query<&mut ErrorPopup>,
    mut navigation: Navigation,
) {
    let mut error_popup = error_popup.get_single_mut().unwrap();

    for keypress in ev_keypresses.read() {
        match keypress.code {
            KeyCode::Esc => {
                error_popup.errors.clear();
            }
            KeyCode::Char(' ') => {
                error_popup.errors.remove(0);
            }
            _ => {}
        }

        if error_popup.errors.is_empty() {
            if let Err(e) = navigation.go_back() {
                logger::dump();
                panic!("{}", e);
            }
        }
    }
}

impl Widget for &ErrorPopup {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.errors.is_empty() {
            return;
        }

        // Accomodate longer errors if possible
        let max_err_length = self.errors.iter().map(|e| e.len()).max().unwrap_or(0) as u16;
        let width = (max_err_length + 4).clamp(30, (3 * area.width) / 4);

        let popup_title = Span::from(" Error ");

        let error_text = (self.errors.iter())
            .map(|e| Line::from(Span::styled(e, Style::new().red().italic())))
            .collect::<Vec<_>>();

        let popup_cta = Line::from(" Dismiss: <Space> / <Esc> ").right_aligned();

        let paragraph = Paragraph::new(error_text).wrap(Wrap { trim: false }).block(
            Block::default()
                .title(popup_title)
                .title_bottom(popup_cta)
                .padding(Padding::horizontal(1))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );

        // Fit multiple errors onto the screen as needed, to a certain extent
        let height = u16::min(paragraph.line_count(width) as u16, area.height / 2);

        let center = center(area, Constraint::Length(width), Constraint::Length(height));

        Clear.render(center, buf);
        paragraph.render(center, buf);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn snapshot_no_errors() {
        let popup = ErrorPopup::default();

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&popup, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn snapshot_one_error() {
        let popup = ErrorPopup {
            errors: vec!["error".into()],
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&popup, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn snapshot_many_errors() {
        let popup = ErrorPopup {
            errors: (0..20).map(|i| format!("error {i}")).collect(),
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&popup, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
