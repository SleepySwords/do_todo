use tui::{
    layout::Rect,
    widgets::{StatefulWidget, Widget},
    Frame,
};

use crate::app::App;

use super::event::{AppEvent, PostEvent};

/// A component that is able to be drawn on the screen.
pub trait Component {
    /// Draws the component onto the [[Drawer]]
    fn draw(&self, app: &App, drawer: &mut Drawer);

    fn key_event(&mut self, _app: &mut App, _key_event: crossterm::event::KeyEvent) -> PostEvent {
        PostEvent::noop(true)
    }

    fn mouse_event(
        &mut self,
        _app: &mut App,
        _mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        PostEvent::noop(true)
    }

    fn update_layout(&mut self, draw_area: Rect);

    fn mount(&mut self, _app: &mut App) {}

    /// This is called before the pop_overlay callback.
    fn unmount(&mut self, _app: &mut App, _event: Option<AppEvent>) -> PostEvent {
        PostEvent::noop(false)
    }
}

// This abstraction is kind off not needed...
pub struct Drawer<'a, 'b> {
    frame: &'a mut Frame<'b>,
}

impl Drawer<'_, '_> {
    pub fn new<'a, 'b>(frame: &'a mut Frame<'b>) -> Drawer<'a, 'b> {
        Drawer { frame }
    }

    // update_layout works nice for now, but might experiment with adding grids.
    pub fn draw_component(&mut self, app: &App, drawable: &dyn Component) {
        drawable.draw(app, self);
    }

    pub fn draw_widget<T: Widget>(&mut self, widget: T, draw_area: Rect) {
        self.frame.render_widget(widget, draw_area);
    }

    pub fn draw_stateful_widget<T: StatefulWidget>(
        &mut self,
        widget: T,
        state: &mut T::State,
        draw_area: Rect,
    ) {
        self.frame.render_stateful_widget(widget, draw_area, state);
    }

    pub fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.frame.set_cursor_position((x, y));
    }
}
