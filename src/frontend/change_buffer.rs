use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Rect, *},
    widgets::Block,
};

use crate::backend::{log::LogResponseEvent, revisions::Revision};

use super::prelude::*;

#[tracing::instrument(skip_all)]
pub fn plugin(app: &mut App) {
    trace!("Initializing plugin...");

    app.add_systems(
        Update,
        (
            (read_keys.pipe(errors::forward))
                .in_set(AppSet::RecordInput)
                .run_if(is_focused::<ChangeBuffer>),
            (read_revisions.pipe(errors::forward))
                .in_set(AppSet::Update)
                .run_if(in_state(Screen::Interface)),
        ),
    );

    trace!("Plugin initialized.");
}

#[derive(Event)]
pub struct ChangeBufferSelectionEvent(pub RevisionSelection);

#[derive(Clone, Component, Default)]
pub struct ChangeBuffer {
    revisions: Vec<Revision>,
    selection: IndexSelection,
    pub viewport_y: usize,
}

#[derive(Clone)]
pub enum IndexSelection {
    Single(usize),
    Range(usize, usize),
}

impl Default for IndexSelection {
    fn default() -> Self {
        IndexSelection::Single(0)
    }
}

#[derive(Clone)]
pub enum RevisionSelection {
    Single(Revision),
    Range(Revision, Revision),
}

fn read_keys(
    mut ev_selection: EventWriter<ChangeBufferSelectionEvent>,
    mut change_buffer: Query<&mut ChangeBuffer>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
    mut navigation: Navigation,
) -> Result<()> {
    let mut change_buffer = change_buffer.get_single_mut()?;

    if change_buffer.revisions.len() == 0 {
        return Ok(());
    }

    for keypress in ev_keypresses.read() {
        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        let max = change_buffer.revisions.len().saturating_sub(1);

        let selection = match (change_buffer.selection.clone(), keypress.code) {
            (_, KeyCode::Char(' ')) => {
                navigation.spawn_popup(SpaceMenu)?;
                return Ok(());
            }
            (_, KeyCode::Char('q')) => {
                exit.send_default();
                return Ok(());
            }
            (IndexSelection::Single(i), KeyCode::Char('j')) => {
                Some(IndexSelection::Single(usize::min(i + 1, max)))
            }
            (IndexSelection::Single(i), KeyCode::Char('k')) => {
                Some(IndexSelection::Single(i.saturating_sub(1)))
            }
            (IndexSelection::Single(i), KeyCode::Char('x')) => {
                if i != (i + 1).clamp(0, max) {
                    Some(IndexSelection::Range(i, usize::min(i + 1, max)))
                } else {
                    Some(IndexSelection::Single(i))
                }
            }
            (IndexSelection::Range(_, end), KeyCode::Char('j')) => {
                Some(IndexSelection::Single(usize::min(end + 1, max)))
            }
            (IndexSelection::Range(start, _), KeyCode::Char('k')) => {
                Some(IndexSelection::Single(start.saturating_sub(1)))
            }
            (IndexSelection::Range(start, end), KeyCode::Char('x')) => {
                Some(IndexSelection::Range(start, usize::min(end + 1, max)))
            }
            _ => None,
        };

        if let Some(selection) = selection {
            change_buffer.selection = selection.clone();
            ev_selection.send(match selection {
                IndexSelection::Single(i) => ChangeBufferSelectionEvent(RevisionSelection::Single(
                    change_buffer.revisions[i].clone(),
                )),
                IndexSelection::Range(start, end) => {
                    ChangeBufferSelectionEvent(RevisionSelection::Range(
                        change_buffer.revisions[start].clone(),
                        change_buffer.revisions[end].clone(),
                    ))
                }
            });
        }
    }

    Ok(())
}

