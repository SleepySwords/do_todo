use chrono::NaiveTime;
use crossterm::event::KeyCode;
use tui::layout::Rect;

use crate::{component::message_box::MessageBox, draw::DrawableComponent, utils};

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

impl DrawableComponent for Logger {
    fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        if self.opened {
            drawer.draw_component(
                app,
                &MessageBox::new_by_list(
                    "Log".to_string(),
                    |_| {},
                    self.logs
                        .iter()
                        .map(|(msg, time)| format!("{}: {}", time.format("%H:%M:%S%.3f"), msg))
                        .collect::<Vec<String>>(),
                    tui::style::Color::Red,
                    self.logs.len(),
                ),
            );
        }
    }

    fn key_event(
        &mut self,
        _: &mut crate::app::App,
        key_event: crossterm::event::KeyEvent,
    ) -> crate::draw::EventResult {
        let key_code = key_event.code;
        if self.opened {
            self.opened = false;
            return crate::draw::EventResult::Consumed;
        }
        if key_code == KeyCode::Char('l') {
            self.opened = true;
            return crate::draw::EventResult::Consumed;
        }
        crate::draw::EventResult::Ignored
    }

    fn update_layout(&mut self, draw_area: tui::layout::Rect) {
        self.draw_area = utils::centre_rect(
            tui::layout::Constraint::Percentage(70),
            tui::layout::Constraint::Percentage(70),
            draw_area,
        );
    }
}
