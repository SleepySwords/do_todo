mod app;
mod task;
mod theme;
mod ui;

use crate::theme::Theme;
use app::{App, Selection};
use chrono::Local;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::fs;
use std::{error::Error, io, path::Path};
use task::{Task, CompletedTask};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use ui::ui;

fn main() -> Result<(), Box<dyn Error>> {
    let (theme, tasks): (Theme, Vec<Task>) = match dirs::home_dir() {
        Some(home_dir) => {
            let config_path = Path::new(&home_dir).join(".config/dtb/config.yml");
            let data_path = Path::new(&home_dir).join(".config/dtb/data.json");
            let config_contents = fs::read_to_string(&config_path);
            let data_contents = fs::read_to_string(&data_path);
            (
                match config_contents {
                    Ok(file) => {
                        serde_yaml::from_str::<Theme>(&file)?
                    },
                    Err(_) => {
                        let theme = Theme::default();
                        fs::write(&config_path, serde_yaml::to_string(&theme)?)?;
                        theme
                    }
                },
                match data_contents {
                    Ok(file) => {
                        serde_json::from_str::<Vec<Task>>(&file)?
                    },
                    Err(_) => {
                        let tasks: Vec<Task> = vec![];
                        fs::write(&data_path, serde_json::to_string(&tasks)?)?;
                        vec![]
                    }
                }
            )
        }
        None => {
            println!("Not found");
            (Theme::default(), vec![])
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
        terminal.draw(|f| ui(app, f))?;

        // This function blocks
        if let Event::Key(key) = event::read()? {
            if app.add_mode {
                match key.code {
                    KeyCode::Char(c) => app.words.push(c),
                    KeyCode::Backspace => {
                        app.words.pop();
                    }
                    KeyCode::Enter => {
                        app.tasks.push(Task::from_string(
                            app.words.drain(..).collect::<String>().trim().to_string(),
                        ));
                        app.add_mode = !app.add_mode
                    }
                    KeyCode::Esc => app.add_mode = !app.add_mode,
                    _ => {}
                }
                continue;
            }

            // Universal keyboard shortcuts (should also be customisable)
            match key.code {
                KeyCode::Char('a') => {
                    app.add_mode = !app.add_mode;
                }
                KeyCode::Char('1') => app.selected_chunk = Selection::CurrentTasks(0),
                KeyCode::Char('2') => app.selected_chunk = Selection::CompletedTasks(0),
                KeyCode::Char('q') => return Ok(()),
                _ => {}
            }
            if let Selection::CurrentTasks(selected_index) = app.selected_chunk {
                handle_current_task(key.code, selected_index, app);
            }
            if let Selection::CompletedTasks(selected_index) = app.selected_chunk {
                handle_completed(key.code, selected_index, app);
            }
        }
    }
}

fn handle_current_task(key_code: KeyCode, selected_index: usize, app: &mut App) {
    match key_code {
        // J and K should have a `handle_movement` method
        KeyCode::Char('j') => {
            if app.tasks.is_empty() {
                return;
            }
            if selected_index == app.tasks.len() - 1 {
                app.selected_chunk = Selection::CurrentTasks(0);
            } else {
                app.selected_chunk = Selection::CurrentTasks(selected_index + 1);
            }
        }
        KeyCode::Char('k') => {
            if app.tasks.is_empty() {
                return;
            }
            if selected_index == 0 {
                app.selected_chunk = Selection::CurrentTasks(app.tasks.len() - 1);
            } else {
                app.selected_chunk = Selection::CurrentTasks(selected_index - 1);
            }
        }
        KeyCode::Char('d') => {
            if app.tasks.is_empty() {
                return;
            }
            app.tasks.remove(selected_index);
            if selected_index == app.tasks.len() && !app.tasks.is_empty() {
                app.selected_chunk = Selection::CurrentTasks(selected_index - 1);
            }
        }
        KeyCode::Char('h') => {
            if app.tasks.is_empty() {
                return;
            }
            app.tasks[selected_index].priority = app.tasks[selected_index].priority.get_next();
        }
        KeyCode::Char('p') => {
            if app.tasks.is_empty() {
                return;
            }
            app.tasks[selected_index].progress = !app.tasks[selected_index].progress;
        }
        KeyCode::Char('c') => {
            if app.tasks.is_empty() {
                return;
            }
            let local = Local::now();
            let time_completed = local.time();
            let task = app.tasks.remove(selected_index);
            app.completed_tasks.push(CompletedTask::from_task(task, time_completed));
            if selected_index == app.tasks.len() && !app.tasks.is_empty() {
                app.selected_chunk = Selection::CurrentTasks(selected_index - 1);
            }
        }
        _ => {}
    }
}

fn handle_completed(key_code: KeyCode, selected_index: usize, app: &mut App) {
    match key_code {
        KeyCode::Char('j') => {
            if app.completed_tasks.is_empty() {
                return;
            }
            if selected_index == app.completed_tasks.len() - 1 {
                app.selected_chunk = Selection::CompletedTasks(0);
            } else {
                app.selected_chunk = Selection::CompletedTasks(selected_index + 1);
            }
        }
        KeyCode::Char('k') => {
            if app.completed_tasks.is_empty() {
                return;
            }
            if selected_index == 0 {
                app.selected_chunk = Selection::CompletedTasks(app.completed_tasks.len() - 1);
            } else {
                app.selected_chunk = Selection::CompletedTasks(selected_index - 1);
            }
        }
        KeyCode::Char('r') => {
            if app.completed_tasks.is_empty() {
                return;
            }
            app.tasks.push(Task::from_completed_task(app.completed_tasks.remove(selected_index)));
            if selected_index == app.tasks.len() && !app.tasks.is_empty() {
                app.selected_chunk = Selection::CompletedTasks(selected_index - 1);
            }
        }
        _ => {}
    }
}
