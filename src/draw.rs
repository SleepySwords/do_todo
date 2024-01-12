use tui::{
    layout::Rect,
    widgets::{StatefulWidget, Widget},
    Frame,
};

use crate::{app::App, component::overlay::Overlay};

pub enum Action {
    PopOverlay(Box<dyn FnOnce(&mut App, Overlay)>),
    Noop
}

pub struct PostEvent {
    pub propegate_further: bool,
    pub action: Action
}

/// A component that is able to be drawn on the screen.
pub trait Component {
    /// Draws the component onto the [[Drawer]]
    fn draw(&self, app: &App, drawer: &mut Drawer);

    fn key_event(&mut self, _app: &mut App, _key_event: crossterm::event::KeyEvent) -> PostEvent {
        PostEvent {
            propegate_further: false,
            action: Action::Noop
        }
    }

    fn mouse_event(
        &mut self,
        _app: &mut App,
        _mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        PostEvent {
            propegate_further: false,
            action: Action::Noop
        }
    }

    fn update_layout(&mut self, draw_area: Rect);
}

// How does this even work, mind blown, wait does it give back ownership when it's done, if so
// that's just really fucking cool.
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

    pub fn set_cursor(&mut self, x: u16, y: u16) {
        self.frame.set_cursor(x, y);
    }
}
