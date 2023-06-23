use tui::{
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Clear, Widget},
};
use unicode_segmentation::UnicodeSegmentation;

use crate::view::DrawableComponent;

struct WrappedText<'a> {
    pub line: Spans<'a>,
}

impl Widget for WrappedText<'_> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        Clear.render(area, buf);

        let mut word = Vec::new();
        let mut current_width = 0u16;
        let mut current_height = 0u16;
        for span in &self.line.0 {
            let style = span.style;
            for grapheme in UnicodeSegmentation::graphemes(span.content.as_ref(), true) {
                let is_newline = grapheme.chars().any(|chr| chr == '\n');
                if is_newline {
                    flush(current_width, current_height, word, area, buf);
                    current_width = 0;
                    current_height += 1;
                    word = Vec::new();
                    continue;
                }
                let is_whitespace = grapheme.chars().all(&char::is_whitespace);
                if is_whitespace {
                    if current_width + word.len() as u16 != area.width {
                        word.push((grapheme, style));
                    }
                    let size = word.len();
                    flush(current_width, current_height, word, area, buf);
                    word = Vec::new();
                    current_width += size as u16;
                    continue;
                }
                word.push((grapheme, style));
                if word.len() as u16 == area.width {
                    flush(current_width, current_height, word, area, buf);
                    word = Vec::new();
                    current_width = 0;
                    current_height += 1;
                }
                if current_width + word.len() as u16 > area.width {
                    current_width = 0;
                    current_height += 1;
                }
            }
        }
        flush(current_width, current_height, word, area, buf);
    }
}

fn flush(
    current_width: u16,
    current_height: u16,
    word: Vec<(&str, Style)>,
    area: tui::layout::Rect,
    buf: &mut tui::buffer::Buffer,
) {
    for (i, (symbol, style)) in word.iter().enumerate() {
        buf.get_mut(i as u16 + current_width + area.x, current_height)
            .set_symbol(symbol);
        buf.get_mut(i as u16 + current_width + area.x, current_height)
            .set_style(*style);
    }
}

pub struct TestWrap {
    pub text: String,
}

impl DrawableComponent for TestWrap {
    fn draw(&self, _: &crate::app::App, _: tui::layout::Rect, drawer: &mut crate::view::Drawer) {
        let text = WrappedText {
            line: Spans::from(self.text.as_str()),
        };
        drawer.draw_widget(
            text,
            Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        )
    }

    fn key_pressed(
        &mut self,
        app: &mut crate::app::App,
        _: crossterm::event::KeyEvent,
    ) -> crate::view::EventResult {
        app.pop_layer();
        crate::view::EventResult::Consumed
    }
}
