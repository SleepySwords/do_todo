use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::{
    layout::Constraint,
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

use crate::{app::App, input::Component, utils};

pub struct InputBoxComponent {
    title: String,
    words: Vec<String>,
    callback: Box<dyn Fn(&mut App, String)>,
}

impl InputBoxComponent {
    pub fn new<T: 'static>(title: String, callback: T) -> InputBoxComponent
    where
        T: Fn(&mut App, String),
    {
        InputBoxComponent {
            title,
            words: vec![String::default()],
            callback: Box::new(callback),
        }
    }

    pub fn filled(
        title: String,
        words: String,
        callback: Box<dyn Fn(&mut App, String)>,
    ) -> InputBoxComponent {
        InputBoxComponent {
            title,
            words: vec![words],
            callback,
        }
    }
}

// TODO: create an on_create event and probably an on_destroy event
impl Component for InputBoxComponent {
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()> {
        match key_code {
            KeyCode::Enter => {
                // Clone :(
                if !self.words.join("\n").is_empty() {
                    (self.callback)(app, self.words.join("\n"));
                }
                return None;
            }
            KeyCode::Char(char) => {
                if let Some(x) = self.words.last_mut() {
                    x.push(char);
                }
            }
            KeyCode::Backspace => {
                if let Some(x) = self.words.last_mut() {
                    if x.is_empty() {
                        if self.words.len() > 1 {
                            self.words.pop();
                        }
                    } else {
                        x.pop();
                    }
                }
            }
            KeyCode::Tab => self.words.push(String::default()),
            KeyCode::Esc => {
                // May be better to have a custom escape function
                return None;
            }
            _ => {}
        }
        Some(())
    }

    fn draw<B: tui::backend::Backend>(&self, app: &App, _: Rect, f: &mut tui::Frame<B>) {
        const PADDING: usize = 2;
        const CURSOR_SIZE: usize = 1;
        // Perhaps should respect the boundries of the draw rect?
        let area = utils::centered_rect(
            Constraint::Percentage(70),
            Constraint::Length((self.words.len() as u16).max(1) + PADDING as u16),
            f.size(),
        );

        let lines = self
            .words
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if i == self.words.len() - 1 {
                    let substring_length = if x.len() > area.width as usize - PADDING - CURSOR_SIZE
                    {
                        x.len() + PADDING + CURSOR_SIZE - area.width as usize
                    } else {
                        0
                    };
                    &x[substring_length..]
                } else {
                    &x[..(area.width as usize - PADDING - CURSOR_SIZE).min(x.len())]
                }
            })
            .collect::<Vec<&str>>();

        let text = Text::from(lines.join("\n"));
        let input_box = Paragraph::new(text);
        let input_box = input_box.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .title(self.title.as_ref()),
        );
        f.render_widget(Clear, area);
        f.render_widget(input_box, area);

        let current_line = lines.len() - 1;
        f.set_cursor(
            area.x + 1 + lines[current_line].len() as u16,
            area.y + 1 + current_line as u16,
        )
    }
}
