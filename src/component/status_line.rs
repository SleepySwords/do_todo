use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

// TODO: Proper impl with actual colours
pub struct StatusLine {
    pub status_line: String,
    pub colour: Color,
}

impl Default for StatusLine {
    fn default() -> Self {
        StatusLine {
            status_line: String::default(),
            colour: Color::White,
        }
    }
}

impl StatusLine {
    pub fn new(status_line: String) -> StatusLine {
        StatusLine {
            status_line,
            colour: Color::White,
        }
    }
}

impl StatusLine {
    // Should be able to do commands?!
    pub fn draw<B: tui::backend::Backend>(&self, _: &App, draw_area: Rect, f: &mut Frame<B>) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(self.colour));
        let paragraph = Paragraph::new(help);
        f.render_widget(paragraph, draw_area);
    }
}
