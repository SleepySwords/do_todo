use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};
use itertools::Itertools;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::Line,
    widgets::{Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    utils::{self, handle_mouse_movement},
};

use super::{
    dialog::DialogAction,
    input_box::{InputBox, InputBoxBuilder},
};

pub struct FuzzyBox {
    draw_area: Rect,
    input: InputBox,
    active: Vec<usize>,
    list_draw_area: Rect,
    list_index: usize,
    options: Vec<DialogAction>,
    prev_mode: Option<Mode>,
}

impl FuzzyBox {
    fn generate_rect(&self, rect: Rect) -> Rect {
        utils::centre_rect(Constraint::Percentage(70), Constraint::Percentage(80), rect)
    }
}

impl DrawableComponent for FuzzyBox {
    fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        self.input.draw(app, drawer);
        let list = List::new(
            self.active
                .iter()
                .map(|&id| self.options[id].name.as_str())
                .map(|name| ListItem::new(Line::from(name)))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(app.theme.highlight_dropdown_style())
        .block(app.theme.styled_block("", Color::Green));

        let mut list_state = ListState::default();
        list_state.select(Some(self.list_index));

        drawer.draw_widget(Clear, self.list_draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.list_draw_area);
    }

    fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = self.generate_rect(draw_area);
        let layout = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(80)])
            .split(self.draw_area);

        self.input.update_layout(layout[0]);
        self.list_draw_area = layout[1];
    }

    fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> EventResult {
        if utils::inside_rect((mouse_event.row, mouse_event.column), self.input.draw_area) {
            self.input.mouse_event(app, mouse_event)
        } else if utils::inside_rect((mouse_event.row, mouse_event.column), self.list_draw_area) {
            return handle_mouse_movement(
                app,
                self.list_draw_area,
                None,
                self.active.len(),
                &mut self.list_index,
                mouse_event,
            );
        } else {
            if let MouseEventKind::Down(_) = mouse_event.kind {
                app.pop_layer();
                if let Some(mode) = self.prev_mode {
                    app.mode = mode;
                }
            }
            EventResult::Consumed
        }
    }

    fn key_event(&mut self, app: &mut App, key_event: KeyEvent) -> EventResult {
        let code = key_event.code;
        match code {
            KeyCode::Char('n') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.list_index = (self.list_index + 1).rem_euclid(self.active.len());
                return EventResult::Consumed;
            }
            KeyCode::Down => {
                self.list_index = (self.list_index + 1).rem_euclid(self.active.len());
                return EventResult::Consumed;
            }
            KeyCode::Char('p') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                match self.list_index.checked_sub(1) {
                    Some(val) => self.list_index = val,
                    None => self.list_index = self.active.len() - 1,
                }
                return EventResult::Consumed;
            }
            KeyCode::Up => {
                match self.list_index.checked_sub(1) {
                    Some(val) => self.list_index = val,
                    None => self.list_index = self.active.len() - 1,
                }
                return EventResult::Consumed;
            }
            KeyCode::Enter => {
                app.pop_layer();
                if let Some(mode) = self.prev_mode {
                    app.mode = mode;
                }
                if let Some(Some(opt)) = self
                    .active
                    .get(self.list_index)
                    .map(|&id| self.options.get_mut(id))
                {
                    if let Some(callback) = opt.function.take() {
                        (callback)(app);
                    }
                }
                EventResult::Consumed
            }
            _ => {
                let e = self.input.key_event(app, key_event);
                let input = self.input.text().to_ascii_lowercase();
                self.active.clear();
                self.list_index = 0;
                for (i, ele) in self.options.iter().enumerate() {
                    if ele.name.to_ascii_lowercase().contains(&input) {
                        self.active.push(i)
                    }
                }
                e
            }
        }
    }
}

#[derive(Default)]
pub struct FuzzyBoxBuilder {
    draw_area: Rect,
    title: String,
    options: Vec<DialogAction>,
    prev_mode: Option<Mode>,
}

impl FuzzyBoxBuilder {
    pub fn build(self) -> FuzzyBox {
        let active = (0..self.options.len()).collect_vec();
        FuzzyBox {
            draw_area: self.draw_area,
            options: self.options,
            prev_mode: self.prev_mode,
            input: InputBoxBuilder::default()
                .full_width(true)
                .title(self.title)
                .prev_mode(self.prev_mode)
                .build(),
            active,
            list_draw_area: Rect::default(),
            list_index: 0,
        }
    }

    pub fn add_option<F: 'static>(self, name: String, function: F) -> Self
    where
        F: FnOnce(&mut App),
    {
        self.add_dialog_action(DialogAction::new(name, function))
    }

    pub fn add_dialog_action(mut self, dialog_action: DialogAction) -> Self {
        self.options.push(dialog_action);
        self
    }

    pub fn options(mut self, options: Vec<DialogAction>) -> Self {
        self.options = options;
        self
    }

    pub fn save_mode(mut self, app: &mut App) -> Self {
        self.prev_mode = Some(app.mode);
        app.mode = Mode::Overlay;
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}
