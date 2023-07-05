use tui::layout::Rect;

use crate::{
    app::App,
    draw::{DrawableComponent, Drawer, EventResult},
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
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        for layout in &self.children {
            drawer.draw_component(app, layout.as_ref());
        }
    }

    fn key_pressed(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        for child in &mut self.children.iter_mut().rev() {
            if child.key_pressed(app, key_event) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        for child in &mut self.children.iter_mut().rev() {
            if child.mouse_event(app, mouse_event) == EventResult::Consumed {
                return EventResult::Consumed;
            }
        }
        EventResult::Ignored
    }

    fn update_layout(&mut self, rect: Rect) {
        for child in self.children.iter_mut() {
            child.update_layout(rect);
        }
    }
}
