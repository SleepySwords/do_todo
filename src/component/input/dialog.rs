use crossterm::event::{KeyCode, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    utils::{self, handle_mouse_movement},
};

type DialogCallback = Box<dyn FnOnce(&mut App)>;

pub struct DialogAction {
    pub name: String,
    pub function: Option<DialogCallback>,
}

impl DialogAction {
    pub fn new<F: 'static>(name: String, function: F) -> DialogAction
    where
        F: FnOnce(&mut App),
    {
        DialogAction {
            name,
            function: Some(Box::new(function)),
        }
    }
}

pub struct DialogBox {
    draw_area: Rect,
    title: String,
    index: usize,
    options: Vec<DialogAction>,
    prev_mode: Option<Mode>,
    full_width: bool,
}

impl DialogBox {
    fn generate_rect(&self) -> Rect {
        utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            self.draw_area,
        )
    }
}

impl DrawableComponent for DialogBox {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let draw_area = if self.full_width {
            self.draw_area
        } else {
            self.generate_rect()
        };
        let list = List::new(
            self.options
                .iter()
                .map(|action| ListItem::new(Line::from(action.name.as_str())))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(tui::style::Color::LightMagenta),
        )
        .block(utils::generate_default_block(
            self.title.as_str(),
            Mode::Overlay,
            app,
        ));

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        drawer.draw_widget(Clear, draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, draw_area);
    }

    fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let key_code = key_event.code;
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return EventResult::Consumed;
            }
        }
        utils::handle_key_movement(key_code, &mut self.index, self.options.len());
        match key_code {
            KeyCode::Enter => {
                app.pop_layer();
                if let Some(mode) = self.prev_mode {
                    app.mode = mode;
                }
                if let Some(opt) = self.options.get_mut(self.index) {
                    if let Some(callback) = opt.function.take() {
                        (callback)(app);
                    }
                }
            }
            KeyCode::Esc => {
                // TODO: May be better to have a custom escape function
                app.pop_layer();
                if let Some(mode) = self.prev_mode {
                    app.mode = mode;
                }
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
            return handle_mouse_movement(
                app,
                draw_area,
                None,
                self.options.len(),
                &mut self.index,
                mouse_event,
            );
        }

        if let MouseEventKind::Down(_) = mouse_event.kind {
            app.pop_layer();
            if let Some(mode) = self.prev_mode {
                app.mode = mode;
            }
        }
        EventResult::Consumed
    }

    fn update_layout(&mut self, area: Rect) {
        self.draw_area = area;
    }
}

#[derive(Default)]
pub struct DialogBoxBuilder {
    draw_area: Rect,
    title: String,
    index: usize,
    options: Vec<DialogAction>,
    prev_mode: Option<Mode>,
    full_width: bool,
}

impl DialogBoxBuilder {
    pub fn build(self) -> DialogBox {
        DialogBox {
            draw_area: self.draw_area,
            title: self.title,
            index: self.index,
            options: self.options,
            prev_mode: self.prev_mode,
            full_width: self.full_width,
        }
    }

    pub fn add_option(mut self, dialog_action: DialogAction) -> Self {
        self.options.push(dialog_action);
        self
    }

    pub fn options(mut self, options: Vec<DialogAction>) -> Self {
        self.options = options;
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
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
