mod app;
mod task;
mod theme;
mod ui;
mod config;
mod input;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::{fs, io};
use std::error::Error;
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

    let (theme, tasks) = config::get_config()?;
    let mut app = App::new(theme, tasks);
    let result = start_app(&mut app, &mut terminal);

    if let Err(err) = result {
        println!("{:?}", err)
    }

    fs::write(dirs::home_dir().unwrap().join(".config/dtb/data.json"), serde_json::to_string(&app.tasks)?)?;

    // Cleanup
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
    loop {
        terminal.draw(|f| ui::render_ui(app, f))?;

        // This function blocks
        if let Event::Key(key) = event::read()? {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                if key.code == KeyCode::Char('c') {
                    return Ok(());
                }
            }
            if input::handle_input(key.code, app).is_none() {
                return Ok(());
            }
        }
    }
}
