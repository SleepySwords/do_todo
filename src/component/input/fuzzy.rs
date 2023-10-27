use crossterm::event::KeyCode;
use tui::layout::{Constraint, Layout, Rect};

use crate::{draw::DrawableComponent, utils};

use super::{
    dialog::{DialogAction, DialogBox},
    input_box::InputBox,
};

pub struct FuzzyBox {
    pub draw_area: Rect,
    pub input: InputBox,
    pub dialog: DialogBox,
    pub inactive: Vec<DialogAction>,
}

impl FuzzyBox {
    fn generate_rect(&self, rect: Rect) -> Rect {
        utils::centre_rect(Constraint::Percentage(70), Constraint::Percentage(80), rect)
    }
}

impl DrawableComponent for FuzzyBox {
    fn draw(&self, app: &crate::app::App, drawer: &mut crate::draw::Drawer) {
        self.input.draw(app, drawer);
        self.dialog.draw(app, drawer);
    }

    fn update_layout(&mut self, draw_area: Rect) {
        self.draw_area = self.generate_rect(draw_area);
        let layout = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Percentage(80)])
            .split(self.draw_area);

        self.input.update_layout(layout[0]);
        self.dialog.update_layout(layout[1]);
    }

    fn key_event(
        &mut self,
        app: &mut crate::app::App,
        key_event: crossterm::event::KeyEvent,
    ) -> crate::draw::EventResult {
        let code = key_event.code;
        match code {
            KeyCode::Enter => self.dialog.key_event(app, key_event),
            _ => {
                let e = self.input.key_event(app, key_event);
                let input = self.input.text().to_ascii_lowercase();
                // FIXME: this mucks up the order, and is a pretty bad way to do things
                for i in (0..self.dialog.options.len()).rev() {
                    if !self.dialog.options[i]
                        .name
                        .to_ascii_lowercase()
                        .contains(&input)
                    {
                        self.inactive.push(self.dialog.options.remove(i));
                    }
                }
                for i in (0..self.inactive.len()).rev() {
                    if self.inactive[i].name.to_ascii_lowercase().contains(&input) {
                        self.dialog.options.push(self.inactive.remove(i));
                    }
                }
                e
            }
        }
    }
}

struct FuzzyBuilder {}
