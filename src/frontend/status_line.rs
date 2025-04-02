use bevy::prelude::*;
use ratatui::{
    prelude::{Rect, *},
    widgets::Block,
};

use crate::events::prelude::*;

use super::{change_buffer::RevisionSelection, prelude::*};

pub fn plugin(app: &mut App) {
    app.register_scoped_type::<StatusLine>(Screen::Interface);

    app.add_systems(
        Update,
        (monitor_logged_revset, monitor_selected_revset)
            .run_if(in_state(Screen::Interface))
            .in_set(AppSet::Update),
    );
}

#[derive(Default, Reflect, Resource)]
pub struct StatusLine {
    pub logged_revset: Option<String>,
    pub selected_revset: Option<RevisionSelection>,
}

impl StatusLine {
    fn revset(&self) -> String {
        self.logged_revset.clone().unwrap_or("-".into())
    }
}

fn monitor_logged_revset(
    mut ev_log_request: EventReader<LogRequestEvent>,
    mut status_line: ResMut<StatusLine>,
) {
    for LogRequestEvent { revset } in ev_log_request.read() {
        status_line.logged_revset = Some(revset.clone());
    }
}

fn monitor_selected_revset(
    mut ev_selection: EventReader<ChangeBufferSelectionEvent>,
    mut status_line: ResMut<StatusLine>,
) {
    for ChangeBufferSelectionEvent(selection) in ev_selection.read() {
        status_line.selected_revset = Some(selection.clone());
    }
}

impl Widget for &StatusLine {
    fn render(self, status_line_area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let selected_revset = match &self.selected_revset {
            Some(RevisionSelection::Single(rev)) => {
                let change_id = rev.change_id.into_parts();
                Line::from(vec![
                    Span::styled(change_id.head, Style::new().magenta()),
                    Span::styled(change_id.tail, Style::new().black()),
                ])
            }
            Some(RevisionSelection::Range(start, end)) => {
                let start_change_id = start.change_id.into_parts();
                let end_change_id = end.change_id.into_parts();
                Line::from(vec![
                    Span::styled(start_change_id.head, Style::new().magenta()),
                    Span::styled(start_change_id.tail, Style::new().black()),
                    Span::styled("..", Style::new().black()),
                    Span::styled(end_change_id.head, Style::new().magenta()),
                    Span::styled(end_change_id.tail, Style::new().black()),
                ])
            }
            None => "-".into(),
        };

        let logged_revset = Span::styled(self.revset(), Style::new().black());

        let [_, selected_revset_area, logged_revset_area] = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .spacing(2)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(selected_revset.width() as u16),
                Constraint::Length(self.revset().len().try_into().unwrap()),
            ])
            .areas(status_line_area);

        Block::default().on_white().render(status_line_area, buf);
        selected_revset.render(selected_revset_area, buf);
        logged_revset.render(logged_revset_area, buf);
    }
}
