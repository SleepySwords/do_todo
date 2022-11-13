use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Clear, Paragraph};

use crate::app::App;
use crate::utils::{centre_rect, wrap_text};
use crate::view::DrawableComponent;

pub struct MessageBox {
    title: String,
    message: String,
    colour: Color,
}

impl MessageBox {
    pub fn new(title: String, words: String, colour: Color) -> MessageBox {
        MessageBox {
            title,
            message: words,
            colour,
        }
    }
}

impl DrawableComponent for MessageBox {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut crate::view::Drawer) {
        let style = Style::default().fg(self.colour);
        let text = self
            .message
            .split("\n")
            .map(|msg| Span::styled(msg, style))
            .collect::<Vec<Span>>();
        let text = wrap_text(
            tui::text::Spans(text),
            (Constraint::Percentage(70).apply(draw_area.width) - 2).into(),
        );
        let height = text.height();
        let message_box = Paragraph::new(text);
        let message_box = message_box.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .border_style(style)
                .title(self.title.as_ref()),
        );
        let draw_area = centre_rect(
            Constraint::Percentage(70),
            Constraint::Length((height + 2).try_into().unwrap()),
            draw_area,
        );
        drawer.draw_widget(Clear, draw_area);
        drawer.draw_widget(message_box, draw_area);
    }

    fn key_pressed(
        &mut self,
        app: &mut App,
        _: crossterm::event::KeyCode,
    ) -> crate::view::EventResult {
        app.pop_layer();
        crate::view::EventResult::Consumed
    }
}
