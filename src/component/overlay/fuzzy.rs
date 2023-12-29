use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use itertools::Itertools;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Clear, List, ListItem, ListState},
};
use tui_textarea::{CursorMove, Input, TextArea};

use crate::{
    app::{App, Mode},
    draw::EventResult,
    utils::{self, handle_mouse_movement},
};

use super::{dialog::DialogAction, input_box::InputBox, Overlay};

pub struct FuzzyBox<'a> {
    draw_area: Rect,
    list_draw_area: Rect,
    text_draw_area: Rect,
    title: String,
    text_area: TextArea<'static>,
    active: Vec<usize>,
    pub index: usize,
    options: Vec<DialogAction<'a>>,
    prev_mode: Option<Mode>,
}

impl FuzzyBox<'_> {
    fn generate_rect(&self, rect: Rect) -> Rect {
        // FIXME: consider using length of options.
        utils::centre_rect(Constraint::Percentage(70), Constraint::Percentage(80), rect)
    }

    pub fn key_event(app: &mut App, key_event: KeyEvent) -> EventResult {
        let code = key_event.code;
        let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.last_mut() else {
            return EventResult::Ignored;
        };
        match code {
            _ if app.theme.move_down_fuzzy.is_pressed(key_event) => {
                if fuzzy.active.is_empty() {
                    return EventResult::Consumed;
                }
                fuzzy.index = (fuzzy.index + 1).rem_euclid(fuzzy.active.len());
                EventResult::Consumed
            }
            KeyCode::Down => {
                if fuzzy.active.is_empty() {
                    return EventResult::Consumed;
                }
                fuzzy.index = (fuzzy.index + 1).rem_euclid(fuzzy.active.len());
                EventResult::Consumed
            }
            _ if app.theme.move_up_fuzzy.is_pressed(key_event) => {
                if fuzzy.active.is_empty() {
                    return EventResult::Consumed;
                }
                match fuzzy.index.checked_sub(1) {
                    Some(val) => fuzzy.index = val,
                    None => fuzzy.index = fuzzy.active.len() - 1,
                }
                EventResult::Consumed
            }
            KeyCode::Up => {
                if fuzzy.active.is_empty() {
                    return EventResult::Consumed;
                }
                match fuzzy.index.checked_sub(1) {
                    Some(val) => fuzzy.index = val,
                    None => fuzzy.index = fuzzy.active.len() - 1,
                }
                EventResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(Overlay::Fuzzy(mut fuzzy)) = app.overlays.pop() {
                    if let Some(mode) = fuzzy.prev_mode {
                        app.mode = mode;
                    }
                    if let Some(Some(opt)) = fuzzy
                        .active
                        .get(fuzzy.index)
                        .map(|&id| fuzzy.options.get_mut(id))
                    {
                        if let Some(callback) = opt.function.take() {
                            (callback)(app);
                        }
                    }
                }
                EventResult::Consumed
            }
            KeyCode::Esc => {
                if let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.pop() {
                    if let Some(mode) = fuzzy.prev_mode {
                        app.mode = mode;
                    }
                }
                EventResult::Consumed
            }
            _ => {
                fuzzy.text_area.input(Input::from(key_event));
                let input = fuzzy.text_area.lines().join("\n").to_ascii_lowercase();
                fuzzy.active.clear();
                fuzzy.index = 0;
                for (i, ele) in fuzzy.options.iter().enumerate() {
                    // FIXME: Might be better to store as a seperate variable for this.
                    let name = ele
                        .name
                        .spans
                        .iter()
                        .map(|sp| sp.content.clone())
                        .collect::<String>();
                    if name.to_ascii_lowercase().contains(&input) {
                        fuzzy.active.push(i)
                    }
                }
                EventResult::Consumed
            }
        }
    }

    pub fn draw(app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.last() else {
            return;
        };

        InputBox::draw_input_box(
            &app.theme,
            fuzzy.text_draw_area,
            &fuzzy.text_area,
            fuzzy.title.as_ref(),
            drawer,
        );

        let mut list = List::new(
            fuzzy
                .active
                .iter()
                .map(|&id| ListItem::new(fuzzy.options[id].name.clone())) // NOTE: This should
                // probably be fine, as
                // there would have to be
                // a construction of a
                // Line every call anyway.
                .collect::<Vec<ListItem>>(),
        )
        .highlight_symbol(&app.theme.selected_cursor)
        .block(app.theme.styled_block("", app.theme.selected_border_colour));

        // FIXME: The colour does not show on the cursor if there is colour in the line :(
        if let Some(Some(opt)) = fuzzy
            .active
            .get(fuzzy.index)
            .map(|&id| fuzzy.options.get(id))
        {
            if opt.name.spans.iter().all(|f| f.style.fg.is_none()) {
                list = list.highlight_style(app.theme.highlight_dropdown_style())
            } else {
                list = list.highlight_style(Style::default().add_modifier(Modifier::BOLD))
            }
        }
        let mut list_state = ListState::default();
        list_state.select(Some(fuzzy.index));

        drawer.draw_widget(Clear, fuzzy.list_draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, fuzzy.list_draw_area);
    }

    pub fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = self.generate_rect(draw_area);
        let layout = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(80)])
            .split(self.draw_area);

        self.text_draw_area = layout[0];
        self.list_draw_area = layout[1];
    }

    pub fn mouse_event(app: &mut App, mouse_event: MouseEvent) -> EventResult {
        let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.last_mut() else {
            return EventResult::Ignored;
        };
        if utils::inside_rect((mouse_event.row, mouse_event.column), fuzzy.text_draw_area) {
            match mouse_event.kind {
                MouseEventKind::Down(..) => {}
                _ => {
                    return EventResult::Consumed;
                }
            }

            let draw_area = fuzzy.text_draw_area;

            if !utils::inside_rect((mouse_event.row, mouse_event.column), draw_area) {
                if let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.pop() {
                    if let Some(mode) = fuzzy.prev_mode {
                        app.mode = mode;
                    }
                }
                return EventResult::Consumed;
            }

            // Either we use inner on draw_area to exclude border, or this to include it
            // and set the border to jump to 0
            if draw_area.x == mouse_event.column {
                fuzzy
                    .text_area
                    .move_cursor(CursorMove::Jump(mouse_event.row - draw_area.y - 1, 0));
            } else if draw_area.y == mouse_event.row {
                fuzzy
                    .text_area
                    .move_cursor(CursorMove::Jump(0, mouse_event.column - draw_area.x - 1));
            } else {
                fuzzy.text_area.move_cursor(CursorMove::Jump(
                    mouse_event.row - draw_area.y - 1,
                    mouse_event.column - draw_area.x - 1,
                ));
            }
            EventResult::Consumed
        } else if utils::inside_rect((mouse_event.row, mouse_event.column), fuzzy.list_draw_area) {
            let ar = fuzzy.list_draw_area;
            let size = fuzzy.active.len();
            return handle_mouse_movement(app, ar, Mode::Overlay, size, mouse_event);
        } else {
            if let MouseEventKind::Down(_) = mouse_event.kind {
                if let Some(Overlay::Fuzzy(fuzzy)) = app.overlays.pop() {
                    if let Some(mode) = fuzzy.prev_mode {
                        app.mode = mode;
                    }
                }
            }
            EventResult::Consumed
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
    pub fn build(self) -> Overlay<'a> {
        let active = (0..self.options.len()).collect_vec();
        Overlay::Fuzzy(FuzzyBox {
            draw_area: self.draw_area,
            text_draw_area: Rect::default(),
            list_draw_area: Rect::default(),
            index: 0,
            options: self.options,
            prev_mode: self.prev_mode,
            title: self.title,
            text_area: TextArea::default(),
            active,
        })
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
