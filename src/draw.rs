use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    widgets::{StatefulWidget, Widget},
    Frame,
};

use std::io::Stdout;

use crate::app::App;

#[derive(PartialEq, Eq, Debug)]
pub enum EventResult {
    Consumed,
    Ignored,
}

/// A component that is able to be drawn on the screen.
pub trait DrawableComponent {
    /// Draws the component onto the [[Drawer]]
    fn draw(&self, app: &App, drawer: &mut Drawer);

    fn key_event(&mut self, _app: &mut App, _key_code: crossterm::event::KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        _app: &mut App,
        _mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        EventResult::Ignored
    }

    fn update_layout(&mut self, draw_area: Rect);
}

// How does this even work, mind blown, wait does it give back ownership when it's done, if so
// that's just really fucking cool.
pub struct Drawer<'a, 'b, 'c> {
    backend: &'a mut DrawFrame<'b, 'c>,
}

impl Drawer<'_, '_, '_> {
    pub fn new<'a, 'b, 'c>(backend: &'a mut DrawFrame<'b, 'c>) -> Drawer<'a, 'b, 'c> {
        Drawer { backend }
    }

    // update_layout works nice for now, but might experiment with adding grids.
    pub fn draw_component(&mut self, app: &App, drawable: &dyn DrawableComponent) {
        drawable.draw(app, self);
    }

    pub fn draw_widget<T: Widget>(&mut self, widget: T, draw_area: Rect) {
        self.backend.draw_widget(widget, draw_area);
    }

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

pub enum DrawFrame<'a, 'b> {
    CrosstermRenderer(&'a mut Frame<'b, CrosstermBackend<Stdout>>),
}

impl DrawFrame<'_, '_> {
    fn draw_widget<T: Widget>(&mut self, widget: T, draw_area: Rect) {
        match self {
            DrawFrame::CrosstermRenderer(f) => {
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
            DrawFrame::CrosstermRenderer(f) => {
                f.render_stateful_widget(widget, draw_area, state);
            }
        }
    }

    fn set_cursor(&mut self, x: u16, y: u16) {
        match self {
            DrawFrame::CrosstermRenderer(f) => {
                f.set_cursor(x, y);
            }
        }
    }
}
