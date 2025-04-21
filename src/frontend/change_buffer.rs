use anyhow::anyhow;
use bevy::prelude::*;
use bevy_ratatui::event::KeyEvent;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    prelude::{Rect, *},
    widgets::Block,
};

use crate::backend::{
    log::{LogOutput, LogResponseEvent, RefreshLogEvent},
    revisions::{ChangeId, CommitId, Revision},
    JujutsuCli,
};

use super::{command_line::NotificationEvent, prelude::*};

#[mutants::skip]
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
    log_output: Vec<LogOutput>,
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

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum RevisionSelection {
    Single(Revision),
    Range(Revision, Revision),
}

fn read_keys(
    mut ev_selection: EventWriter<ChangeBufferSelectionEvent>,
    mut ev_notification: EventWriter<NotificationEvent>,
    mut ev_refresh_log: EventWriter<RefreshLogEvent>,
    mut change_buffer: Query<&mut ChangeBuffer>,
    mut ev_keypresses: EventReader<KeyEvent>,
    mut exit: EventWriter<AppExit>,
    mut navigation: Navigation,
    jj_cli: Res<JujutsuCli>,
) -> Result<()> {
    let mut change_buffer = change_buffer.get_single_mut()?;

    if change_buffer.revisions.is_empty() {
        return Ok(());
    }

    for keypress in ev_keypresses.read() {
        if keypress.kind != KeyEventKind::Press {
            continue;
        }

        let max = change_buffer.revisions.len().saturating_sub(1);

        let mut selection = None;

        match (change_buffer.selection.clone(), keypress.code) {
            (_, KeyCode::Char(' ')) => {
                navigation.spawn_popup(SpaceMenu)?;
            }
            (_, KeyCode::Char('q')) => {
                exit.send_default();
            }
            (IndexSelection::Single(i), KeyCode::Char('a')) => {
                let change_id = change_buffer.revisions[i].change_id.clone();
                jj_cli.abandon(change_id.clone())?;
                ev_refresh_log.send_default();
                ev_notification.send(NotificationEvent::new(format!("Abandoned {change_id}")));
            }
            (_, KeyCode::Char('u')) => {
                jj_cli.undo()?;
                ev_refresh_log.send_default();
                ev_notification.send(NotificationEvent::new("Undo complete".into()));
            }
            (IndexSelection::Single(i), KeyCode::Char('n')) => {
                let change_id = change_buffer.revisions[i].change_id.clone();
                jj_cli.new(change_id.clone())?;
                selection = Some(IndexSelection::Single(0));
                ev_refresh_log.send_default();
                ev_notification.send(NotificationEvent::new(format!(
                    "Created new commit on {change_id}"
                )));
            }
            (IndexSelection::Single(i), KeyCode::Char('s')) => {
                let change_id = change_buffer.revisions[i].change_id.clone();
                jj_cli.squash(change_id.clone())?;
                ev_refresh_log.send_default();
                ev_notification.send(NotificationEvent::new(format!("Squashed {change_id}")));
            }
            (IndexSelection::Single(i), KeyCode::Enter) => {
                let change_id = change_buffer.revisions[i].change_id.clone();
                jj_cli.edit(change_id.clone())?;
                ev_refresh_log.send_default();
                ev_notification.send(NotificationEvent::new(format!("Editing {change_id}")));
            }
            (IndexSelection::Single(i), KeyCode::Char('j')) => {
                selection = Some(IndexSelection::Single(usize::min(i + 1, max)));
            }
            (IndexSelection::Single(i), KeyCode::Char('k')) => {
                selection = Some(IndexSelection::Single(i.saturating_sub(1)));
            }
            (IndexSelection::Single(i), KeyCode::Char('x')) => {
                selection = if i != (i + 1).clamp(0, max) {
                    Some(IndexSelection::Range(i, usize::min(i + 1, max)))
                } else {
                    Some(IndexSelection::Single(i))
                };
            }
            (IndexSelection::Range(_, end), KeyCode::Char('j')) => {
                selection = Some(IndexSelection::Single(usize::min(end + 1, max)));
            }
            (IndexSelection::Range(start, _), KeyCode::Char('k')) => {
                selection = Some(IndexSelection::Single(start.saturating_sub(1)));
            }
            (IndexSelection::Range(start, end), KeyCode::Char('x')) => {
                selection = Some(IndexSelection::Range(start, usize::min(end + 1, max)));
            }
            _ => {}
        }

        if let Some(selection) = selection {
            change_buffer.selection = selection.clone();
            let revisions = change_buffer.revisions.clone();
            ev_selection.send(ChangeBufferSelectionEvent(get_revision_selection(
                revisions, selection,
            )));
        }
    }

    Ok(())
}

