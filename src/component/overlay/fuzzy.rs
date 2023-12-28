use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use itertools::Itertools;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
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

pub struct FuzzyBox<'a> {
    draw_area: Rect,
    input: InputBox,
    active: Vec<usize>,
    list_draw_area: Rect,
    list_index: usize,
    options: Vec<DialogAction<'a>>,
    prev_mode: Option<Mode>,
}

impl FuzzyBox<'_> {
    fn generate_rect(&self, rect: Rect) -> Rect {
        // FIXME: consider using length of options.
        utils::centre_rect(Constraint::Percentage(70), Constraint::Percentage(80), rect)
    }
}

impl DrawableComponent for FuzzyBox<'_> {
    fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        self.input.draw(app, drawer);
        let mut list = List::new(
            self.active
                .iter()
                .map(|&id| ListItem::new(self.options[id].name.clone())) // NOTE: This should
                // probably be fine, as
                // there would have to be
                // a construction of a
                // Line every call anyway.
                .collect::<Vec<ListItem>>(),
        )
        .highlight_symbol(&app.theme.selected_cursor)
        .block(app.theme.styled_block("", app.theme.selected_border_colour));

        // FIXME: The colour does not show on the cursor if there is colour in the line :(
        if let Some(Some(opt)) = self
            .active
            .get(self.list_index)
            .map(|&id| self.options.get(id))
        {
            if opt.name.spans.iter().all(|f| f.style.fg.is_none()) {
                list = list.highlight_style(app.theme.highlight_dropdown_style())
            } else {
                list = list.highlight_style(Style::default().add_modifier(Modifier::BOLD))
            }
        }
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
                Mode::Overlay,
                self.active.len(),
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
            _ if app.theme.move_down_fuzzy.is_pressed(key_event) => {
                if self.active.is_empty() {
                    return EventResult::Consumed;
                }
                self.list_index = (self.list_index + 1).rem_euclid(self.active.len());
                EventResult::Consumed
            }
            KeyCode::Down => {
                if self.active.is_empty() {
                    return EventResult::Consumed;
                }
                self.list_index = (self.list_index + 1).rem_euclid(self.active.len());
                EventResult::Consumed
            }
            _ if app.theme.move_up_fuzzy.is_pressed(key_event) => {
                if self.active.is_empty() {
                    return EventResult::Consumed;
                }
                match self.list_index.checked_sub(1) {
                    Some(val) => self.list_index = val,
                    None => self.list_index = self.active.len() - 1,
                }
                EventResult::Consumed
            }
            KeyCode::Up => {
                if self.active.is_empty() {
                    return EventResult::Consumed;
                }
                match self.list_index.checked_sub(1) {
                    Some(val) => self.list_index = val,
                    None => self.list_index = self.active.len() - 1,
                }
                EventResult::Consumed
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
                    // FIXME: Might be better to store as a seperate variable for this.
                    let name = ele
                        .name
                        .spans
                        .iter()
                        .map(|sp| sp.content.clone())
                        .collect::<String>();
                    if name.to_ascii_lowercase().contains(&input) {
                        self.active.push(i)
                    }
                }
                e
            }
        }
    }
}

#[derive(Default)]
pub struct FuzzyBoxBuilder<'a> {
    draw_area: Rect,
    title: String,
    options: Vec<DialogAction<'a>>,
    prev_mode: Option<Mode>,
}

impl<'a> FuzzyBoxBuilder<'a> {
    pub fn build(self) -> FuzzyBox<'a> {
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

    pub fn options(mut self, options: Vec<DialogAction<'a>>) -> Self {
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
