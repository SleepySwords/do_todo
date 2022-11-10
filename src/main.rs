mod actions;
mod app;
mod component;
mod config;
mod error;
mod screens;
mod task;
mod test;
mod theme;
mod utils;
mod view;

use app::App;
use component::layout::{stack_layout::StackLayout, adjacent_layout::AdjacentLayout};
use config::save_data;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use screens::main_screen::MainScreenLayer;
use view::{DrawBackend, DrawableComponent, Drawer, WidgetComponent};

use std::io;
use std::{error::Error, io::Stdout};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (theme, tasks) = config::get_data().expect("Could not get data");
    let mut app = App::new(theme, tasks);
    let result = start_app(&mut app, &mut terminal);

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        println!("{:?}", err);
        return Err(Box::new(err));
    }

    save_data(&mut app)?;

    Ok(())
}

pub fn start_app(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    let mut layout = StackLayout {
        children: vec![Box::new(MainScreenLayer::new())],
    };

    while !app.should_shutdown() {
        terminal.draw(|f| {
            let draw_size = f.size();
            let mut renderbackend = DrawBackend::CrosstermRenderer(f);
            let mut renderer = Drawer::new(draw_size, &mut renderbackend);
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
