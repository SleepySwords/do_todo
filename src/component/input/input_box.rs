use crossterm::event::KeyCode;

use tui::layout::{Constraint, Rect};
use tui::style::Style;
use tui::{
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::component::message_box::MessageBox;
use crate::error::AppError;
use crate::utils;
use crate::view::{DrawableComponent, EventResult};

type InputBoxCallback = Box<dyn Fn(&mut App, String) -> Result<(), AppError>>;

pub const PADDING: usize = 2;
pub const CURSOR_SIZE: usize = 1;

pub struct InputBox {
    title: String,
    pub user_input: Vec<String>,
    callback: InputBoxCallback,
}

impl InputBox {
    pub fn new<T: 'static>(title: String, callback: T) -> InputBox
    where
        T: Fn(&mut App, String) -> Result<(), AppError>,
    {
        InputBox {
            title,
            user_input: vec![String::default()],
            callback: Box::new(callback),
        }
    }

    pub fn filled(title: String, words: String, callback: InputBoxCallback) -> InputBox {
        InputBox {
            title,
            user_input: vec![words],
            callback,
        }
    }
}

impl DrawableComponent for InputBox {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut crate::view::Drawer) {
        let draw_area = utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.user_input.len() as u16 + 2),
            draw_area,
        );
        let lines = self
            .user_input
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if i == self.user_input.len() - 1 {
                    let substring_length =
                        if x.len() > draw_area.width as usize - PADDING - CURSOR_SIZE {
                            x.len() + PADDING + CURSOR_SIZE - draw_area.width as usize
                        } else {
                            0
                        };
                    &x[substring_length..]
                } else {
                    &x[..(draw_area.width as usize - PADDING - CURSOR_SIZE).min(x.len())]
                }
            })
            .collect::<Vec<&str>>();

        let text = Text::from(lines.join("\n"));
        let input_box = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.selected_border_colour))
                .border_type(app.theme.border_style.border_type)
                .title(self.title.as_ref()),
        );
        drawer.draw_widget(Clear, draw_area);
        drawer.draw_widget(input_box, draw_area);

        let current_line = lines.len() - 1;
        drawer.set_cursor(
            draw_area.x + 1 + lines[current_line].len() as u16,
            draw_area.y + 1 + current_line as u16,
        )
    }

    fn key_pressed(
        &mut self,
        app: &mut App,
        key_code: crossterm::event::KeyCode,
    ) -> crate::view::EventResult {
        match key_code {
            KeyCode::Enter => {
                if !self.user_input.join("\n").is_empty() {
                    app.pop_stack();
                    let err = (self.callback)(app, self.user_input.join("\n"));
                    if err.is_err() {
                        app.append_stack(Box::new(MessageBox::new(
                            String::from("Error"),
                            err.err().unwrap().to_string(),
                            tui::style::Color::Red,
                        )));
                    }
                    return EventResult::Consumed;
                }
            }
            KeyCode::Char(char) => {
                if let Some(x) = self.user_input.last_mut() {
                    x.push(char);
                }
            }
            KeyCode::Backspace => {
                if let Some(x) = self.user_input.last_mut() {
                    if x.is_empty() {
                        if self.user_input.len() > 1 {
                            self.user_input.pop();
                        }
                    } else {
                        x.pop();
                    }
                }
            }
            KeyCode::Tab => self.user_input.push(String::default()),
            KeyCode::Esc => {
                app.pop_stack();
            }
            _ => {}
        }
        EventResult::Consumed
    }
}
