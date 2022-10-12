use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::Style;
use tui::{
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, UserInputType};
use crate::error::AppError;

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

    pub fn handle_event(app: &mut App, key_code: KeyCode) -> Result<(), AppError> {
        let context = if let Some(UserInputType::InputBox(context)) = app.popup_context_mut() {
            context
        } else {
            return Ok(());
        };
        match key_code {
            KeyCode::Enter => {
                if !context.user_input.join("\n").is_empty() {
                    if let Some(UserInputType::InputBox(context)) = app.pop_popup() {
                        let err = (context.callback)(app, context.user_input.join("\n"));
                        if err.is_err() {
                            app.append_layer(UserInputType::InputBox(context));
                            return err;
                        }
                    }
                }
            }
            KeyCode::Char(char) => {
                if let Some(x) = context.user_input.last_mut() {
                    x.push(char);
                }
            }
            KeyCode::Backspace => {
                if let Some(x) = context.user_input.last_mut() {
                    if x.is_empty() {
                        if context.user_input.len() > 1 {
                            context.user_input.pop();
                        }
                    } else {
                        x.pop();
                    }
                }
            }
            KeyCode::Tab => context.user_input.push(String::default()),
            KeyCode::Esc => {
                app.pop_popup();
            }
            _ => {}
        }

        Ok(())
    }

    pub fn draw<B: tui::backend::Backend>(
        &self,
        app: &App,
        draw_area: Rect,
        f: &mut tui::Frame<B>,
    ) {
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
        f.render_widget(Clear, draw_area);
        f.render_widget(input_box, draw_area);

        let current_line = lines.len() - 1;
        f.set_cursor(
            draw_area.x + 1 + lines[current_line].len() as u16,
            draw_area.y + 1 + current_line as u16,
        )
    }
}
