use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::{
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::{App, PopUpComponents};
use crate::error::AppError;

type InputBoxCallback = Box<dyn Fn(&mut App, String) -> Result<(), AppError>>;
pub const PADDING: usize = 2;
pub const CURSOR_SIZE: usize = 1;

pub struct InputBoxComponent {
    title: String,
    pub words: Vec<String>,
    callback: InputBoxCallback,
}

impl InputBoxComponent {
    pub fn new<T: 'static>(title: String, callback: T) -> InputBoxComponent
    where
        T: Fn(&mut App, String) -> Result<(), AppError>,
    {
        InputBoxComponent {
            title,
            words: vec![String::default()],
            callback: Box::new(callback),
        }
    }

    pub fn filled(title: String, words: String, callback: InputBoxCallback) -> InputBoxComponent {
        InputBoxComponent {
            title,
            words: vec![words],
            callback,
        }
    }

    pub fn handle_event(app: &mut App, key_code: KeyCode) -> Result<(), AppError> {
        let context = if let Some(PopUpComponents::InputBox(context)) = app.popup_context_mut() {
            context
        } else {
            return Ok(());
        };
        match key_code {
            KeyCode::Enter => {
                if !context.words.join("\n").is_empty() {
                    if let Some(PopUpComponents::InputBox(context)) = app.pop_popup() {
                        let err = (context.callback)(app, context.words.join("\n"));
                        if err.is_err() {
                            app.append_layer(PopUpComponents::InputBox(context));
                            return err;
                        }
                    }
                }
            }
            KeyCode::Char(char) => {
                if let Some(x) = context.words.last_mut() {
                    x.push(char);
                }
            }
            KeyCode::Backspace => {
                if let Some(x) = context.words.last_mut() {
                    if x.is_empty() {
                        if context.words.len() > 1 {
                            context.words.pop();
                        }
                    } else {
                        x.pop();
                    }
                }
            }
            KeyCode::Tab => context.words.push(String::default()),
            KeyCode::Esc => {
                // May be better to have a custom escape function
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
            .words
            .iter()
            .enumerate()
            .map(|(i, x)| {
                if i == self.words.len() - 1 {
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
        let input_box = Paragraph::new(text);
        let input_box = input_box.block(
            Block::default()
                .borders(Borders::ALL)
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
