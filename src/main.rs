mod actions;
mod app;
mod component;
mod config;
mod error;
mod input;
mod screens;
mod task;
mod test;
mod theme;
mod ui;
mod utils;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::error::Error;
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TODO: Should try and recover if it fails
    let (theme, tasks) = config::get_data()?;
    let mut app = App::new(theme, tasks);
    let result = start_app(&mut app, &mut terminal);

    if let Err(err) = result {
        println!("{:?}", err)
    }

    // Shutting down application

    config::save_data(&app.theme, &app.task_store)?;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

pub fn start_app<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> io::Result<()> {
    while !app.should_shutdown() {
        terminal.draw(|f| ui::render_ui(app, f))?;

        // This function blocks
        // TODO: We are probably going to have to implement a Tick system eventually.
        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                app.shutdown();
            }
            input::handle_key(key, app);
        }
    }
    Ok(())
}
