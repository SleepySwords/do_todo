use crossterm::event::{MouseEvent, MouseEventKind};
use tui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    utils::{self, centre_rect},
};

type MessageCallback = dyn FnOnce(&mut App);

pub struct MessageBox {
    title: String,
    callback: Option<Box<MessageCallback>>,
    message: Vec<String>,
    colour: Color,
    selected_index: usize,
    mode_to_restore: Option<Mode>,
    draw_area: Rect,
}

impl MessageBox {
    pub fn new<T: FnOnce(&mut App) + 'static>(
        title: String,
        callback: T,
        words: String,
        colour: Color,
        selected_index: usize,
    ) -> MessageBox {
        MessageBox {
            title,
            callback: Some(Box::new(callback)),
            message: words
                .split('\n')
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            colour,
            selected_index,
            mode_to_restore: None,
            draw_area: Rect::default(),
        }
    }

    pub fn new_by_list<T: Fn(&mut App) + 'static>(
        title: String,
        draw_area: Rect,
        callback: T,
        words: Vec<String>,
        colour: Color,
        selected_index: usize,
    ) -> MessageBox {
        MessageBox {
            title,
            callback: Some(Box::new(callback)),
            message: words,
            colour,
            selected_index,
            mode_to_restore: None,
            draw_area,
        }
    }

    pub fn save_mode(mut self, app: &mut App) -> Self {
        self.mode_to_restore = Some(app.mode);
        app.mode = Mode::Overlay;
        self
    }
}

impl DrawableComponent for MessageBox {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
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
                .border_type(app.theme.border_style)
                .border_style(style)
                .title(self.title.as_ref()),
        );
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));
        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
    }

    fn key_event(&mut self, app: &mut App, _: crossterm::event::KeyEvent) -> EventResult {
        app.pop_layer();
        if let Some(mode) = self.mode_to_restore {
            app.mode = mode;
        }
        if let Some(callback) = self.callback.take() {
            (callback)(app);
        }
        crate::draw::EventResult::Consumed
    }

    fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> EventResult {
        if let MouseEventKind::Down(..) = mouse_event.kind {
            if !utils::inside_rect((mouse_event.row, mouse_event.column), self.draw_area) {
                app.pop_layer();
                if let Some(mode) = self.mode_to_restore {
                    app.mode = mode;
                }
                if let Some(callback) = self.callback.take() {
                    (callback)(app);
                }
            }
        }
        EventResult::Consumed
    }

    fn update_layout(&mut self, draw_area: Rect) {
        let height = ((self.message.len() + 2) as u16)
            .min(Constraint::Percentage(70).apply(draw_area.height));
        self.draw_area = centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(height),
            draw_area,
        );
    }
}
