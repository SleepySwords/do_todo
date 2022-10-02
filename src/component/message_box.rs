use crossterm::event::KeyCode;

use tui::layout::{Rect, Constraint};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::{App, PopUpComponents};
use crate::utils::{wrap_text, centered_rect};

pub struct MessageBoxComponent {
    title: String,
    words: String,
    colour: Color,
}

impl MessageBoxComponent {
    pub fn new(title: String, words: String, colour: Color) -> MessageBoxComponent {
        MessageBoxComponent { title, words, colour }
    }

    pub fn handle_event(app: &mut App, _: KeyCode) {
        if let Some(PopUpComponents::MessageBox(_)) = app.popup_context_mut() {
            app.pop_popup();
        }
    }

    pub fn draw<B: tui::backend::Backend>(
        &self,
        app: &App,
        draw_area: Rect,
        f: &mut tui::Frame<B>,
    ) {
        let style = Style::default().fg(self.colour);
        let text = wrap_text(tui::text::Spans(vec![Span::styled(&self.words, style.clone())]), (draw_area.width - 2).into());
        let height = text.height();
        let message_box = Paragraph::new(text);
        let message_box = message_box.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .border_style(style)
                .title(self.title.as_ref()),
        );
        let draw_area = centered_rect(Constraint::Length(draw_area.width), Constraint::Length((height+2).try_into().unwrap()), draw_area);
        f.render_widget(Clear, draw_area);
        f.render_widget(message_box, draw_area);
    }
}