fn get_revision_selection(
    revisions: Vec<Revision>,
    selection: IndexSelection,
) -> RevisionSelection {
    match selection {
        IndexSelection::Single(i) => RevisionSelection::Single(revisions[i].clone()),
        IndexSelection::Range(start, end) => {
            RevisionSelection::Range(revisions[start].clone(), revisions[end].clone())
        }
    }
}

fn read_revisions(
    mut ev_selection: EventWriter<ChangeBufferSelectionEvent>,
    mut ev_log_response: EventReader<LogResponseEvent>,
    mut change_buffer: Query<&mut ChangeBuffer>,
) -> Result<()> {
    let mut change_buffer = change_buffer.get_single_mut()?;

    for LogResponseEvent(log_output) in ev_log_response.read() {
        let revisions = (log_output.iter())
            .filter_map(|l| l.clone().into_revision().ok())
            .collect();

        if change_buffer.log_output.is_empty() {
            change_buffer.selection = IndexSelection::Single(0);
            change_buffer.log_output = log_output.clone();
            change_buffer.revisions = revisions;
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
                change_buffer.log_output = log_output.clone();
                change_buffer.revisions = revisions;
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

                change_buffer.log_output = log_output.clone();
                change_buffer.revisions = revisions;
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
        let map_index = |i: usize| {
            if self.revisions.is_empty() {
                return 0..=0;
            }

            let revision = &self.revisions[i];
            let line_index = (self.log_output.iter())
                .position(|l| {
                    l.as_revision()
                        .is_some_and(|r| r.change_id.0 == revision.change_id.0)
                })
                .ok_or_else(|| {
                    anyhow!("Couldn't map revision to log line. This should never happen.")
                })
                .unwrap();

            line_index..=line_index
        };

        let selected_lines = match self.selection {
            IndexSelection::Single(index) => map_index(index),
            IndexSelection::Range(start, end) => *map_index(start).start()..=*map_index(end).end(),
        };

        let mut lines = vec![];
        let mut last_selected_index = 0;

        for (i, item) in self.log_output.iter().enumerate() {
            let is_selected = selected_lines.contains(&i);
            lines.append(&mut LogLine::vec_from(item, is_selected));

            if is_selected {
                last_selected_index = lines.len() - 1;
            }
        }

        let (computed_viewport_y, line_range) = viewport::compute_sliding_window(
            lines.len(),
            last_selected_index,
            *viewport_y,
            area.height as usize,
            area.height as usize / 4,
        );

        *viewport_y = computed_viewport_y;
        let lines = &lines[line_range.clone()];

        let [revs_area, empty] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(lines.len() as u16), Constraint::Fill(1)])
            .areas(area);

        let rev_lines = Layout::default()
            .direction(Direction::Vertical)
            .constraints(lines.iter().map(|_| Constraint::Length(1)))
            .split(revs_area);

        for (line, area) in lines.iter().zip(rev_lines.iter()) {
            line.clone().render(*area, buf);
        }

        EmptyBuffer.render(empty, buf);
    }
}

#[derive(Clone)]
enum LogLine {
    Revision(RevisionLine),
    Decoration(DecorationLine),
}

impl LogLine {
    fn vec_from(log_output: &LogOutput, is_selected: bool) -> Vec<Self> {
        match log_output {
            LogOutput::Decoration(text) => {
                vec![Self::Decoration(DecorationLine {
                    text: text.clone(),
                    is_selected,
                })]
            }
            LogOutput::Revision(revision) => RevisionLine::vec_from(revision, is_selected)
                .into_iter()
                .map(Self::Revision)
                .collect(),
        }
    }
}

impl Widget for LogLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        match self {
            Self::Decoration(decoration) => decoration.render(area, buf),
            Self::Revision(revision) => match revision {
                RevisionLine::Top(top) => top.render(area, buf),
                RevisionLine::Bottom(bottom) => bottom.render(area, buf),
            },
        }
    }
}

#[derive(Clone)]
enum RevisionLine {
    Top(RevisionTopLine),
    Bottom(RevisionBottomLine),
}

