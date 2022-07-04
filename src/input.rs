use crossterm::event::KeyCode;
use tui::{backend::Backend, layout::Rect, Frame};

use crate::actions::{self, complete_task};
use crate::components::input_box::InputBoxComponent;
use crate::{
    app::{App, PopUpComponents, SelectedComponent},
    task::Task,
};

pub trait Component {
    // Option should pribably be a custom enum
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()>;

    // Could perhaps be a different trait
    fn draw<B: Backend>(&self, app: &App, area: Rect, f: &mut Frame<B>);
}

// Returning an option is pretty lazy, ill refactor this once again at some point.
pub fn handle_input(key_code: KeyCode, app: &mut App) -> Option<()> {
    // This is some janky af shit
    if let Some(component) = app.popup_stack.pop_front() {
        match component {
            PopUpComponents::InputBox(mut component) => {
                if component.handle_event(app, key_code).is_none() {
                    return Some(());
                }
                app.popup_stack
                    .push_front(PopUpComponents::InputBox(component));
            }
            PopUpComponents::DialogBox(mut component) => {
                if component.handle_event(app, key_code).is_none() {
                    return Some(());
                }
                if let KeyCode::Char(char) = key_code {
                    if char == 'q' {
                        return Some(());
                    }
                }
                app.popup_stack
                    .push_front(PopUpComponents::DialogBox(component));
            }
        }
        return Some(());
    }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => {
            app.popup_stack
                .push_front(PopUpComponents::InputBox(InputBoxComponent::new(
                    String::from("Add a task"),
                    Box::new(|app, mut word| {
                        app.task_data.tasks.push(Task::from_string(
                            word.drain(..).collect::<String>().trim().to_string(),
                        ));
                    }),
                )))
        }
        KeyCode::Char('1') => app.selected_window = SelectedComponent::CurrentTasks(0),
        KeyCode::Char('2') => app.selected_window = SelectedComponent::CompletedTasks(0),
        KeyCode::Char('x') => actions::open_help_menu(app),
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
            app.popup_stack
                .push_front(PopUpComponents::InputBox(InputBoxComponent::filled(
                    // TODO: cleanup this so it doesn't use clone
                    format!(
                        "Edit the task {}",
                        app.task_data.tasks[selected_index].title.clone()
                    ),
                    app.task_data.tasks[selected_index].title.clone(),
                    // This move is kinda jank not too sure, may try to find a better way
                    Box::new(move |app, mut word| {
                        app.task_data.tasks[selected_index].title =
                            word.drain(..).collect::<String>().trim().to_string();
                    }),
                )))
        }
        KeyCode::Char('d') => actions::open_delete_task_menu(app, selected_index),
        // todo proper deletion/popup
        // app.action = Action::Delete(selected_index, 0)
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
        KeyCode::Char('c') => complete_task(app, selected_index),
        _ => {}
    }
}

pub fn handle_completed(key_code: KeyCode, selected_index: usize, app: &mut App) {
    // This way until there is a better implementation for other uis/popups
    if let KeyCode::Char('r') = key_code {
        actions::restore_task(app, selected_index)
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
