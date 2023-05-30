use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
    Frame,
};

use crate::app::App;

#[derive(PartialEq, Eq, Debug)]
pub enum EventResult {
    Consumed,
    Ignored,
}

// not call draw themselves. Then we can pass custom variables through the draw call, should remove
// FIX: rewrite such that StackLayout and AdjacentLayout only generates layouts and
// all uses of RefCell code. This would also allow more flexibility. Perhaps DrawableComponent
// should be an enum, however less flexibility, but would allow for nice callback, since pop_layer
// would know which componenent is which.
// ie: viewer.draw(&mut app, layout, task_list.selected, completed_list.selected)

/// A component that is able to be drawn on the screen.
pub trait DrawableComponent {
    /// Draws the component onto the [[Drawer]]
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer);

    fn key_pressed(&mut self, _app: &mut App, _key_code: crossterm::event::KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        _app: &mut App,
        _mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        EventResult::Ignored
    }

    fn update_layout(&mut self, _rect: Rect) {}
}

// How does this even work, mind blown, wait does it give back ownership when it's done, if so
// that's just really fucking cool.
pub struct Drawer<'a, 'b, 'c> {
    backend: &'a mut DrawBackend<'b, 'c>,
}

impl Drawer<'_, '_, '_> {
    pub fn new<'a, 'b, 'c>(backend: &'a mut DrawBackend<'b, 'c>) -> Drawer<'a, 'b, 'c> {
        Drawer { backend }
    }

   pub fn draw_component(&mut self, app: &App, drawable: &dyn DrawableComponent, draw_area: Rect) {
        let mut render = Drawer {
            backend: self.backend,
        };
        drawable.draw(app, draw_area, &mut render);
    }

    pub fn draw_widget<'a, T: Widget>(&mut self, widget: T, draw_area: Rect) {
        self.backend.draw_widget(widget, draw_area);
    }

    // TODO: does not acknowledge the area
    pub fn draw_stateful_widget<T: StatefulWidget>(
        &mut self,
        widget: T,
        state: &mut T::State,
        draw_area: Rect,
    ) {
        self.backend.draw_stateful_widget(widget, state, draw_area);
    }

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.backend.set_cursor(x, y);
    }
}

pub enum DrawBackend<'a, 'b> {
    CrosstermRenderer(&'a mut Frame<'b, CrosstermBackend<Stdout>>),
}

impl DrawBackend<'_, '_> {
    fn draw_widget<T: Widget>(&mut self, widget: T, draw_area: Rect) {
        match self {
            DrawBackend::CrosstermRenderer(f) => {
                f.render_widget(widget, draw_area);
            }
        }
    }

    fn draw_stateful_widget<T: StatefulWidget>(
        &mut self,
        widget: T,
        state: &mut T::State,
        draw_area: Rect,
    ) {
        match self {
            DrawBackend::CrosstermRenderer(f) => {
                f.render_stateful_widget(widget, draw_area, state);
            }
        }
    }

    fn set_cursor(&mut self, x: u16, y: u16) {
        match self {
            DrawBackend::CrosstermRenderer(f) => {
                f.set_cursor(x, y);
            }
        }
    }
}
