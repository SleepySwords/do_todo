use tui::prelude::{Constraint, Direction, Layout, Rect};

use crate::{app::App, component::overlay::input_box::InputBox, draw::Component, utils};

struct DateScreen {
    date: InputBox,
    month: InputBox,
    year: InputBox,
}

impl Component for DateScreen {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        self.date.draw(app, drawer);
        self.month.draw(app, drawer);
        self.year.draw(app, drawer);
    }

    fn update_layout(&mut self, draw_area: Rect) {
        let centre =
            utils::centre_rect(Constraint::Percentage(70), Constraint::Length(3), draw_area);
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
