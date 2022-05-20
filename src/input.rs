use chrono::Local;
use crossterm::event::KeyCode;

use crate::{
    app::{App, Mode, Windows},
    task::{CompletedTask, Task},
};

// Returning an option is pretty lazy, ill refactor this once again at some point.
pub fn handle_input(key_code: KeyCode, app: &mut App) -> Option<()> {
    if let Mode::Add = app.mode {
        match key_code {
            KeyCode::Char(c) => app.words.push(c),
            KeyCode::Backspace => {
                app.words.pop();
            }
            KeyCode::Enter => {
                app.tasks.push(Task::from_string(
                    app.words.drain(..).collect::<String>().trim().to_string(),
                ));
                app.mode = Mode::Normal;
            }
            KeyCode::Esc => app.mode = Mode::Normal,
            _ => {}
        }
        return Some(());
    }

    if let Mode::Edit(task_index) = app.mode {
        match key_code {
            KeyCode::Char(c) => app.words.push(c),
            KeyCode::Backspace => {
                app.words.pop();
            }
            KeyCode::Enter => {
                app.tasks[task_index].title =
                    app.words.drain(..).collect::<String>().trim().to_string();
                app.mode = Mode::Normal;
            }
            KeyCode::Esc => app.mode = Mode::Normal,
            _ => {}
        }
        return Some(());
    }

    if let Mode::Delete(task_index, index) = app.mode {
        match key_code {
            KeyCode::Enter => {
                if index == 0 {
                    app.tasks.remove(task_index);
                    if task_index == app.tasks.len() && !app.tasks.is_empty() {
                        app.selected_window = Windows::CurrentTasks(task_index - 1);
                    }
                    app.mode = Mode::Normal;
                } else {
                    app.mode = Mode::Normal;
                }
            }
            KeyCode::Char('j') => {
                if index == 1 {
                    app.mode = Mode::Delete(task_index, 0);
                } else {
                    app.mode = Mode::Delete(task_index, index + 1);
                }
            }
            KeyCode::Char('k') => {
                if index == 0 {
                    app.mode = Mode::Delete(task_index, 1);
                } else {
                    app.mode = Mode::Delete(task_index, index - 1);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Normal,
            _ => {}
        }
        return Some(());
    }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => app.mode = Mode::Add,
        KeyCode::Char('1') => app.selected_window = Windows::CurrentTasks(0),
        KeyCode::Char('2') => app.selected_window = Windows::CompletedTasks(0),
        KeyCode::Char('q') => return None,
        _ => {}
    }

    handle_movement(key_code, app);

    if let Windows::CurrentTasks(selected_index) = app.selected_window {
        handle_current_task(key_code, selected_index, app);
    }
    if let Windows::CompletedTasks(selected_index) = app.selected_window {
        handle_completed(key_code, selected_index, app);
    }
    Some(())
}

pub fn handle_current_task(key_code: KeyCode, selected_index: usize, app: &mut App) {
    match key_code {
        KeyCode::Char('e') => {
            app.mode = Mode::Edit(selected_index);
        }
        KeyCode::Char('d') => {
            if app.tasks.is_empty() {
                return;
            }
            app.mode = Mode::Delete(selected_index, 0)
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
            app.completed_tasks
                .push(CompletedTask::from_task(task, time_completed));
            if selected_index == app.tasks.len() && !app.tasks.is_empty() {
                app.selected_window = Windows::CurrentTasks(selected_index - 1);
            }
        }
        _ => {}
    }
}

pub fn handle_completed(key_code: KeyCode, selected_index: usize, app: &mut App) {
    match key_code {
        KeyCode::Char('r') => {
            if app.completed_tasks.is_empty() {
                return;
            }
            app.tasks.push(Task::from_completed_task(
                app.completed_tasks.remove(selected_index),
            ));
            if selected_index == app.tasks.len() && !app.tasks.is_empty() {
                app.selected_window = Windows::CompletedTasks(selected_index - 1);
            }
        }
        _ => {}
    }
}

fn handle_movement(key_code: KeyCode, app: &mut App) {
    let max_index = match app.selected_window {
        Windows::CurrentTasks(_) => app.tasks.len(),
        Windows::CompletedTasks(_) => app.completed_tasks.len(),
    };

    let is_empty = match app.selected_window {
        Windows::CurrentTasks(_) => app.tasks.is_empty(),
        Windows::CompletedTasks(_) => app.completed_tasks.is_empty(),
    };

    let index = app.selected_window.get_selected();
    if index.is_none() {
        return;
    }
    let /* ref */ index = index.unwrap();

    match key_code {
        KeyCode::Char('j') => {
            if is_empty {
                return;
            }
            if *index == max_index - 1 {
                *index = 0;
            } else {
                *index += 1;
            }
        }
        KeyCode::Char('k') => {
            if is_empty {
                return;
            }
            if *index == 0 {
                *index = max_index - 1;
            } else {
                *index -= 1;
            }
        }
        _ => {}
    }
}
