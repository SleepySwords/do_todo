use tui::layout::{Constraint, Direction, Layout, Rect};

use crate::{
    app::App,
    view::{DrawableComponent, Drawer, EventResult},
};

pub enum Element<'a> {
    Owned(Box<dyn DrawableComponent + 'a>),
    Borrow(&'a dyn DrawableComponent),
}

impl Element<'_> {
    pub fn owned<'a, T: DrawableComponent + 'a>(drawable: T) -> Element<'a> {
        Element::Owned(Box::new(drawable))
    }

    pub fn borrow<T: DrawableComponent>(drawable: &T) -> Element {
        Element::Borrow(drawable)
    }
}

pub struct AdjacentLayout<'a> {
    pub children: Vec<(Constraint, Element<'a>)>,
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
                    .map(|f| f.0)
                    .collect::<Vec<Constraint>>(),
            )
            .split(draw_area);

        for (i, drawable) in self.children.iter().enumerate() {
            let drawable = match &drawable.1 {
                Element::Owned(a) => a.as_ref().to_owned(),
                Element::Borrow(a) => a.to_owned(),
            };
            drawer.draw_component(app, drawable, layout_chunk[i]);
        }
    }

    fn key_pressed(&mut self, _: &mut App, _: crossterm::event::KeyCode) -> EventResult {
        EventResult::Ignored
    }
}
