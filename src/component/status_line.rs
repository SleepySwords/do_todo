use crossterm::event::KeyEvent;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
};

use crate::{
    app::App,
    framework::{
        component::{Component, Drawer},
        event::{Action, PostEvent},
    },
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
            colour: Color::Reset,
            draw_area: Rect::default(),
        }
    }
}

impl StatusLine {
    pub fn new(status_line: String) -> StatusLine {
        StatusLine {
            status_line,
            colour: Color::Reset,
            draw_area: Rect::default(),
        }
    }
}

const SPINNER: [&str; 4] = ["-", "\\", "|", "/"];

impl Component for StatusLine {
    // Should be able to do commands?!
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let mut status_line = self.status_line.clone();
        if app.task_list.auto_sort {
            status_line += " Auto sort is current enabled"
        }
        if app.task_store.is_syncing() {
            status_line += &format!(" {}", SPINNER[app.tick % SPINNER.len()]);
        }
        let help = Text::styled(status_line, Style::default().fg(self.colour));
        let paragraph = Paragraph::new(help);
        drawer.draw_widget(paragraph, self.draw_area);
    }

    fn key_event(&mut self, _: &mut App, _: KeyEvent) -> PostEvent {
        PostEvent {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = draw_area;
    }
}
