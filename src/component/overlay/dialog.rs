use crossterm::event::{KeyCode, MouseEventKind};

use tui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    draw::{Action, PostEvent},
    utils::{self, handle_mouse_movement},
};

use super::{fuzzy::FuzzyBox, Overlay};

type DialogCallback = Box<dyn FnOnce(&mut App) -> PostEvent>;

pub struct DialogAction<'a> {
    pub name: Line<'a>,
    pub function: Option<DialogCallback>,
}

impl DialogAction<'_> {
    pub fn new<F: 'static>(name: String, function: F) -> DialogAction<'static>
    where
        F: FnOnce(&mut App) -> PostEvent,
    {
        DialogAction {
            name: Line::raw(name),
            function: Some(Box::new(function)),
        }
    }

    pub fn styled<F: 'static>(name: String, style: Style, function: F) -> DialogAction<'static>
    where
        F: FnOnce(&mut App) -> PostEvent,
    {
        DialogAction {
            name: Line::styled(name, style),
            function: Some(Box::new(function)),
        }
    }
}

pub struct DialogBox<'a> {
    draw_area: Rect,
    title: String,
    pub index: usize,
    options: Vec<DialogAction<'a>>,
    prev_mode: Option<Mode>,
}

impl DialogBox<'_> {
    pub fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let mut list = List::new(
            self.options
                .iter()
                .map(|action| action.name.clone())
                .map(ListItem::new)
                .collect::<Vec<ListItem>>(),
        )
        .highlight_symbol(&app.config.selected_cursor)
        .block(utils::ui::generate_default_block(
            app,
            self.title.as_str(),
            Mode::Overlay,
        ));

        if self.options[self.index]
            .name
            .spans
            .iter()
            .all(|f| f.style.fg.is_none())
        {
            list = list.highlight_style(app.config.highlight_dropdown_style())
        } else {
            list = list.highlight_style(Style::default().add_modifier(Modifier::BOLD))
        }

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        drawer.draw_widget(Clear, self.draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
    }

    pub fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> PostEvent {
        let key_code = key_event.code;
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return PostEvent {
                    propegate_further: false,
                    action: Action::Noop,
                };
            }
        }
        utils::handle_key_movement(&app.config, key_event, &mut self.index, self.options.len());
        match key_code {
            KeyCode::Enter => {
                return PostEvent::pop_overlay(false, |app, overlay| {
                    if let Overlay::Dialog(DialogBox {
                        index,
                        mut options,
                        prev_mode,
                        ..
                    }) = overlay
                    {
                        if let Some(mode) = prev_mode {
                            app.mode = mode;
                        }
                        if let Some(opt) = options.get_mut(index) {
                            if let Some(callback) = opt.function.take() {
                                return (callback)(app);
                            }
                        }
                    }
                    PostEvent::noop(false)
                })
            }
            KeyCode::Esc => {
                return PostEvent::pop_overlay(false, |app, overlay| {
                    if let Overlay::Dialog(DialogBox { prev_mode, .. }) = overlay {
                        if let Some(mode) = prev_mode {
                            app.mode = mode;
                        }
                    }
                    PostEvent::noop(false)
                })
            }
            _ => {}
        }
        PostEvent {
            propegate_further: false,
            action: Action::Noop,
        }
    }

    pub fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        if utils::inside_rect((mouse_event.row, mouse_event.column), self.draw_area) {
            let draw_area = self.draw_area;
            let size = self.options.len();
            return handle_mouse_movement(
                &mut self.index,
                &mut app.mode,
                draw_area,
                Mode::Overlay,
                size,
                mouse_event,
            );
        }

        if let MouseEventKind::Down(_) = mouse_event.kind {
            return PostEvent::pop_overlay(false, |app, overlay| {
                if let Overlay::Fuzzy(FuzzyBox { prev_mode, .. }) = overlay {
                    if let Some(mode) = prev_mode {
                        app.mode = mode;
                    }
                }
                PostEvent::noop(false)
            });
        }
        PostEvent {
            propegate_further: false,
            action: Action::Noop,
        }
    }

    pub fn update_layout(&mut self, area: Rect) {
        self.draw_area = utils::centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            area,
        )
    }
}

#[derive(Default)]
pub struct DialogBoxBuilder<'a> {
    draw_area: Rect,
    title: String,
    index: usize,
    options: Vec<DialogAction<'a>>,
    prev_mode: Option<Mode>,
}

impl<'a> DialogBoxBuilder<'a> {
    pub fn build(self) -> Overlay<'a> {
        Overlay::Dialog(DialogBox {
            draw_area: self.draw_area,
            title: self.title,
            index: self.index,
            options: self.options,
            prev_mode: self.prev_mode,
        })
    }

    pub fn add_option(mut self, dialog_action: DialogAction<'a>) -> Self {
        self.options.push(dialog_action);
        self
    }

    pub fn options(mut self, options: Vec<DialogAction<'a>>) -> Self {
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
