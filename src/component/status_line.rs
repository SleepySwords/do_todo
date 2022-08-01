use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub struct StatusLineComponent {
    status_line: String,
}

impl StatusLineComponent {
    pub fn new(status_line: String) -> StatusLineComponent {
        StatusLineComponent { status_line }
    }
}

impl StatusLineComponent {
    // Should be able to do commands?!
    pub fn draw<B: tui::backend::Backend>(&self, _: &App, area: Rect, f: &mut Frame<B>) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(Color::White));
        let paragraph = Paragraph::new(help);
        f.render_widget(paragraph, area);
    }
}
