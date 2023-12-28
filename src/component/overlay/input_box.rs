use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Clear},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    error::AppError,
    utils,
};

type InputBoxCallback = Option<Box<dyn FnOnce(&mut App, String) -> Result<(), AppError>>>;
type ErrorCallback = Box<dyn Fn(&mut App, AppError)>;

pub struct InputBox {
    pub draw_area: Rect,
    title: String,
    text_area: TextArea<'static>,
    callback: InputBoxCallback,
    error_callback: ErrorCallback,
    prev_mode: Option<Mode>,
    full_width: bool,
}

impl InputBox {
    #[allow(dead_code)]
    pub fn filled(title: String, words: &str, callback: InputBoxCallback) -> InputBox {
        let words = words
            .split('\n')
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
            prev_mode: None,
            full_width: false,
        }
    }

    pub fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }
}

impl DrawableComponent for InputBox {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let widget = self.text_area.widget();
        let boxes = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.selected_border_colour))
            .border_type(app.theme.border_type)
            .title(self.title.as_ref());
        let box_area = boxes.inner(self.draw_area);

        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_widget(boxes, self.draw_area);
        drawer.draw_widget(widget, box_area);
    }

    fn key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EventResult {
        match key_event.code {
            KeyCode::Enter => {
                if !self.text_area.lines().join("\n").is_empty() {
                    // When popping the layer, probably should do the callback, rather than have an
                    // option.
                    app.pop_layer();
                    if let Some(mode) = self.prev_mode {
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
                if let Some(mode) = self.prev_mode {
                    app.mode = mode;
                }
            }
            _ => {
                self.text_area.input(Input::from(key_event));
            }
        }
        EventResult::Consumed
    }

    fn update_layout(&mut self, draw_area: Rect) {
        if self.full_width {
            self.draw_area = draw_area
        } else {
            self.draw_area = utils::centre_rect(
                Constraint::Percentage(70),
                Constraint::Length(self.text_area.lines().len() as u16 + 2),
                draw_area,
            );
        }
    }

    fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> EventResult {
        match mouse_event.kind {
            MouseEventKind::Down(..) => {}
            _ => {
                return EventResult::Consumed;
            }
        }

        let draw_area = self.draw_area;

        if !utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
            app.pop_layer();
            if let Some(mode) = self.prev_mode {
                app.mode = mode;
            }
            return EventResult::Consumed;
        }

        // Either we use inner on draw_area to exclude border, or this to include it
        // and set the border to jump to 0
        if draw_area.x == mouse_event.column {
            self.text_area
                .move_cursor(CursorMove::Jump(mouse_event.row - draw_area.y - 1, 0));
        } else if draw_area.y == mouse_event.row {
            self.text_area
                .move_cursor(CursorMove::Jump(0, mouse_event.column - draw_area.x - 1));
        } else {
            self.text_area.move_cursor(CursorMove::Jump(
                mouse_event.row - draw_area.y - 1,
                mouse_event.column - draw_area.x - 1,
            ));
        }
        EventResult::Consumed
    }
}

pub struct InputBoxBuilder {
    title: String,
    text_area: TextArea<'static>,
    callback: InputBoxCallback,
    error_callback: ErrorCallback,
    draw_area: Rect,
    prev_mode: Option<Mode>,
    full_width: bool,
}

impl Default for InputBoxBuilder {
    fn default() -> Self {
        InputBoxBuilder {
            title: String::default(),
            text_area: TextArea::default(),
            callback: Some(Box::new(|_app, _task| Ok(()))),
            error_callback: Box::new(|_app, _err| {}),
            draw_area: Rect::default(),
            prev_mode: None,
            full_width: false,
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
            prev_mode: self.prev_mode,
            full_width: self.full_width,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }

    pub fn fill(self, words: &str) -> Self {
        let words = words
            .split('\n')
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

    pub fn prev_mode(mut self, mode: Option<Mode>) -> Self {
        self.prev_mode = mode;
        self
    }

    pub fn save_mode(mut self, app: &mut App) -> Self {
        self.prev_mode = Some(app.mode);
        app.mode = Mode::Overlay;
        self
    }

    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }
}
