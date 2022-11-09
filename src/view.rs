use std::{
    io::{self, Stdout},
    vec,
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, StatefulWidget, Widget},
    Frame, Terminal,
};

use crate::{app::App, screens::main_screen::MainScreenLayer};

#[derive(PartialEq, Eq, Debug)]
pub enum EventResult {
    Consumed,
    Ignored,
}

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

pub struct StackLayout {
    pub children: Vec<Box<dyn DrawableComponent>>,
}

impl StackLayout {
    pub fn pop_layer(&mut self) {
        self.children.pop();
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
        Drawer {
            draw_area,
            backend,
        }
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

    /// Todo does not acknowledge the area
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

/// Stores all the different types of Renderers in an enum
/// This is to because generics are not allowed in traits as they are not "Object-safe" like wtf
/// let me have my generics
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

pub struct WidgetComponent {
    rect: Rect,
    colour: Color,
}

impl DrawableComponent for WidgetComponent {
    fn draw(&self, _: &App, _: Rect, renderer: &mut Drawer) {
        let widget = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colour));
        renderer.draw_widget(widget, self.rect);
    }

    fn key_pressed(&mut self, _: &mut App, _: crossterm::event::KeyCode) -> EventResult {
        EventResult::Ignored
    }
}

impl WidgetComponent {
    pub fn new(rect: Rect) -> WidgetComponent {
        WidgetComponent {
            rect,
            colour: Color::White,
        }
    }
    fn new_colour(rect: Rect, colour: Color) -> WidgetComponent {
        WidgetComponent { rect, colour }
    }
}

struct BiLayout {
    first: Box<dyn DrawableComponent>,
    second: Box<dyn DrawableComponent>,
}

impl DrawableComponent for BiLayout {
    fn draw(&self, app: &App, draw_area: Rect, renderer: &mut Drawer) {
        let layout_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(draw_area);

        renderer.draw_component(app, self.first.as_ref(), layout_chunk[0]);
        renderer.draw_component(app, self.second.as_ref(), layout_chunk[1]);
    }

    fn key_pressed(&mut self, _: &mut App, _: crossterm::event::KeyCode) -> EventResult {
        EventResult::Ignored
    }
}

/// Maybe it would be better to do this in the traditional style, ie: calling draw in each
/// individual component so there is no need for the DrawableComponent trait, something to consider
/// i guess.
pub fn test_render(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    let mut layout = StackLayout {
        children: vec![
            Box::new(MainScreenLayer::new()),
            Box::new(WidgetComponent::new(Rect::new(0, 0, 50, 50))),
            Box::new(BiLayout {
                first: Box::new(WidgetComponent::new(Rect::new(10, 10, 10, 10))),
                second: Box::new(WidgetComponent::new(Rect::new(0, 0, 10, 10))),
            }),
            Box::new(WidgetComponent::new_colour(
                Rect::new(10, 10, 10, 10),
                Color::Red,
            )),
        ],
    };

    while !app.should_shutdown() {
        terminal.draw(|f| {
            let draw_size = f.size();
            let mut renderbackend = DrawBackend::CrosstermRenderer(f);
            let mut renderer = Drawer {
                draw_area: draw_size,
                backend: &mut renderbackend,
            };
            layout.draw(app, draw_size, &mut renderer);
        })?;

        if let Event::Key(event) = event::read()? {
            if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(());
            }
            layout.key_pressed(app, event.code);

            while let Some(callback) = app.callbacks.pop_front() {
                callback(app, &mut layout);
            }
        }
    }
    Ok(())
}

// FIX: Rc + Refcell give overhead
// FIX: Maybe just chuck everything in the App class, which also solves the bottom problem to an
// extent.
// FIX: Everything inside a box -> heap allocated -> More overhead :(
