use crossterm::event::{KeyCode, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    text::Line,
    widgets::{Clear, List, ListItem, ListState},
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
}

impl DrawableComponent for DialogBox {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let list = List::new(
            self.options
                .iter()
                .map(|action| action.name.as_str())
                .map(|action| ListItem::new(Line::from(action)))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(app.theme.highlight_dropdown_style())
        .block(utils::ui::generate_default_block(
            app,
            self.title.as_str(),
            Mode::Overlay,
        ));

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
    }

    fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let key_code = key_event.code;
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return EventResult::Consumed;
            }
        }
        utils::handle_key_movement(&app.theme, key_event, &mut self.index, self.options.len());
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
        if utils::inside_rect((mouse_event.row, mouse_event.column), self.draw_area) {
            return handle_mouse_movement(
                app,
                self.draw_area,
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
        self.draw_area = utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            area,
        )
    }
}

#[derive(Default)]
pub struct DialogBoxBuilder {
    draw_area: Rect,
    title: String,
    index: usize,
    options: Vec<DialogAction>,
    prev_mode: Option<Mode>,
}

impl DialogBoxBuilder {
    pub fn build(self) -> DialogBox {
        DialogBox {
            draw_area: self.draw_area,
            title: self.title,
            index: self.index,
            options: self.options,
            prev_mode: self.prev_mode,
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
}
