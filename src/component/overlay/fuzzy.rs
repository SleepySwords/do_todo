use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
use itertools::Itertools;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    draw::{Action, PostEvent},
    utils::{self, handle_mouse_movement},
};

use super::{
    dialog::DialogAction,
    input_box::{InputBox, InputBoxBuilder},
    Overlay,
};

pub struct FuzzyBox<'a> {
    draw_area: Rect,
    list_draw_area: Rect,
    input_box: InputBox,
    active: Vec<usize>,
    pub index: usize,
    options: Vec<DialogAction<'a>>,
    pub prev_mode: Option<Mode>,
}

impl FuzzyBox<'_> {
    fn generate_rect(&self, rect: Rect) -> Rect {
        // FIXME: consider using length of options.
        utils::centre_rect(Constraint::Percentage(70), Constraint::Percentage(80), rect)
    }

    pub fn key_event(&mut self, app: &mut App, key_event: KeyEvent) -> PostEvent {
        let code = key_event.code;
        match code {
            _ if app.config.move_down_fuzzy.is_pressed(key_event) => {
                if self.active.is_empty() {
                    return PostEvent {
                        propegate_further: false,
                        action: Action::Noop,
                    };
                }
                self.index = (self.index + 1).rem_euclid(self.active.len());
                PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                }
            }
            KeyCode::Down => {
                if self.active.is_empty() {
                    return PostEvent {
                        propegate_further: false,
                        action: Action::Noop,
                    };
                }
                self.index = (self.index + 1).rem_euclid(self.active.len());
                PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                }
            }
            _ if app.config.move_up_fuzzy.is_pressed(key_event) => {
                if self.active.is_empty() {
                    return PostEvent {
                        propegate_further: false,
                        action: Action::Noop,
                    };
                }
                match self.index.checked_sub(1) {
                    Some(val) => self.index = val,
                    None => self.index = self.active.len() - 1,
                }
                PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                }
            }
            KeyCode::Up => {
                if self.active.is_empty() {
                    return PostEvent {
                        propegate_further: false,
                        action: Action::Noop,
                    };
                }
                match self.index.checked_sub(1) {
                    Some(val) => self.index = val,
                    None => self.index = self.active.len() - 1,
                }
                PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                }
            }
            KeyCode::Enter => {
                return PostEvent::pop_overlay(false, |app, overlay| {
                    if let Overlay::Fuzzy(FuzzyBox {
                        active,
                        index,
                        mut options,
                        prev_mode,
                        ..
                    }) = overlay
                    {
                        if let Some(mode) = prev_mode {
                            app.mode = mode;
                        }
                        if let Some(Some(opt)) = active.get(index).map(|&id| options.get_mut(id)) {
                            if let Some(callback) = opt.function.take() {
                                return (callback)(app);
                            }
                        }
                    }
                    PostEvent::noop(false)
                });
            }
            KeyCode::Esc => PostEvent::pop_overlay(false, |app, overlay| {
                if let Overlay::Fuzzy(FuzzyBox {
                    prev_mode: Some(mode),
                    ..
                }) = overlay
                {
                    app.mode = mode;
                }
                PostEvent::noop(false)
            }),
            _ => {
                self.input_box.key_event(app, key_event);
                let input = self.input_box.text().to_ascii_lowercase();
                self.active.clear();
                self.index = 0;
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
                PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                }
            }
        }
    }

    pub fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        self.input_box.draw(app, drawer);

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
        .highlight_symbol(&app.config.selected_cursor)
        .block(
            app.config
                .styled_block("", app.config.selected_border_colour),
        );

        // FIXME: The colour does not show on the cursor if there is colour in the line :(
        if let Some(Some(opt)) = self.active.get(self.index).map(|&id| self.options.get(id)) {
            if opt.name.spans.iter().all(|f| f.style.fg.is_none()) {
                list = list.highlight_style(app.config.highlight_dropdown_style())
            } else {
                list = list.highlight_style(Style::default().add_modifier(Modifier::BOLD))
            }
        }
        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        drawer.draw_widget(Clear, self.list_draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.list_draw_area);
    }

    pub fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = self.generate_rect(draw_area);
        let layout = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(80)])
            .split(self.draw_area);

        self.input_box.draw_area = layout[0];
        self.list_draw_area = layout[1];
    }

    pub fn mouse_event(&mut self, app: &mut App, mouse_event: MouseEvent) -> PostEvent {
        if utils::inside_rect(
            (mouse_event.row, mouse_event.column),
            self.input_box.draw_area,
        ) {
            self.input_box.mouse_event(app, mouse_event)
        } else if utils::inside_rect((mouse_event.row, mouse_event.column), self.list_draw_area) {
            let ar = self.list_draw_area;
            let size = self.active.len();
            return handle_mouse_movement(
                &mut self.index,
                &mut app.mode,
                ar,
                Mode::Overlay,
                size,
                mouse_event,
            );
        } else {
            if let MouseEventKind::Down(_) = mouse_event.kind {
                return PostEvent::pop_overlay(false, |app: &mut App, overlay| {
                    if let Overlay::Fuzzy(FuzzyBox {
                        prev_mode: Some(mode),
                        ..
                    }) = overlay
                    {
                        app.mode = mode;
                    }
                    PostEvent::noop(false)
                });
            }
            PostEvent {
                propegate_further: false,
                action: Action::Noop,
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
    pub fn build(self) -> Overlay<'a> {
        let active = (0..self.options.len()).collect_vec();
        Overlay::Fuzzy(FuzzyBox {
            draw_area: self.draw_area,
            input_box: InputBoxBuilder::default().title(self.title).build(),
            list_draw_area: Rect::default(),
            index: 0,
            options: self.options,
            prev_mode: self.prev_mode,
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
