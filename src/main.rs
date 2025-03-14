use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

mod actions;
mod app;
mod component;
mod config;
mod data;
mod data_io;
mod error;
mod framework;
mod input;
mod screens;
mod storage;
mod task;
mod tests;
mod utils;

use component::{logger::Logger, message_box::MessageBox, overlay::Overlay};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use error::AppError;
use framework::{
    component::{Component, Drawer},
    event::PostEvent,
    screen_manager::ScreenManager,
};
use futures::{FutureExt, TryStreamExt};
use tokio::time::Instant;
use tracing_subscriber::Registry;
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

use crate::{app::App, screens::main_screen::MainScreen};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    let is_debug = true;

    #[cfg(not(debug_assertions))]
    let is_debug = false;

    let (config, tasks) = data_io::get_data(is_debug).await;

    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(config, tasks);
    let mut screen_manager = ScreenManager {
        app,
        overlays: vec![],
    };

    let result = start_app(&mut screen_manager, &mut terminal).await;

    // Shutting down application

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    let app = screen_manager.app;
    data_io::save_config(&app.config, app.task_store);

    if let Err(err) = result {
        eprintln!("{:?}", err);
        Err(Box::new(err))?;
    }

    Ok(())
}

pub async fn start_app(
    screen_manager: &mut ScreenManager,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<()> {
    let mut main_screen = MainScreen::new();

    let mut logger = Logger::default();

    let subscriber = Registry::default().with(logger.clone());
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    tracing::info!("Todoist Logger is active");

    let mut interval = tokio::time::interval_at(Instant::now(), Duration::from_millis(100));
    let mut event_stream = EventStream::new();

    while !screen_manager.app.should_shutdown() {
        terminal.draw(|f| {
            let draw_size = f.area();

            let mut drawer = Drawer::new(f);

            let chunk = Layout::default()
                .direction(tui::layout::Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(1)])
                .split(draw_size);

            main_screen.update_layout(chunk[0]);
            main_screen.draw(&screen_manager.app, &mut drawer);

            for overlay in screen_manager.overlays.iter_mut() {
                overlay.update_layout(chunk[0])
            }
            for overlay in screen_manager.overlays.iter() {
                overlay.draw(&screen_manager.app, &mut drawer)
            }

            screen_manager.app.status_line.update_layout(chunk[1]);
            screen_manager
                .app
                .status_line
                .draw(&screen_manager.app, &mut drawer);

            logger.update_layout(draw_size);
            logger.draw(&screen_manager.app, &mut drawer);
        })?;

        let tick = interval.tick();
        let crossterm = &mut event_stream.try_next().fuse();

        tokio::select! {
            _ = tick => {
                screen_manager.app.tick += 1;
            }
            Ok(Some(event)) = crossterm => {
                match event {
                    Event::Key(key_event) => {
                        if key_event.code == KeyCode::Char('c')
                            && key_event.modifiers.contains(KeyModifiers::CONTROL)
                        {
                            return Ok(());
                        }
                        if !screen_manager.app.config.debug
                            || logger
                                .key_event(&mut screen_manager.app, key_event)
                                .propegate_further
                        {
                            let result = input::key_event(screen_manager, key_event);
                            match result {
                                Ok(post_event) => screen_manager.handle_post_event(post_event),
                                Err(AppError::InvalidState(msg)) => {
                                    let prev_mode = screen_manager.app.mode;
                                    screen_manager.push_layer(MessageBox::new(
                                        "An error occured".to_string(),
                                        move |app| {
                                            app.mode = prev_mode;
                                            PostEvent::noop(false)
                                        },
                                        msg,
                                        Color::Red,
                                        0,
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }
                    Event::Mouse(mouse_event) => {
                        let post_event = Overlay::mouse_event(screen_manager, mouse_event);
                        let propegate = post_event.propegate_further;
                        screen_manager.handle_post_event(post_event);
                        if propegate {
                            main_screen.mouse_event(&mut screen_manager.app, mouse_event);
                        }
                    }
                    Event::Resize(x, y) => {
                        tracing::debug!("{} {}", x, y);
                    }
                    _ => {
                        println!("oakfe");
                    }
                }
            }
        }
    }
    Ok(())
}
