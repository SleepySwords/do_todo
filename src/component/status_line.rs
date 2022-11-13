use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

use crate::{
    app::App,
    view::{DrawableComponent, Drawer, EventResult},
};

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

impl DrawableComponent for StatusLine {
    // Should be able to do commands?!
    fn draw(&self, _: &App, draw_area: Rect, drawer: &mut Drawer) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(self.colour));
        let paragraph = Paragraph::new(help);
        drawer.draw_widget(paragraph, draw_area);
    }

    fn key_pressed(&mut self, _: &mut App, _: KeyCode) -> EventResult {
        EventResult::Ignored
    }
}
