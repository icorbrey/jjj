use ratatui::{prelude::*, widgets::Paragraph};

/// Renders dim tildes to the given area to indicate that the buffer is empty.
pub struct EmptyBuffer;

impl Widget for EmptyBuffer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(
            (0..area.height)
                .map(|_| Line::from("  ~".dim()))
                .collect::<Vec<Line>>(),
        )
        .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::backend::TestBackend;

    use super::*;

    #[test]
    fn snapshot_command_line() {
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(EmptyBuffer, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }
}
