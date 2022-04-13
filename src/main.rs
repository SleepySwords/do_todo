pub mod app;
pub mod task;

use app::App;
use std::{error::Error, io};

use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use tui::{Terminal, backend::CrosstermBackend};

fn main() -> Result<(), Box<dyn Error>>{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let result = app.run_app(&mut terminal);

    if let Err(err) = result {
        println!("{:?}", err)
    }

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

