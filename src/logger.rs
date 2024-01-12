use chrono::NaiveTime;
use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    draw::{Action, Component, PostEvent},
    utils,
};

#[derive(Default)]
pub struct Logger {
    logs: Vec<(String, NaiveTime)>,
    opened: bool,
    draw_area: Rect,
}

impl Logger {
    pub fn update(&mut self, log: Vec<(String, NaiveTime)>) {
        self.logs = log;
    }
}

impl Component for Logger {
    fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        if self.opened {
            let style = Style::default().fg(Color::Red);
            let text = self
                .logs
                .iter()
                .map(|(msg, time)| format!("{}: {}", time.format("%H:%M:%S%.3f"), msg))
                .map(|msg| ListItem::new(Span::styled(msg, style)))
                .collect::<Vec<ListItem>>();
            // Add multiline support.
            let list = List::new(text);
            let list = list.block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(app.config.border_type)
                    .title("Logger")
                    .border_style(style),
            );
            let mut list_state = ListState::default();
            if !self.logs.is_empty() {
                list_state.select(Some(self.logs.len() - 1));
            }
            drawer.draw_widget(Clear, self.draw_area);
            drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
        }
    }

    fn key_event(
        &mut self,
        _: &mut crate::app::App,
        key_event: crossterm::event::KeyEvent,
    ) -> PostEvent {
        let key_code = key_event.code;
        if self.opened {
            self.opened = false;
            return PostEvent {
                propegate_further: true,
                action: Action::Noop,
            };
        }
        if key_code == KeyCode::Char('p') {
            self.opened = true;
            return PostEvent {
                propegate_further: true,
                action: Action::Noop,
            };
        }
        return PostEvent {
            propegate_further: false,
            action: Action::Noop,
        };
    }

    fn update_layout(&mut self, draw_area: tui::layout::Rect) {
        self.draw_area = utils::centre_rect(
            tui::layout::Constraint::Percentage(70),
            tui::layout::Constraint::Percentage(70),
            draw_area,
        );
    }
}
