use crossterm::event::{MouseEvent, MouseEventKind};
use tui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    framework::{
        component::{Component, Drawer},
        event::{AppEvent, PostEvent},
    },
    utils::{self, centre_rect},
};

type MessageCallback = dyn Fn(&mut App) -> PostEvent;

pub struct MessageBox {
    title: String,
    on_close: Option<Box<MessageCallback>>,
    message: Vec<String>,
    colour: Color,
    selected_index: usize,
    pub prev_mode: Option<Mode>,
    draw_area: Rect,
}

impl MessageBox {
    pub fn new<T: Fn(&mut App) -> PostEvent + 'static>(
        title: String,
        callback: T,
        words: String,
        colour: Color,
        selected_index: usize,
    ) -> MessageBox {
        MessageBox {
            title,
            on_close: Some(Box::new(callback)),
            message: words
                .split('\n')
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            colour,
            selected_index,
            prev_mode: None,
            draw_area: Rect::default(),
        }
    }
}

impl Component for MessageBox {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let style = Style::default().fg(self.colour);
        let text = self
            .message
            .iter()
            .map(|msg| ListItem::new(Span::styled(msg, style)))
            .collect::<Vec<ListItem>>();
        // Add multiline support.
        let list = List::new(text);
        let list = list.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.config.border_type)
                .border_style(style)
                .title(self.title.as_ref()),
        );
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));
        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
    }

    fn key_event(&mut self, _: &mut App, _: crossterm::event::KeyEvent) -> PostEvent {
        PostEvent::pop_layer(None)
    }

    fn mount(&mut self, app: &mut App) {
        self.prev_mode = Some(app.mode);
        app.mode = Mode::Overlay;
    }

    fn unmount(&mut self, app: &mut App, _: Option<AppEvent>) -> PostEvent {
        if let Some(mode) = self.prev_mode {
            app.mode = mode;
        }
        if let Some(callback) = self.on_close.take() {
            return (callback)(app);
        }
        PostEvent::noop(false)
    }

    fn mouse_event(&mut self, _app: &mut App, mouse_event: MouseEvent) -> PostEvent {
        if let MouseEventKind::Down(..) = mouse_event.kind {
            if !utils::inside_rect((mouse_event.row, mouse_event.column), self.draw_area) {
                return PostEvent::pop_layer(None);
            }
        }
        PostEvent::noop(false)
    }

    fn update_layout(&mut self, draw_area: Rect) {
        let height = ((self.message.len() + 2) as u16).min((draw_area.height) * 70 / 100);
        self.draw_area = centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(height),
            draw_area,
        );
    }
}

#[derive(Default)]
pub struct MessageBoxBuilder {
    title: String,
    on_close: Option<Box<MessageCallback>>,
    message: Vec<String>,
    colour: Color,
    selected_index: usize,
    draw_area: Rect,
}

impl MessageBoxBuilder {
    pub fn build(self) -> MessageBox {
        MessageBox {
            title: self.title,
            on_close: self.on_close,
            draw_area: self.draw_area,
            prev_mode: None,
            colour: self.colour,
            selected_index: self.selected_index,
            message: self.message,
        }
    }

    pub fn title<T: Into<String>>(mut self, title: T) -> Self {
        self.title = title.into();
        self
    }

    pub fn message<T: Into<String>>(mut self, message: T) -> Self {
        self.message = message
            .into()
            .split('\n')
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        self
    }

    pub fn colour(mut self, colour: Color) -> Self {
        self.colour = colour;
        self
    }

    pub fn on_close<T>(mut self, callback: T) -> Self
    where
        T: Fn(&mut App) -> PostEvent + 'static,
    {
        self.on_close = Some(Box::new(callback));
        self
    }
}
