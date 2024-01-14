use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Clear},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app::{App, Mode},
    config::Config,
    draw::{Action, Drawer, PostEvent},
    error::AppError,
    utils,
};

use super::{fuzzy::FuzzyBox, Overlay};

type InputBoxCallback = Option<Box<dyn FnOnce(&mut App, String) -> Result<PostEvent, AppError>>>;
type ErrorCallback = Box<dyn Fn(&mut App, AppError) -> PostEvent>;

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
            error_callback: Box::new(|_, _| return PostEvent::noop(false)),
            draw_area: Rect::default(),
            prev_mode: None,
            full_width: false,
        }
    }

    pub fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }

    pub fn draw(&self, app: &App, drawer: &mut Drawer) {
        Self::draw_input_box(
            &app.config,
            self.draw_area,
            &self.text_area,
            &self.title,
            drawer,
        );
    }

    pub fn draw_input_box(
        config: &Config,
        draw_area: Rect,
        text_area: &TextArea,
        title: &str,
        drawer: &mut Drawer,
    ) {
        let widget = text_area.widget();
        let boxes = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(config.selected_border_colour))
            .border_type(config.border_type)
            .title(title.as_ref());
        let box_area = boxes.inner(draw_area);

        drawer.draw_widget(Clear, draw_area);
        drawer.draw_widget(boxes, draw_area);
        drawer.draw_widget(widget, box_area);
    }

    pub fn key_event(&mut self, app: &mut App, key_event: KeyEvent) -> PostEvent {
        match key_event.code {
            KeyCode::Enter => {
                if !self.text_area.lines().join("\n").is_empty() {
                    // When popping the layer, probably should do the callback, rather than have an
                    // option.
                    return PostEvent::pop_overlay(false, |app: &mut App, overlay| {
                        if let Overlay::Input(InputBox {
                            mut callback,
                            prev_mode,
                            text_area,
                            error_callback,
                            ..
                        }) = overlay
                        {
                            if let Some(mode) = prev_mode {
                                app.mode = mode;
                            }

                            return if let Some(callback) = callback.take() {
                                let err = (callback)(app, text_area.lines().join("\n"));
                                match err {
                                    Ok(post_event) => post_event,
                                    Err(err) => (error_callback)(app, err),
                                }
                            } else {
                                PostEvent::noop(false)
                            };
                        }
                        return PostEvent::noop(false);
                    });
                }
            }
            KeyCode::Tab => {
                self.text_area.insert_newline();
            }
            KeyCode::Esc => {
                return PostEvent::pop_overlay(false, |app: &mut App, overlay| {
                    if let Overlay::Input(InputBox { prev_mode, .. }) = overlay {
                        if let Some(mode) = prev_mode {
                            app.mode = mode;
                        }
                    }
                    return PostEvent::noop(false);
                })
            }
            _ => {
                self.text_area.input(Input::from(key_event));
            }
        }
        PostEvent {
            propegate_further: false,
            action: Action::Noop,
        }
    }

    pub fn update_layout(&mut self, draw_area: Rect) {
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

    pub fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> PostEvent {
        match mouse_event.kind {
            MouseEventKind::Down(..) => {}
            _ => {
                return PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                };
            }
        }

        let draw_area = self.draw_area;

        if !utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
            return PostEvent::pop_overlay(false, |app: &mut App, overlay| {
                if let Overlay::Fuzzy(FuzzyBox { prev_mode, .. }) = overlay {
                    if let Some(mode) = prev_mode {
                        app.mode = mode;
                    }
                }
                return PostEvent::noop(false);
            });
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
        PostEvent {
            propegate_further: false,
            action: Action::Noop,
        }
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
            callback: Some(Box::new(|_app, _task| Ok(PostEvent::noop(false)))),
            error_callback: Box::new(|_app, _err| return PostEvent::noop(false)),
            draw_area: Rect::default(),
            prev_mode: None,
            full_width: false,
        }
    }
}

impl InputBoxBuilder {
    pub fn build(self) -> Overlay<'static> {
        Overlay::Input(InputBox {
            title: self.title,
            text_area: self.text_area,
            callback: self.callback,
            error_callback: self.error_callback,
            draw_area: self.draw_area,
            prev_mode: self.prev_mode,
            full_width: self.full_width,
        })
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
        T: FnOnce(&mut App, String) -> Result<PostEvent, AppError>,
    {
        self.callback = Some(Box::new(callback));
        self
    }

    pub fn error_callback<T: 'static>(mut self, error_callback: T) -> Self
    where
        T: Fn(&mut App, AppError) -> PostEvent,
    {
        self.error_callback = Box::new(error_callback);
        self
    }

    pub fn save_mode(mut self, app: &mut App) -> Self {
        self.prev_mode = Some(app.mode);
        app.mode = Mode::Overlay;
        self
    }
}
