use chrono::NaiveTime;
use itertools::Itertools;
use crossterm::event::KeyCode;

use crate::{component::message_box::MessageBox, utils, view::DrawableComponent};

#[derive(Default)]
pub struct Logger {
    logs: Vec<(String, NaiveTime)>,
    opened: bool,
}

impl Logger {
    pub fn update(&mut self, log: Vec<(String, NaiveTime)>) {
        self.logs = log;
    }
}

impl DrawableComponent for Logger {
    fn draw(
        &self,
        app: &crate::app::App,
        draw_area: tui::layout::Rect,
        drawer: &mut crate::view::Drawer,
    ) {
        if self.opened {
            drawer.draw_component(
                app,
                &MessageBox::new(
                    "Log".to_string(),
                    |_| {},
                    self.logs.iter().map(|(msg, time)| format!("{}: {}", time.format("%H:%M:%S%.3f"), msg)).join("\n"),
                    tui::style::Color::Red,
                    self.logs.len()
                ),
                utils::centre_rect(
                    tui::layout::Constraint::Percentage(70),
                    tui::layout::Constraint::Percentage(70),
                    draw_area,
                ),
            );
        }
    }

    fn key_pressed(
        &mut self,
        _: &mut crate::app::App,
        key_code: crossterm::event::KeyCode,
    ) -> crate::view::EventResult {
        if self.opened {
            self.opened = false;
            return crate::view::EventResult::Consumed;
        }
        if key_code == KeyCode::Char('l') {
            self.opened = true;
            return crate::view::EventResult::Consumed;
        }
        crate::view::EventResult::Ignored
    }
}
