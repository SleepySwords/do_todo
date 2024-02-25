use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, Borders, Clear},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app::{App, Mode},
    framework::{
        component::{Component, Drawer},
        event::{AppEvent, PostEvent},
    },
    utils,
};

type InputBoxCallback = Option<Box<dyn Fn(&mut App, String) -> PostEvent>>;

pub struct InputBox {
    pub draw_area: Rect,
    title: String,
    text_area: TextArea<'static>,
    on_submit: InputBoxCallback,
    prev_mode: Option<Mode>,
    full_width: bool,
}

impl InputBox {
    pub fn text(&self) -> String {
        self.text_area.lines().join("\n")
    }
}

impl Component for InputBox {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let widget = self.text_area.widget();
        let boxes = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.config.selected_border_colour))
            .border_type(app.config.border_type)
            .title(self.title.as_ref());
        let box_area = boxes.inner(self.draw_area);

        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_widget(boxes, self.draw_area);
        drawer.draw_widget(widget, box_area);
    }

    fn key_event(&mut self, _app: &mut App, key_event: KeyEvent) -> PostEvent {
        match key_event.code {
            KeyCode::Enter => {
                if self.text_area.lines().join("\n").is_empty() {
                    return PostEvent::noop(false);
                }
                return PostEvent::pop_layer(Some(AppEvent::Submit));
            }
            KeyCode::Tab => {
                self.text_area.insert_newline();
            }
            KeyCode::Esc => return PostEvent::pop_layer(Some(AppEvent::Cancel)),
            _ => {
                self.text_area.input(Input::from(key_event));
            }
        }
        PostEvent::noop(false)
    }

    fn mount(&mut self, app: &mut App) {
        self.prev_mode = Some(app.mode);
        app.mode = Mode::Overlay;
    }

    fn unmount(&mut self, app: &mut App, event: Option<AppEvent>) -> PostEvent {
        if let Some(mode) = self.prev_mode {
            app.mode = mode;
        }
        match event {
            Some(AppEvent::Submit) => {
                if let Some(callback) = &self.on_submit {
                    return (callback)(app, self.text_area.lines().join("\n"));
                }
            }
            Some(AppEvent::Cancel) => {}
            _ => {}
        }
        PostEvent::noop(false)
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

    fn mouse_event(&mut self, _: &mut App, mouse_event: MouseEvent) -> PostEvent {
        match mouse_event.kind {
            MouseEventKind::Down(..) => {}
            _ => return PostEvent::noop(false),
        }

        let draw_area = self.draw_area;

        if !utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
            return PostEvent::pop_layer(Some(AppEvent::Cancel));
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
        PostEvent::noop(false)
    }
}

pub struct InputBoxBuilder {
    title: String,
    text_area: TextArea<'static>,
    on_submit: InputBoxCallback,
    draw_area: Rect,
    full_width: bool,
}

impl Default for InputBoxBuilder {
    fn default() -> Self {
        InputBoxBuilder {
            title: String::default(),
            text_area: TextArea::default(),
            on_submit: Some(Box::new(|_app, _task| PostEvent::noop(false))),
            draw_area: Rect::default(),
            full_width: false,
        }
    }
}

impl InputBoxBuilder {
    pub fn build(self) -> InputBox {
        InputBox {
            title: self.title,
            text_area: self.text_area,
            on_submit: self.on_submit,
            draw_area: self.draw_area,
            prev_mode: None,
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

    pub fn on_submit<T: 'static>(mut self, callback: T) -> Self
    where
        T: Fn(&mut App, String) -> PostEvent,
    {
        self.on_submit = Some(Box::new(callback));
        self
    }
}
