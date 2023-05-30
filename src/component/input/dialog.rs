use crossterm::event::{KeyCode, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::App,
    utils,
    view::{DrawableComponent, EventResult},
};

pub struct DialogAction {
    name: String,
    function: Box<dyn Fn(&mut App)>,
}

impl DialogAction {
    pub fn new<F: 'static>(name: String, function: F) -> DialogAction
    where
        F: Fn(&mut App),
    {
        DialogAction {
            name,
            function: Box::new(function),
        }
    }
}

pub struct DialogBox {
    draw_area: Rect,
    title: String,
    index: usize,
    pub options: Vec<DialogAction>,
}

impl DialogBox {
    pub fn new(title: String, options: Vec<DialogAction>) -> DialogBox {
        if options.is_empty() {
            panic!("The size of the options is 0");
        }
        DialogBox {
            draw_area: Rect::default(),
            title,
            index: 0,
            options,
        }
    }

    fn generate_rect(&self) -> Rect {
        utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            self.draw_area,
        )
    }
}

impl DrawableComponent for DialogBox {
    fn draw(&self, app: &App, _: Rect, drawer: &mut crate::view::Drawer) {
        let draw_area = self.generate_rect();
        let list = List::new(
            self.options
                .iter()
                .map(|action| ListItem::new(Spans::from(action.name.as_str())))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(tui::style::Color::LightMagenta),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .title(self.title.as_str())
                .border_style(Style::default().fg(tui::style::Color::Green)),
        );

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        drawer.draw_widget(Clear, draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, draw_area);
    }

    fn key_pressed(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let key_code = key_event.code;
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return EventResult::Consumed;
            }
        }
        utils::handle_movement(key_code, &mut self.index, self.options.len());
        match key_code {
            KeyCode::Enter => {
                app.pop_layer();
                (self.options[self.index].function)(app);
            }
            KeyCode::Esc => {
                // May be better to have a custom escape function
                app.pop_layer();
            }
            _ => {}
        }
        EventResult::Consumed
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        let draw_area = self.generate_rect();
        if utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
            if let MouseEventKind::ScrollDown = mouse_event.kind {
                if self.index < self.options.len() - 1 {
                    self.index += 1;
                }
            }
            if let MouseEventKind::ScrollUp = mouse_event.kind {
                if self.index > 0 {
                    self.index -= 1;
                }
            }
            if let MouseEventKind::Down(_) = mouse_event.kind {
                let i = (mouse_event.row - draw_area.y) as usize;
                if i == 0 || i > self.options.len() {
                    return EventResult::Consumed;
                }
                self.index = i - 1usize;
            }
            return EventResult::Consumed;
        }

        if let MouseEventKind::Down(_) = mouse_event.kind {
            app.pop_layer();
        }
        EventResult::Consumed
    }

    fn update_layout(&mut self, area: Rect) {
        self.draw_area = area;
    }
}
