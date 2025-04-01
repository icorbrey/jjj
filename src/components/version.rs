use lazy_static::lazy_static;
use ratatui::prelude::*;

pub struct Version;

lazy_static! {
    static ref JJJ_VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

impl Widget for Version {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Text::from(format!("jjj v{}", JJJ_VERSION.to_string()))
            .alignment(Alignment::Right)
            .render(area, buf);
    }
}
