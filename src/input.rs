use chrono::Local;
use crossterm::event::KeyCode;

use crate::{app::{App, Selection}, task::{CompletedTask, Task}};

// Returning an option is pretty lazy, ill refactor this once again at some point.
pub fn handle_input(key_code: KeyCode, app: &mut App) -> Option<()>{
    if app.add_mode {
        match key_code {
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
        return Some(())
    }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => {
            app.add_mode = !app.add_mode;
        }
        KeyCode::Char('1') => app.selected_chunk = Selection::CurrentTasks(0),
        KeyCode::Char('2') => app.selected_chunk = Selection::CompletedTasks(0),
        KeyCode::Char('q') => return None,
        _ => {}
    }
    if let Selection::CurrentTasks(selected_index) = app.selected_chunk {
        handle_current_task(key_code, selected_index, app);
    }
    if let Selection::CompletedTasks(selected_index) = app.selected_chunk {
        handle_completed(key_code, selected_index, app);
    }
    Some(())
}

pub fn handle_current_task(key_code: KeyCode, selected_index: usize, app: &mut App) {
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

pub fn handle_completed(key_code: KeyCode, selected_index: usize, app: &mut App) {
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