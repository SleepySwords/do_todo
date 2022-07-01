use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::Paragraph,
    Frame,
};

use crate::{app::App, input::Component};

pub struct StatusLineComponent {
    status_line: String,
}

impl StatusLineComponent {
    pub fn new(status_line: String) -> StatusLineComponent {
        StatusLineComponent { status_line }
    }
}

impl Component for StatusLineComponent {
    fn handle_event(&mut self, _: &mut App, _: KeyCode) -> Option<()> {
        None
    }

    fn draw<B: tui::backend::Backend>(&self, _: &App, area: Rect, f: &mut Frame<B>) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(Color::White));
        let paragraph = Paragraph::new(help);
        f.render_widget(paragraph, area);
    }
}
