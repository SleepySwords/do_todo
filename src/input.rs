use crossterm::event::KeyEvent;

use crate::{
    actions,
    app::{App, Mode},
    component::{
        completed_list::CompletedList,
        overlay::{input_box::InputBoxBuilder, Overlay},
    },
    config::KeyBindings,
    draw::EventResult,
    error::AppError,
    task::{Task, TaskStore},
    utils,
};

pub fn key_event(app: &mut App, key_event: KeyEvent) -> Result<EventResult, AppError> {
    let event = match app.mode {
        Mode::Overlay => Overlay::key_event(app, key_event),
        Mode::CurrentTasks => task_list_input(app, key_event),
        Mode::CompletedTasks => completed_list_input(app, key_event),
    };
    if let Ok(EventResult::Ignored) = event {
        Ok(universal_input(app, key_event))
    } else {
        event
    }
}

fn task_list_input(app: &mut App, key_event: KeyEvent) -> Result<EventResult, AppError> {
    let theme = &app.config;

    let selected_index = &mut app.task_list.selected_index;

    // Move this to the actions class
    match KeyBindings::from_event(theme, key_event) {
        KeyBindings::ChangePriorityKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(EventResult::Ignored);
            }

            let old_task = {
                let Some(task) = app.task_store.task_mut(*selected_index) else {
                    return Ok(EventResult::Ignored);
                };
                task.priority = task.priority.next_priority();

                task.clone()
            };

            if app.task_store.auto_sort {
                app.task_store.sort();
            }

            *selected_index = app.task_store.task_position(&old_task).ok_or_else(|| {
                AppError::InvalidState("Cannot find the selected tasks index.".to_string())
            })?
        }
        KeyBindings::MoveTaskDown => {
            let autosort = app.task_store.auto_sort;

            let Some((parent_tasks, parent_index, local_index, is_global)) =
                app.task_store.find_parent(*selected_index)
            else {
                return Ok(EventResult::Ignored);
            };

            let new_index = (local_index + 1) % parent_tasks.len();

            let Some(parent_subtasks) = app.task_store.subtasks(parent_index, is_global) else {
                return Ok(EventResult::Ignored);
            };

            let task = &parent_subtasks[local_index];
            let task_above = &parent_subtasks[new_index];

            if task.priority == task_above.priority || !autosort {
                let task = parent_subtasks.remove(local_index);

                parent_subtasks.insert(new_index, task);

                *selected_index = TaskStore::local_index_to_global(
                    new_index,
                    parent_subtasks,
                    parent_index,
                    is_global,
                );
            }
        }
        KeyBindings::MoveTaskUp => {
            let autosort = app.task_store.auto_sort;

            let Some((parent_subtasks, parent_index, local_index, is_global)) =
                app.task_store.find_parent(*selected_index)
            else {
                return Ok(EventResult::Ignored);
            };

            if parent_subtasks.is_empty() {
                return Ok(EventResult::Ignored);
            }

            let new_index =
                (local_index as isize - 1).rem_euclid(parent_subtasks.len() as isize) as usize;

            let Some(mut_parent_subtasks) = app.task_store.subtasks(parent_index, is_global) else {
                return Ok(EventResult::Ignored);
            };

            let task = &mut_parent_subtasks[local_index];
            let task_above = &mut_parent_subtasks[new_index];

            if task.priority == task_above.priority || !autosort {
                let task = mut_parent_subtasks.remove(local_index);

                mut_parent_subtasks.insert(new_index, task);

                *selected_index = TaskStore::local_index_to_global(
                    new_index,
                    mut_parent_subtasks,
                    parent_index,
                    is_global,
                )
            }
        }
        KeyBindings::DeleteKey => actions::open_delete_task_menu(app),
        KeyBindings::EditKey => {
            let index = *selected_index;
            let Some(task) = app.task_store.task(index) else {
                return Ok(EventResult::Ignored);
            };
            let edit_box = InputBoxBuilder::default()
                .title(String::from("Edit the selected task"))
                .fill(task.title.as_str())
                .callback(move |app, word| {
                    let Some(task) = app.task_store.task_mut(index) else {
                        return Ok(());
                    };
                    task.title = word.trim().to_string();
                    Ok(())
                })
                .save_mode(app)
                .build();
            app.push_layer(edit_box)
        }
        KeyBindings::TagMenu => actions::open_tag_menu(app),
        KeyBindings::FlipProgressKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(EventResult::Ignored);
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return Ok(EventResult::Ignored);
            };
            task.progress = !task.progress;
        }
        KeyBindings::CompleteKey => actions::complete_task(app),
        KeyBindings::OpenSubtasksKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(EventResult::Ignored);
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return Ok(EventResult::Ignored);
            };
            task.opened = !task.opened;
        }
        KeyBindings::MoveSubtaskLevelUp => {
            let Some((parent_tasks, parent_index, local_index, is_task_global)) =
                app.task_store.find_parent(*selected_index)
            else {
                return Ok(EventResult::Ignored);
            };

            if parent_tasks.is_empty() {
                return Ok(EventResult::Ignored);
            }

            if local_index == 0 {
                return Ok(EventResult::Ignored);
            }

            let prev_local_index = local_index - 1;
            let prev_global_index = TaskStore::local_index_to_global(
                prev_local_index,
                parent_tasks,
                parent_index,
                is_task_global,
            );

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return Ok(EventResult::Ignored);
            };

            let Some(prev_task) = app.task_store.task_mut(prev_global_index) else {
                return Ok(EventResult::Ignored);
            };

            prev_task.opened = true;
            prev_task.sub_tasks.push(task);

            // FIXME: refactor this to ideally not clone
            if app.task_store.auto_sort {
                // FIXME: should be an error.
                let Some(task) = app.task_store.task(*selected_index).cloned() else {
                    return Ok(EventResult::Ignored);
                };
                app.task_store.sort();
                if let Some(task_pos) = app.task_store.task_position(&task) {
                    *selected_index = task_pos;
                }
            }
        }
        KeyBindings::MoveSubtaskLevelDown => {
            let Some((_, parent_index, _, is_task_global)) =
                app.task_store.find_parent(*selected_index)
            else {
                return Ok(EventResult::Ignored);
            };

            if is_task_global {
                return Ok(EventResult::Ignored);
            }

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return Ok(EventResult::Ignored);
            };

            let Some((grandparent_subtasks, grandparent_index, _, is_parent_global)) =
                app.task_store.find_parent(parent_index)
            else {
                return Ok(EventResult::Ignored);
            };

            let parent_local_index = grandparent_subtasks
                .iter()
                .position(|t| {
                    t == app
                        .task_store
                        .task(parent_index)
                        .expect("This is definately a task")
                })
                .ok_or_else(|| {
                    AppError::InvalidState("Cannot find the parent tasks index.".to_string())
                })?;

            let Some(grandparent_subtasks) =
                app.task_store.subtasks(grandparent_index, is_parent_global)
            else {
                return Ok(EventResult::Ignored);
            };

            let new_index = parent_local_index + 1;
            grandparent_subtasks.insert(new_index, task);
            *selected_index = TaskStore::local_index_to_global(
                new_index,
                grandparent_subtasks,
                grandparent_index,
                is_parent_global,
            );
            // FIXME: refactor this to ideally not clone
            if app.task_store.auto_sort {
                // FIXME: should be an error.
                let Some(task) = app.task_store.task(*selected_index).cloned() else {
                    return Ok(EventResult::Ignored);
                };
                app.task_store.sort();
                if let Some(task_pos) = app.task_store.task_position(&task) {
                    *selected_index = task_pos;
                }
            }
        }
        _ => {
            return Ok(utils::handle_key_movement(
                theme,
                key_event,
                selected_index,
                app.task_store.find_tasks_draw_size(),
            ));
        }
    }
    Ok(EventResult::Consumed)
}

