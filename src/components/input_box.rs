use crossterm::event::KeyCode;

use tui::{
    layout::Rect,
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::{app::App, input::Component};

pub struct InputBoxComponent {
    title: String,
    words: String,
    callback: Box<dyn Fn(&mut App, String)>,
}

impl InputBoxComponent {
    pub fn new(title: String, callback: Box<dyn Fn(&mut App, String)>) -> InputBoxComponent {
        InputBoxComponent {
            title,
            words: String::default(),
            callback,
        }
    }

    pub fn new_preexisting_word(
        title: String,
        words: String,
        callback: Box<dyn Fn(&mut App, String)>,
    ) -> InputBoxComponent {
        InputBoxComponent {
            title,
            words,
            callback,
        }
    }
}

impl Component for InputBoxComponent {
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()> {
        match key_code {
            KeyCode::Enter => {
                // Clone :(
                (self.callback)(app, self.words.clone());
                return None;
            }
            KeyCode::Char(char) => {
                self.words.push(char);
            }
            KeyCode::Backspace => {
                self.words.pop();
            }
            KeyCode::Esc => {
                // May be better to have a custom escape function
                return None;
            }
            _ => {}
        }
        Some(())
    }

    fn draw<B: tui::backend::Backend>(&self, _: &App, area: Rect, f: &mut tui::Frame<B>) {
        // Fix this bit
        f.render_widget(Clear, area);
        let words = textwrap::fill(self.words.as_ref(), (area.width - 2) as usize);
        let text = Text::from(words.as_ref());
        let help_message = Paragraph::new(text);
        let input_box = help_message.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(self.title.as_ref()),
        );
        let height = words.chars().filter(|c| *c == '\n').count() + 1;
        f.render_widget(Clear, area);
        f.render_widget(input_box, area);
        f.set_cursor(
            area.x + 1 + (self.words.len() % (area.width - 1) as usize) as u16,
            area.y + height as u16,
        )
    }
}
