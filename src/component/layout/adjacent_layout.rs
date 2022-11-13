use tui::layout::{Constraint, Direction, Layout, Rect};

use crate::{
    app::App,
    view::{DrawableComponent, Drawer, EventResult},
};

pub enum Child<'a> {
    Owned(Constraint, Box<dyn DrawableComponent + 'a>),
    Borrow(Constraint, &'a dyn DrawableComponent),
}

impl Child<'_> {
    pub fn owned<'a, T: DrawableComponent + 'a>(contraint: Constraint, drawable: T) -> Child<'a> {
        Child::Owned(contraint, Box::new(drawable))
    }

    pub fn borrow<T: DrawableComponent>(constraint: Constraint, drawable: &T) -> Child {
        Child::Borrow(constraint, drawable)
    }

    fn constraint(&self) -> Constraint {
        match self {
            Child::Owned(constraint, _) => *constraint,
            Child::Borrow(constraint, _) => *constraint,
        }
    }
}

pub struct AdjacentLayout<'a> {
    pub children: Vec<Child<'a>>,
    pub direction: Direction,
}

impl Default for AdjacentLayout<'_> {
    fn default() -> Self {
        AdjacentLayout {
            children: Vec::new(),
            direction: Direction::Vertical,
        }
    }
}

impl DrawableComponent for AdjacentLayout<'_> {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer) {
        let layout_chunk = Layout::default()
            .direction(self.direction.clone())
            .constraints(
                self.children
                    .iter()
                    .map(|f| f.constraint())
                    .collect::<Vec<Constraint>>(),
            )
            .split(draw_area);

        for (i, drawable) in self.children.iter().enumerate() {
            let drawable = match &drawable {
                Child::Owned(_, a) => a.as_ref().to_owned(),
                Child::Borrow(_, a) => a.to_owned(),
            };
            drawer.draw_component(app, drawable, layout_chunk[i]);
        }
    }

    fn key_pressed(&mut self, _: &mut App, _: crossterm::event::KeyCode) -> EventResult {
        EventResult::Ignored
    }
}
