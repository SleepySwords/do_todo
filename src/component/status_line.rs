use crossterm::event::KeyEvent;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

use crate::{
    app::App,
    draw::{DrawableComponent, Drawer, EventResult},
};

// TODO: Proper impl with actual colours
pub struct StatusLine {
    pub status_line: String,
    pub colour: Color,
    pub draw_area: Rect,
}

impl Default for StatusLine {
    fn default() -> Self {
        StatusLine {
            status_line: String::default(),
            colour: Color::White,
            draw_area: Rect::default(),
        }
    }
}

impl StatusLine {
    pub fn new(status_line: String) -> StatusLine {
        StatusLine {
            status_line,
            colour: Color::White,
            draw_area: Rect::default(),
        }
    }
}

impl DrawableComponent for StatusLine {
    // Should be able to do commands?!
    fn draw(&self, _: &App, drawer: &mut Drawer) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(self.colour));
        let paragraph = Paragraph::new(help);
        drawer.draw_widget(paragraph, self.draw_area);
    }

    fn key_pressed(&mut self, _: &mut App, _: KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = draw_area;
    }
}
