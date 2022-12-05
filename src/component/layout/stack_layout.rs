use tui::layout::Rect;

use crate::{
    app::App,
    view::{DrawableComponent, Drawer, EventResult},
};

pub struct StackLayout {
    pub children: Vec<Box<dyn DrawableComponent>>,
}

impl StackLayout {
    pub fn pop_layer(&mut self) -> Option<Box<dyn DrawableComponent>> {
        self.children.pop()
    }

    pub fn append_layer(&mut self, component: Box<dyn DrawableComponent>) {
        self.children.push(component);
    }
}

impl DrawableComponent for StackLayout {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer) {
        for layout in &self.children {
            drawer.draw_component(app, layout.as_ref(), draw_area);
        }
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        for child in &mut self.children.iter_mut().rev() {
            if child.key_pressed(app, key_code) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }
}
