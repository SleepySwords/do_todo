use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

// TODO: Proper impl with actual colours
pub struct StatusLineComponent {
    pub status_line: String,
    pub colour: Color,
}

impl Default for StatusLineComponent {
    fn default() -> Self {
        StatusLineComponent {
            status_line: String::default(),
            colour: Color::White,
        }
    }
}

impl StatusLineComponent {
    pub fn new(status_line: String) -> StatusLineComponent {
        StatusLineComponent {
            status_line,
            colour: Color::White,
        }
    }
}

impl StatusLineComponent {
    // Should be able to do commands?!
    pub fn draw<B: tui::backend::Backend>(&self, _: &App, area: Rect, f: &mut Frame<B>) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(self.colour));
        let paragraph = Paragraph::new(help);
        f.render_widget(paragraph, area);
    }
}
