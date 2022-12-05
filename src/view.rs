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

// FIX: rewrite such that StackLayout and AdjacentLayout only generates layouts and
// not call draw themselves. Then we can pass custom variables through the draw call, should remove
// all uses of RefCell code. This would also allow more flexibility. Perhaps DrawableComponent
// should be an enum, however less flexibility, but would allow for nice callback, since pop_layer
// would know which componenent is which.
// ie: viewer.draw(&mut app, layout, task_list.selected, completed_list.selected)

/// A component that is able to be drawn on the screen.
pub trait DrawableComponent {
    /// Draws the component onto the [[Drawer]]
    /// Takes a draw_area which has the origin (0, 0) and the allowed width and height of the
    /// component.
    // On a side note, we should probably not have the draw_area within the Drawer,
    // it's quite weird tbh, and just to prevent writing out of bands of the frame.
    // Something to consider in the next refactor
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer);

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult;
}

// How does this even work, mind blown, wait does it give back ownership when it's done, if so
// that's just really fucking cool.
pub struct Drawer<'a, 'b, 'c> {
    draw_area: Rect,
    backend: &'a mut DrawBackend<'b, 'c>,
}

impl Drawer<'_, '_, '_> {
    pub fn new<'a, 'b, 'c>(
        draw_area: Rect,
        backend: &'a mut DrawBackend<'b, 'c>,
    ) -> Drawer<'a, 'b, 'c> {
        Drawer { draw_area, backend }
    }

    pub fn draw_component(&mut self, app: &App, drawable: &dyn DrawableComponent, draw_area: Rect) {
        let render_draw_area = Rect {
            x: self.draw_area.x + draw_area.x,
            y: self.draw_area.y + draw_area.y,
            width: draw_area.width,
            height: draw_area.height,
        };
        let mut render = Drawer {
            draw_area: render_draw_area,
            backend: self.backend,
        };
        let draw_area = Rect {
            x: 0,
            y: 0,
            width: draw_area.width,
            height: draw_area.height,
        };
        drawable.draw(app, draw_area, &mut render);
    }

    pub fn draw_widget<T: Widget>(&mut self, widget: T, mut draw_area: Rect) {
        draw_area.x += self.draw_area.x;
        draw_area.y += self.draw_area.y;
        self.backend.draw_widget(widget, draw_area);
    }

    // TODO: does not acknowledge the area
    pub fn draw_stateful_widget<T: StatefulWidget>(
        &mut self,
        widget: T,
        state: &mut T::State,
        mut draw_area: Rect,
    ) {
        draw_area.x += self.draw_area.x;
        draw_area.y += self.draw_area.y;
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
