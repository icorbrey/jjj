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
