mod actions;
mod app;
mod component;
mod config;
mod error;
mod logger;
mod screens;
mod task;
mod test;
mod theme;
mod utils;
mod view;

use app::App;
use component::layout::{
        adjacent_layout::{AdjacentLayout, Child},
        stack_layout::StackLayout,
    };
use config::save_data;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use logger::Logger;
use screens::main_screen::MainScreenLayer;
use view::{DrawBackend, DrawableComponent, Drawer, EventResult};

use std::io;
use std::{error::Error, io::Stdout};
use tui::{backend::CrosstermBackend, Terminal};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // TODO: Should try and recover if it fails
    let (theme, tasks) = config::get_data().expect("Could not get data");
    let mut app = App::new(theme, tasks);
    let result = start_app(&mut app, &mut terminal);

    // Shutting down application

    config::save_data(&app.theme, &app.task_store)?;

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

    save_data(&app.theme, &app.task_store)?;

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
            app.app_size = draw_size;

            let mut draw_backend = DrawBackend::CrosstermRenderer(f);
            let mut drawer = Drawer::new(draw_size, &mut draw_backend);
            let layout = AdjacentLayout {
                children: vec![
                    Child::Borrow(tui::layout::Constraint::Min(1), &stack_layout),
                    Child::Borrow(tui::layout::Constraint::Length(1), &app.status_line),
                ],
                direction: tui::layout::Direction::Vertical,
            };
            layout.draw(app, draw_size, &mut drawer);
            logger.draw(app, draw_size, &mut drawer);
        })?;

        // This function blocks
        // TODO: We are probably going to have to implement a Tick system eventually, using mspc
        match event::read()? {
            Event::Key(event) => {
                if event.code == KeyCode::Char('c')
                    && event.modifiers.contains(KeyModifiers::CONTROL)
                {
                    return Ok(());
                }
                if logger.key_pressed(app, event.code) == EventResult::Ignored {
                    stack_layout.key_pressed(app, event.code);
                }

                while let Some(callback) = app.callbacks.pop_front() {
                    callback(app, &mut stack_layout);
                }
            }
            Event::Mouse(event) => {
                // stack_layout.mouse_event(app, event);
            }
            Event::Resize(x, y) => {
                app.println(format!("{} {}", x, y));
            }
        }
        logger.update(app.logs.clone());
    }
    Ok(())
}
