mod actions;
mod app;
mod component;
mod config;
mod data_io;
mod draw;
mod error;
mod input;
mod key;
mod logger;
mod screens;
mod task;
mod tests;
mod utils;

use app::MainApp;
use component::{message_box::MessageBox, overlay::Overlay};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use draw::PostEvent;
use error::AppError;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::Color,
    Terminal,
};

use std::{
    error::Error,
    io::{self, Stdout},
    time::Duration,
};

use crate::{
    app::App,
    draw::{Component, Drawer},
    logger::Logger,
    screens::main_screen::MainScreen,
};

fn main() -> Result<(), Box<dyn Error>> {
    let (theme, tasks) = data_io::get_data();

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(theme, tasks);
    let mut main_app = MainApp {
        app,
        overlays: vec![],
    };

    let result = start_app(&mut main_app, &mut terminal);

    // Shutting down application

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let app = main_app.app;
    data_io::save_data(&app.config, &app.task_store);

    if let Err(err) = result {
        eprintln!("{:?}", err);
        return Err(Box::new(err));
    }

    Ok(())
}

pub fn start_app(
    main_app: &mut MainApp,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    let mut main_screen = MainScreen::new();

    let mut logger = Logger::default();

    while !main_app.app.should_shutdown() {
        terminal.draw(|f| {
            let draw_size = f.size();

            let mut drawer = Drawer::new(f);

            let chunk = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(draw_size);

            main_screen.update_layout(chunk[0]);
            main_screen.draw(&main_app.app, &mut drawer);

            for overlay in main_app.overlays.iter_mut() {
                overlay.update_layout(chunk[0])
            }
            for overlay in main_app.overlays.iter() {
                overlay.draw(main_app, &mut drawer)
            }

            main_app.app.status_line.update_layout(chunk[1]);
            main_app.app.status_line.draw(&main_app.app, &mut drawer);

            logger.update_layout(draw_size);
            logger.draw(&main_app.app, &mut drawer);
        })?;

        // This function blocks
        // TODO: We are probably going to have to implement a Tick system eventually, using mspc
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.code == KeyCode::Char('c')
                        && key_event.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        return Ok(());
                    }
                    if !main_app.app.config.debug
                        || logger
                            .key_event(&mut main_app.app, key_event)
                            .propegate_further
                    {
                        let result = input::key_event(main_app, key_event);
                        match result {
                            Ok(post_event) => main_app.handle_post_event(post_event),
                            Err(AppError::InvalidState(msg)) => {
                                let prev_mode = main_app.app.mode;
                                main_app.push_layer(Overlay::Message(MessageBox::new(
                                    "An error occured".to_string(),
                                    move |app| {
                                        app.mode = prev_mode;
                                        PostEvent::noop(false)
                                    },
                                    msg,
                                    Color::Red,
                                    0,
                                )));
                            }
                            _ => {}
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    let post_event = Overlay::mouse_event(main_app, mouse_event);
                    let propegate = post_event.propegate_further;
                    main_app.handle_post_event(post_event);
                    if propegate {
                        main_screen.mouse_event(&mut main_app.app, mouse_event);
                    }
                }
                Event::Resize(x, y) => {
                    main_app.app.println(format!("{} {}", x, y));
                }
                _ => {}
            }
        }
    }
    Ok(())
}