fn completed_list_input(app: &mut App, key_event: KeyEvent) -> Result<EventResult, AppError> {
    let result = utils::handle_key_movement(
        &app.config,
        key_event,
        &mut app.completed_list.selected_index,
        app.task_store.completed_tasks.len(),
    );

    if result == EventResult::Consumed {
        return Ok(EventResult::Consumed);
    }

    if app.config.restore_key.is_pressed(key_event) {
        CompletedList::restore_task(app);
        Ok(EventResult::Consumed)
    } else {
        Ok(EventResult::Ignored)
    }
}

fn universal_input(app: &mut App, key_event: KeyEvent) -> EventResult {
    // Global keybindings
    return match KeyBindings::from_event(&app.config, key_event) {
        KeyBindings::AddKey => {
            let add_input_dialog = InputBoxBuilder::default()
                .title(String::from("Add a task"))
                .callback(move |app, word| {
                    app.task_store
                        .add_task(Task::from_string(word.trim().to_string()));

                    Ok(())
                })
                .save_mode(app)
                .build();
            app.push_layer(add_input_dialog);
            EventResult::Consumed
        }
        KeyBindings::TasksMenuKey => {
            app.mode = Mode::CurrentTasks;
            EventResult::Consumed
        }
        KeyBindings::CompletedTasksMenuKey => {
            app.mode = Mode::CompletedTasks;
            EventResult::Consumed
        }
        KeyBindings::OpenHelpKey => {
            actions::open_help_menu(app);
            EventResult::Consumed
        }
        KeyBindings::QuitKey => {
            app.shutdown();
            EventResult::Consumed
        }
        KeyBindings::SortKey => {
            app.task_store.sort();
            EventResult::Consumed
        }
        KeyBindings::EnableAutosortKey => {
            app.task_store.auto_sort = !app.task_store.auto_sort;
            app.task_store.sort();
            EventResult::Consumed
        }
        _ => EventResult::Ignored,
    };
}
