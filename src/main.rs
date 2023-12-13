mod actions;
mod app;
mod component;
mod config;
mod draw;
mod error;
pub mod key;
mod logger;
mod screens;
mod task;
mod tests;
mod theme;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    Terminal,
};

use std::{
    error::Error,
    io::{self, Stdout},
    time::Duration,
};

use crate::{
    app::App,
    component::layout::stack_layout::StackLayout,
    draw::{DrawableComponent, Drawer, EventResult},
    logger::Logger,
    screens::main_screen::MainScreenLayer,
};

fn main() -> Result<(), Box<dyn Error>> {
    let (theme, tasks) = config::get_data();

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(theme, tasks);

    let result = start_app(&mut app, &mut terminal);

    // Shutting down application

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    config::save_data(&app.theme, &app.task_store);

    if let Err(err) = result {
        eprintln!("{:?}", err);
        return Err(Box::new(err));
    }

    Ok(())
}

pub fn start_app(
    app: &mut App,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    let mut stack_layout = StackLayout {
        children: vec![Box::new(MainScreenLayer::new())],
    };

    let mut logger = Logger::default();

    while !app.should_shutdown() {
        terminal.draw(|f| {
            let draw_size = f.size();

            let mut drawer = Drawer::new(f);

            let chunk = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(draw_size);

            stack_layout.update_layout(chunk[0]);
            stack_layout.draw(app, &mut drawer);

            app.status_line.update_layout(chunk[1]);
            app.status_line.draw(app, &mut drawer);

            logger.update_layout(draw_size);
            logger.draw(app, &mut drawer);
        })?;

        // This function blocks
        // TODO: We are probably going to have to implement a Tick system eventually, using mspc
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(event) => {
                    if event.code == KeyCode::Char('c')
                        && event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }
                    if EventResult::Ignored == stack_layout.key_event(app, event) {
                        logger.key_event(app, event);
                    }
                }
                Event::Mouse(event) => {
                    stack_layout.mouse_event(app, event);
                }
                Event::Resize(x, y) => {
                    app.println(format!("{} {}", x, y));
                }
                _ => {}
            }
            while let Some(callback) = app.callbacks.pop_front() {
                callback(app, &mut stack_layout);
            }
            logger.update(app.logs.clone());
        }
    }
    Ok(())
}