impl RevisionLine {
    fn vec_from(revision: &Revision, is_selected: bool) -> Vec<Self> {
        let graph = revision.graph.clone();
        vec![
            RevisionLine::Top(RevisionTopLine {
                graph: graph.head,
                change_id: revision.change_id.clone(),
                commit_id: revision.commit_id.clone(),
                is_root: revision.is_root,
                is_selected,
                author: revision.author.clone(),
                timestamp: revision.timestamp.clone(),
                bookmarks: revision.bookmarks.clone(),
            }),
            RevisionLine::Bottom(RevisionBottomLine {
                graph: graph.tail,
                is_empty: revision.is_empty,
                description: revision.description.clone(),
                is_selected,
            }),
        ]
    }
}

#[derive(Clone)]
struct RevisionTopLine {
    graph: String,
    change_id: ChangeId,
    commit_id: CommitId,
    is_root: bool,
    is_selected: bool,
    author: String,
    timestamp: String,
    bookmarks: Vec<String>,
}

impl Widget for RevisionTopLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut left = Line::from("  ");

        let change_id = self.change_id.into_parts();

        left += Span::from(self.graph.clone());
        left += Span::styled(change_id.head, Style::new().not_dim().light_magenta());
        left += Span::styled(change_id.tail, Style::new().dim());

        if self.is_root {
            left += Span::styled(" root()", Style::new().green());
        } else {
            left += Span::styled(format!(" {}", self.author), Style::new().yellow());
            left += Span::styled(format!(" {}", self.timestamp), Style::new().cyan());
        }

        // In case the window isn't wide enough for both left and right sides, make sure the left
        // side is legible and separate.
        left += "  ".into();

        let mut right = Line::default().alignment(Alignment::Right);

        for bookmark in self.bookmarks.iter() {
            right += Span::styled(format!("{bookmark} "), Style::new().magenta());
        }

        let commit_id = self.commit_id.into_parts();

        right += Span::styled(commit_id.head, Style::new().not_dim().blue());
        right += Span::styled(commit_id.tail, Style::new().dim());

        right += Span::from(" ");

        let [left_area, right_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(left.width() as u16), Constraint::Fill(1)])
            .areas(area);

        if self.is_selected {
            Block::default()
                .style(Style::new().on_dark_gray())
                .render(area, buf)
        }

        right.render(right_area, buf);
        left.render(left_area, buf);
    }
}

#[derive(Clone)]
struct RevisionBottomLine {
    graph: String,
    is_empty: bool,
    description: Option<String>,
    is_selected: bool,
}

impl Widget for RevisionBottomLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let mut bottom = Line::from("  ");

        bottom += Span::from(self.graph.clone());

        if self.is_empty {
            bottom += Span::styled("(empty) ", Style::new().green());
        }

        if let Some(description) =
            (self.description.clone()).and_then(|d| d.lines().nth(0).map(|d| d.to_string()))
        {
            bottom += Span::from(format!("{description} "));
        } else {
            bottom += Span::styled(
                "(no description set) ",
                if self.is_empty {
                    Style::new().green()
                } else {
                    Style::new().yellow()
                },
            );
        }

        if self.is_selected {
            Block::default()
                .style(Style::new().on_dark_gray())
                .render(area, buf)
        }

        bottom.render(area, buf);
    }
}

#[derive(Clone)]
struct DecorationLine {
    text: String,
    is_selected: bool,
}

impl Widget for DecorationLine {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        if self.is_selected {
            Block::default()
                .style(Style::new().on_dark_gray())
                .render(area, buf)
        }

        Span::styled(format!("  {}", self.text), Style::new().dim()).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn snapshot_revision_top_line() {
        let line = RevisionTopLine {
            change_id: ChangeId("12345678".into(), 1),
            commit_id: CommitId("abcdefgh".into(), 2),
            graph: ">  ".into(),
            is_root: false,
            is_selected: true,
            author: "John Smith".into(),
            timestamp: "2 days ago".into(),
            bookmarks: vec!["one".into(), "two".into()],
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(line, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn snapshot_revision_bottom_line() {
        let line = RevisionBottomLine {
            description: Some("This is a test".into()),
            graph: "|  ".into(),
            is_empty: true,
            is_selected: false,
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(line, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn snapshot_decoration_line() {
        let line = DecorationLine {
            text: "~  (elided revisions)".into(),
            is_selected: true,
        };

        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(line, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
