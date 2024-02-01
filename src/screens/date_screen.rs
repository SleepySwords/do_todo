use tui::prelude::{Constraint, Direction, Layout, Rect};

use crate::{app::{App, Mode}, component::overlay::input_box::InputBox, draw::{Component, PostEvent}};

pub struct DateScreen {
    pub date: InputBox,
    pub month: InputBox,
    pub year: InputBox,
}

impl Component for DateScreen {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        self.date.draw(app, drawer);
        self.month.draw(app, drawer);
        self.year.draw(app, drawer);
    }

    fn key_event(&mut self, _app: &mut App, _key_event: crossterm::event::KeyEvent) -> crate::draw::PostEvent {
        return PostEvent::pop_overlay(move |app, _| {
            app.mode = Mode::CurrentTasks;
            return PostEvent::noop(false);
        })
    }

    fn update_layout(&mut self, draw_area: Rect) {
        let draw = Layout::default()
            .constraints(&[
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40),
            ])
            .direction(Direction::Horizontal)
            .split(draw_area);

        self.date.update_layout(draw[0]);
        self.month.update_layout(draw[1]);
        self.year.update_layout(draw[2]);
    }
}
