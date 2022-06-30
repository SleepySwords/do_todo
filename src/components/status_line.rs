use tui::{style::{Style, Color}, widgets::Paragraph, text::Text};

use crate::input::Component;

pub struct StatusLineComponent {
    status_line: String
}

impl StatusLineComponent {
    pub fn new(status_line: String) -> StatusLineComponent {
        StatusLineComponent { status_line }
    }
}

impl Component for StatusLineComponent {
    fn handle_event(&mut self, _: &mut crate::app::App, _: crossterm::event::KeyCode) -> Option<()> {
        None
    }

    fn draw<B: tui::backend::Backend>(&self, _: &crate::app::App, area: tui::layout::Rect, f: &mut tui::Frame<B>) {
        let help = Text::styled(self.status_line.as_str(), Style::default().fg(Color::White));
        let paragraph = Paragraph::new(help);
        f.render_widget(paragraph, area);
    }
}
