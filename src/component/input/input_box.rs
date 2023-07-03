use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use tui::layout::{Constraint, Rect};
use tui::style::Style;
use tui::widgets::{Block, Borders, Clear};
use tui_textarea::{TextArea, Input};

use crate::app::{App, Mode};
use crate::error::AppError;
use crate::utils;
use crate::draw::{DrawableComponent, EventResult};

type InputBoxCallback = Option<Box<dyn FnOnce(&mut App, String) -> Result<(), AppError>>>;
type ErrorCallback = Box<dyn Fn(&mut App, AppError)>;

pub const PADDING: usize = 2;
pub const CURSOR_SIZE: usize = 1;

pub struct InputBox {
    title: String,
    text_area: TextArea<'static>,
    callback: InputBoxCallback,
    error_callback: ErrorCallback,
    draw_area: Rect,
    mode_to_restore: Option<Mode>,
}

impl InputBox {
    pub fn filled(title: String, words: &str, callback: InputBoxCallback) -> InputBox {
        let words = words
            .split("\n")
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        let c = words[0].len();

        let mut text_area = TextArea::new(words);
        text_area.move_cursor(tui_textarea::CursorMove::Jump(0, c as u16));
        InputBox {
            title,
            text_area,
            callback,
            error_callback: Box::new(|_, _| {}),
            draw_area: Rect::default(),
            mode_to_restore: None,
        }
    }
}

impl DrawableComponent for InputBox {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut crate::draw::Drawer) {
        let draw_area = utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.text_area.lines().len() as u16 + 2),
            draw_area,
        );

        let widget = self.text_area.widget();

        let boxes = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.selected_border_colour))
            .border_type(app.theme.border_style.border_type)
            .title(self.title.as_ref());
        let box_area = boxes.inner(draw_area);
        drawer.draw_widget(Clear, draw_area);
        drawer.draw_widget(boxes, draw_area);
        drawer.draw_widget(widget, box_area);
    }

    fn key_pressed(&mut self, app: &mut App, key_event: KeyEvent) -> EventResult {
        let key_code = key_event.code;
        match key_code {
            KeyCode::Enter => {
                if !self.text_area.lines().join("\n").is_empty() {
                    // When popping the layer, probably should do the callback, rather than have an
                    // option.
                    app.pop_layer();
                    if let Some(mode) = self.mode_to_restore {
                        app.mode = mode;
                    }
                    if let Some(callback) = self.callback.take() {
                        let err = (callback)(app, self.text_area.lines().join("\n"));
                        if err.is_err() {
                            (self.error_callback)(app, err.err().unwrap());
                        }
                    }
                }
            }
            KeyCode::Tab => {
                self.text_area.insert_newline();
            }
            KeyCode::Esc => {
                app.pop_layer();
                if let Some(mode) = self.mode_to_restore {
                    app.mode = mode;
                }
            }
            _ => {
                self.text_area.input(Input::from(key_event));
            }
        }
        EventResult::Consumed
    }

    fn update_layout(&mut self, rect: Rect) {
        self.draw_area = rect;
    }

    fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> EventResult {
        let draw_area = utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.text_area.lines().len() as u16 + 2),
            self.draw_area,
        );
        if let MouseEventKind::Down(..) = mouse_event.kind {
            if utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
                app.println(format!("{:?} {:?}", draw_area.x, mouse_event.column));
                // Either we use inner on draw_area to exclude border, or this to include it
                // and set the border to jump to 0
                if draw_area.x == mouse_event.column {
                    self.text_area
                        .move_cursor(tui_textarea::CursorMove::Jump(0, 0));
                } else {
                    self.text_area.move_cursor(tui_textarea::CursorMove::Jump(
                        0,
                        mouse_event.column - draw_area.x - 1,
                    ));
                }
                EventResult::Consumed
            } else {
                app.pop_layer();
                EventResult::Consumed
            }
        } else {
            EventResult::Consumed
        }
    }
}

pub struct InputBoxBuilder {
    title: String,
    text_area: TextArea<'static>,
    callback: InputBoxCallback,
    error_callback: ErrorCallback,
    draw_area: Rect,
    mode_to_restore: Option<Mode>,
}

impl Default for InputBoxBuilder {
    fn default() -> Self {
        InputBoxBuilder {
            title: String::default(),
            text_area: TextArea::default(),
            callback: Some(Box::new(|_app, _task| Ok(()))),
            error_callback: Box::new(|_app, _err| {}),
            draw_area: Rect::default(),
            mode_to_restore: None,
        }
    }
}

impl InputBoxBuilder {
    pub fn build(self) -> InputBox {
        InputBox {
            title: self.title,
            text_area: self.text_area,
            callback: self.callback,
            error_callback: self.error_callback,
            draw_area: self.draw_area,
            mode_to_restore: self.mode_to_restore,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn fill(self, words: &str) -> Self {
        let words = words
            .split("\n")
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        let c = words[0].len();

        let mut text_area = TextArea::new(words);
        text_area.move_cursor(tui_textarea::CursorMove::Jump(0, c as u16));
        self.text_area(text_area)
    }

    pub fn text_area(mut self, text_area: TextArea<'static>) -> Self {
        self.text_area = text_area;
        self
    }

    pub fn callback<T: 'static>(mut self, callback: T) -> Self
    where
        T: FnOnce(&mut App, String) -> Result<(), AppError>,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    pub fn error_callback<T: 'static>(mut self, error_callback: T) -> Self
    where
        T: Fn(&mut App, AppError),
    {
        self.error_callback = Box::new(error_callback);
        self
    }

    pub fn draw_area(mut self, draw_area: Rect) -> Self {
        self.draw_area = draw_area;
        self
    }

    pub fn save_mode(mut self, app: &mut App) -> Self {
        self.mode_to_restore = Some(app.mode);
        app.mode = Mode::Overlay;
        self
    }
}
