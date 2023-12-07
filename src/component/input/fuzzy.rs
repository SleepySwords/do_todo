use crossterm::event::{KeyCode, KeyModifiers};
use itertools::Itertools;
use tui::{
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::App,
    draw::{DrawableComponent, EventResult},
    utils,
};

use super::{
    dialog::DialogAction,
    input_box::{InputBox, InputBoxBuilder},
};

pub struct FuzzyBox {
    pub draw_area: Rect,
    pub input: InputBox,
    pub active: Vec<usize>,
    pub list_draw_area: Rect,
    pub list_index: usize,
    pub options: Vec<DialogAction>,
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
                .map(|action| ListItem::new(Line::from(action)))
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
                .title("")
                .border_style(Style::default().fg(tui::style::Color::Green)),
        );

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

    fn key_event(
        &mut self,
        app: &mut crate::app::App,
        key_event: crossterm::event::KeyEvent,
    ) -> crate::draw::EventResult {
        let code = key_event.code;
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('n') => {
                    self.list_index += 1;
                    if self.list_index >= self.active.len() {
                        self.list_index = 0;
                    }
                    return EventResult::Consumed;
                }
                KeyCode::Char('p') => {
                    if self.list_index == 0 {
                        self.list_index = self.active.len() - 1;
                    } else {
                        self.list_index -= 1;
                    }
                    return EventResult::Consumed;
                }
                _ => {}
            }
        }
        match code {
            KeyCode::Enter => {
                app.pop_layer();
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
}

impl FuzzyBoxBuilder {
    pub fn build(self) -> FuzzyBox {
        let active = (0..self.options.len()).collect_vec();
        FuzzyBox {
            draw_area: self.draw_area,
            options: self.options,
            input: InputBoxBuilder::default().full_width(true).title(self.title).build(),
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

    pub fn title(mut self, title: String) -> Self {
        self.title = title;
        self
    }
}