fn read_revisions(
    mut ev_selection: EventWriter<ChangeBufferSelectionEvent>,
    mut ev_log_response: EventReader<LogResponseEvent>,
    mut change_buffer: Query<&mut ChangeBuffer>,
) -> Result<()> {
    let mut change_buffer = change_buffer.get_single_mut()?;

    for LogResponseEvent(revisions) in ev_log_response.read() {
        if change_buffer.revisions.is_empty() {
            change_buffer.selection = IndexSelection::Single(0);
            change_buffer.revisions = revisions.clone();
            continue;
        }

        match change_buffer.selection {
            // Select the same change by change_id if it exists
            IndexSelection::Single(i) => {
                let change_id = &change_buffer.revisions[i].change_id;
                let index = (revisions.iter())
                    .position(|r| r.change_id.0 == *change_id.0)
                    .unwrap_or(0);

                change_buffer.selection = IndexSelection::Single(index);
                change_buffer.revisions = revisions.clone();
            }

            // Select the range of commits with the same start and end change_id if they exist
            IndexSelection::Range(start_prev, end_prev) => {
                let start_change_id = &change_buffer.revisions[start_prev].change_id;
                let end_change_id = &change_buffer.revisions[end_prev].change_id;

                change_buffer.selection = (revisions.iter())
                    .position(|r| r.change_id.0 == *start_change_id.0)
                    .zip((revisions.iter()).position(|r| r.change_id.0 == *end_change_id.0))
                    .map(|(start, end)| IndexSelection::Range(start, end))
                    .unwrap_or(IndexSelection::Single(0));
                change_buffer.revisions = revisions.clone();
            }
        }

        ev_selection.send(match change_buffer.selection {
            IndexSelection::Single(i) => ChangeBufferSelectionEvent(RevisionSelection::Single(
                change_buffer.revisions[i].clone(),
            )),
            IndexSelection::Range(start, end) => {
                ChangeBufferSelectionEvent(RevisionSelection::Range(
                    change_buffer.revisions[start].clone(),
                    change_buffer.revisions[end].clone(),
                ))
            }
        });
    }

    Ok(())
}

impl StatefulWidget for ChangeBuffer {
    type State = usize;

    fn render(self, area: Rect, buf: &mut Buffer, viewport_y: &mut Self::State) {
        let (computed_viewport_y, rev_range) = viewport::compute_sliding_window(
            self.revisions.len(),
            match self.selection {
                IndexSelection::Single(index) => index,
                IndexSelection::Range(_, end) => end,
            },
            *viewport_y,
            area.height as usize,
            area.height as usize / 4,
        );

        *viewport_y = computed_viewport_y;
        let revs =
            self.revisions.iter().enumerate().collect::<Vec<_>>()[rev_range.clone()].to_vec();

        let [revs_area, empty] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(revs.len() as u16), Constraint::Fill(1)])
            .areas(area);

        let rev_lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rev_range.map(|_| Constraint::Length(1)))
            .split(revs_area);

        for ((i, revision), area) in revs.iter().zip(rev_lines.iter()) {
            let is_selected = match self.selection {
                IndexSelection::Single(index) => *i == index,
                IndexSelection::Range(start_index, end_index) => {
                    start_index <= *i && *i <= end_index
                }
            };

            if is_selected {
                Block::default()
                    .style(Style::new().on_dark_gray())
                    .render(*area, buf);
            }

            let mut left = Line::from("    ");

            let change_id = revision.change_id.into_parts();

            left += Span::styled(change_id.head, Style::new().not_dim().light_magenta());
            left += Span::styled(change_id.tail, Style::new().dim());

            if revision.is_root {
                left += Span::styled(" root()", Style::new().green());
            } else {
                if revision.is_empty {
                    left += Span::styled(" empty()", Style::new().green());
                }

                if let Some(description) = (revision.description.clone())
                    .and_then(|d| d.lines().nth(0).map(|d| d.to_string()))
                {
                    left += Span::from(format!(" {description}"));
                } else {
                    left += Span::styled(
                        " (no description set)",
                        if revision.is_empty {
                            Style::new().green()
                        } else {
                            Style::new().yellow()
                        },
                    );
                }
            }

            let mut right = Line::default();

            let commit_id = revision.commit_id.into_parts();

            right += Span::styled(commit_id.head, Style::new().not_dim().blue());
            right += Span::styled(commit_id.tail, Style::new().dim());

            right += Span::from(" ");

            let [left_area, _, right_area] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(left.width() as u16),
                    Constraint::Fill(1),
                    Constraint::Length(right.width() as u16),
                ])
                .areas(*area);

            left.render(left_area, buf);
            right.render(right_area, buf);
        }

        EmptyBuffer.render(empty, buf);
    }
}
