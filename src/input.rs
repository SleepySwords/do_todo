use chrono::Local;
use crossterm::event::KeyCode;
use tui::{backend::Backend, layout::Rect, Frame};
use tui::widgets::Widget;

use crate::{
    app::{Action, App, SelectedComponent},
    task::{CompletedTask, Task}, components::dialog::DialogComponent,
};
use crate::app::TaskData;

pub trait Component {
    // Option should pribably be a custom enum
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()>;

    // Could perhaps be a different trait
    fn draw<B: Backend>(&self, app: &App, area: Rect, f: &mut Frame<B>);
}

// Returning an option is pretty lazy, ill refactor this once again at some point.
pub fn handle_input(key_code: KeyCode, app: &mut App) -> Option<()> {
    // This is some janky af shit
    if let Some(mut component) = app.dialog_stack.pop_front() {
        if component.handle_event(app, key_code).is_none() {
            return Some(());
        }
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return Some(());
            }
        }
        app.dialog_stack.push_front(component);
        return Some(());
    }

    if let Action::Add = app.action {
        match key_code {
            KeyCode::Char(c) => app.words.push(c),
            KeyCode::Backspace => {
                app.words.pop();
            }
            KeyCode::Enter => {
                app.task_data.tasks.push(Task::from_string(
                    app.words.drain(..).collect::<String>().trim().to_string(),
                ));
                app.action = Action::Normal;
            }
            KeyCode::Esc => app.action = Action::Normal,
            _ => {}
        }
        return Some(());
    }

    if let Action::Edit(task_index) = app.action {
        match key_code {
            KeyCode::Char(c) => app.words.push(c),
            KeyCode::Backspace => {
                app.words.pop();
            }
            KeyCode::Enter => {
                app.task_data.tasks[task_index].title =
                    app.words.drain(..).collect::<String>().trim().to_string();
                app.action = Action::Normal;
            }
            KeyCode::Esc => app.action = Action::Normal,
            _ => {}
        }
        return Some(());
    }

    // if let Action::Delete(task_index, index) = app.action {
    //     match key_code {
    //         KeyCode::Enter => {
    //             if index == 0 {
    //                 app.task_data.tasks.remove(task_index);
    //                 if task_index == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
    //                     app.selected_window = SelectedComponent::CurrentTasks(task_index - 1);
    //                 }
    //                 app.action = Action::Normal;
    //             } else {
    //                 app.action = Action::Normal;
    //             }
    //         }
    //         KeyCode::Char('j') => {
    //             if index == 1 {
    //                 app.action = Action::Delete(task_index, 0);
    //             } else {
    //                 app.action = Action::Delete(task_index, index + 1);
    //             }
    //         }
    //         KeyCode::Char('k') => {
    //             if index == 0 {
    //                 app.action = Action::Delete(task_index, 1);
    //             } else {
    //                 app.action = Action::Delete(task_index, index - 1);
    //             }
    //         }
    //         KeyCode::Esc | KeyCode::Char('q') => app.action = Action::Normal,
    //         _ => {}
    //     }
        // return Some(());
    // }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => app.action = Action::Add,
        KeyCode::Char('1') => app.selected_window = SelectedComponent::CurrentTasks(0),
        KeyCode::Char('2') => app.selected_window = SelectedComponent::CompletedTasks(0),
        KeyCode::Char('q') => return None,
        _ => {}
    }

    handle_movement(key_code, app);

    if let SelectedComponent::CurrentTasks(selected_index) = app.selected_window {
        handle_current_task(key_code, selected_index, app);
    }
    if let SelectedComponent::CompletedTasks(selected_index) = app.selected_window {
        handle_completed(key_code, selected_index, app);
    }
    Some(())
}

pub fn handle_current_task(key_code: KeyCode, selected_index: usize, app: &mut App) {
    match key_code {
        KeyCode::Char('e') => {
            app.words = app.task_data.tasks[selected_index].title.clone();
            app.action = Action::Edit(selected_index);
        }
        KeyCode::Char('d') => {
            if app.task_data.tasks.is_empty() {
                return;
            }
            app.dialog_stack.push_front(DialogComponent::new(format!("Delete task {}", app.task_data.tasks[selected_index].title), vec![
                (String::from("Delete"), Box::new(move |app| {
                    app.task_data.tasks.remove(selected_index);
                    if selected_index == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
                        app.selected_window = SelectedComponent::CurrentTasks(selected_index - 1);
                    }
                })),
                (String::from("Delete"), Box::new(move |app| {
                    app.task_data = TaskData::default();
                    app.selected_window = SelectedComponent::CurrentTasks(0);
                })),
                (String::from("Cancel"), Box::new(|_| {})),
            ]));
            // todo proper deletion/popup
            // app.action = Action::Delete(selected_index, 0)
        }
        KeyCode::Char('h') => {
            if app.task_data.tasks.is_empty() {
                return;
            }
            app.task_data.tasks[selected_index].priority =
                app.task_data.tasks[selected_index].priority.get_next();
        }
        KeyCode::Char('p') => {
            if app.task_data.tasks.is_empty() {
                return;
            }
            app.task_data.tasks[selected_index].progress =
                !app.task_data.tasks[selected_index].progress;
        }
        KeyCode::Char('c') => {
            if app.task_data.tasks.is_empty() {
                return;
            }
            let local = Local::now();
            let time_completed = local.naive_local();
            let task = app.task_data.tasks.remove(selected_index);
            app.task_data
                .completed_tasks
                .push(CompletedTask::from_task(task, time_completed));
            if selected_index == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
                app.selected_window = SelectedComponent::CurrentTasks(selected_index - 1);
            }
        }
        _ => {}
    }
}

pub fn handle_completed(key_code: KeyCode, selected_index: usize, app: &mut App) {
    // This way until there is a better implementation for other uis/popups
    if let KeyCode::Char('r') = key_code {
        if app.task_data.completed_tasks.is_empty() {
            return;
        }
        app.task_data.tasks.push(Task::from_completed_task(
            app.task_data.completed_tasks.remove(selected_index),
        ));
        if selected_index == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
            app.selected_window = SelectedComponent::CompletedTasks(selected_index - 1);
        }
    }
}

fn handle_movement(key_code: KeyCode, app: &mut App) {
    let max_index = match app.selected_window {
        SelectedComponent::CurrentTasks(_) => app.task_data.tasks.len(),
        SelectedComponent::CompletedTasks(_) => app.task_data.completed_tasks.len(),
    };

    let is_empty = match app.selected_window {
        SelectedComponent::CurrentTasks(_) => app.task_data.tasks.is_empty(),
        SelectedComponent::CompletedTasks(_) => app.task_data.completed_tasks.is_empty(),
    };

    let index = app.selected_window.get_selected();
    if index.is_none() {
        return;
    }
    let index = index.unwrap();

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
