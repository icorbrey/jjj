use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Wrap},
};

use crate::errors::ErrorEvent;

use super::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_scoped_type::<ErrorPopup>(Screen::Interface);

    app.add_systems(
        Update,
        (
            query_errors.in_set(AppSet::Update),
            check_inputs
                .in_set(AppSet::RecordInput)
                .run_if(in_state(Focus::ErrorPopup)),
        )
            .run_if(in_state(Screen::Interface)),
    );
}

fn query_errors(
    mut ev_errors: EventReader<ErrorEvent>,
    mut error_popup: ResMut<ErrorPopup>,
    mut focus: ResMut<NextState<Focus>>,
) {
    for error in ev_errors.read() {
        error_popup.errors.push(error.to_string());
        focus.set(Focus::ErrorPopup);
    }
}

fn check_inputs(
    mut ev_keypresses: EventReader<KeyEvent>,
    mut error_popup: ResMut<ErrorPopup>,
    mut focus: ResMut<NextState<Focus>>,
) {
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
            focus.set(Focus::ChangeBuffer);
        }
    }
}

#[derive(Default, Reflect, Resource)]
pub struct ErrorPopup {
    pub errors: Vec<String>,
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
            // .map(|e| {
            //     if max_err_length < e.len() as u16 {
            //         format!("{}...", &e[..(max_err_length - 3) as usize])
            //     } else {
            //         e.clone()
            //     }
            // })
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
