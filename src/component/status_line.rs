use crossterm::event::KeyEvent;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

use crate::{
    app::App,
    draw::{Component, Drawer, PostAction, Action},
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

impl Component for StatusLine {
    // Should be able to do commands?!
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let help = Text::styled(
            self.status_line.clone()
                + if app.task_store.auto_sort {
                    " Auto sort is current enabled"
                } else {
                    ""
                },
            Style::default().fg(self.colour),
        );
        let paragraph = Paragraph::new(help);
        drawer.draw_widget(paragraph, self.draw_area);
    }

    fn key_event(&mut self, _: &mut App, _: KeyEvent) -> PostAction {
        PostAction {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = draw_area;
    }
}
