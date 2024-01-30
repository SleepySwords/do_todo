use chrono::NaiveDate;
use crossterm::event::KeyEvent;

use crate::{
    actions::HelpEntry,
    app::{App, Mode, ScreenManager},
    component::{
        completed_list::CompletedList,
        overlay::{input_box::InputBoxBuilder, Overlay},
    },
    config::{Config, KeyBindings},
    draw::PostEvent,
    error::AppError,
    task::{FindParentResult, Task, TaskStore},
    utils,
};

pub fn key_event(
    screen_manager: &mut ScreenManager,
    key_event: KeyEvent,
) -> Result<PostEvent, AppError> {
    let event = match screen_manager.app.mode {
        Mode::Overlay => Overlay::key_event(screen_manager, key_event),
        Mode::CurrentTasks => task_list_input(&mut screen_manager.app, key_event),
        Mode::CompletedTasks => completed_list_input(&mut screen_manager.app, key_event),
    };
    if let Ok(PostEvent {
        propegate_further: true,
        ..
    }) = event
    {
        Ok(universal_input(&mut screen_manager.app, key_event))
    } else {
        event
    }
}

pub fn help_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    let event = match app.mode {
        Mode::CurrentTasks => task_list_input(app, key_event),
        Mode::CompletedTasks => completed_list_input(app, key_event),
        _ => Ok(PostEvent::noop(false)),
    };
    if let Ok(PostEvent {
        propegate_further: true,
        ..
    }) = event
    {
        Ok(universal_input(app, key_event))
    } else {
        event
    }
}

fn task_list_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    let theme = &app.config;

    let selected_index = &mut app.task_list.selected_index;

    // Move this to the actions class
    match KeyBindings::from_event(theme, key_event) {
        KeyBindings::ChangePriorityKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(PostEvent::noop(true));
            }

            let old_task = {
                let Some(task) = app.task_store.task_mut(*selected_index) else {
                    return Ok(PostEvent::noop(true));
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
        KeyBindings::AddDate => {
            let input = InputBoxBuilder::default()
                .title("Date".to_string())
                .save_mode(app)
                .callback(|app, date| {
                    app.println(format!("{:?}", NaiveDate::parse_from_str(&date, "%Y")));
                    Ok(PostEvent::noop(false))
                })
                .build_overlay();
            return Ok(PostEvent::push_overlay(input));
        }
        KeyBindings::MoveTaskDown => {
            let autosort = app.task_store.auto_sort;

            let Some(FindParentResult {
                tasks: parent_tasks,
                parent_index,
                task_local_offset: local_index,
            }) = app.task_store.find_parent(*selected_index)
            else {
                return Ok(PostEvent::noop(true));
            };

            let new_index = (local_index + 1) % parent_tasks.len();

            let Some(parent_subtasks) = app.task_store.subtasks(parent_index) else {
                return Ok(PostEvent::noop(true));
            };

            let task = &parent_subtasks[local_index];
            let task_above = &parent_subtasks[new_index];

            if task.priority == task_above.priority || !autosort {
                let task = parent_subtasks.remove(local_index);

                parent_subtasks.insert(new_index, task);

                *selected_index =
                    TaskStore::local_index_to_global(new_index, parent_subtasks, parent_index);
            }
        }
        KeyBindings::MoveTaskUp => {
            let autosort = app.task_store.auto_sort;

            let Some(FindParentResult {
                tasks: parent_subtasks,
                parent_index,
                task_local_offset: local_index,
            }) = app.task_store.find_parent(*selected_index)
            else {
                return Ok(PostEvent::noop(true));
            };

            if parent_subtasks.is_empty() {
                return Ok(PostEvent::noop(true));
            }

            let new_index =
                (local_index as isize - 1).rem_euclid(parent_subtasks.len() as isize) as usize;

            let Some(mut_parent_subtasks) = app.task_store.subtasks(parent_index) else {
                return Ok(PostEvent::noop(true));
            };

            let task = &mut_parent_subtasks[local_index];
            let task_above = &mut_parent_subtasks[new_index];

            if task.priority == task_above.priority || !autosort {
                let task = mut_parent_subtasks.remove(local_index);

                mut_parent_subtasks.insert(new_index, task);

                *selected_index =
                    TaskStore::local_index_to_global(new_index, mut_parent_subtasks, parent_index)
            }
        }
        KeyBindings::DeleteKey => {
            return Ok(app.create_delete_selected_task_menu());
        }
        KeyBindings::EditKey => {
            let index = *selected_index;
            let Some(task) = app.task_store.task(index) else {
                return Ok(PostEvent::noop(true));
            };
            let edit_box = InputBoxBuilder::default()
                .title(String::from("Edit the selected task"))
                .fill(task.title.as_str())
                .callback(move |app, word| {
                    let Some(task) = app.task_store.task_mut(index) else {
                        return Ok(PostEvent::noop(false));
                    };
                    task.title = word.trim().to_string();
                    Ok(PostEvent::noop(false))
                })
                .save_mode(app)
                .build_overlay();
            return Ok(PostEvent::push_overlay(edit_box));
        }
        KeyBindings::TagMenu => return Ok(app.create_tag_menu()),
        KeyBindings::FlipProgressKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(PostEvent::noop(true));
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return Ok(PostEvent::noop(true));
            };
            task.progress = !task.progress;
        }
        KeyBindings::CompleteKey => app.complete_selected_task(),
        KeyBindings::OpenSubtasksKey => {
            if app.task_store.tasks.is_empty() {
                return Ok(PostEvent::noop(true));
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return Ok(PostEvent::noop(true));
            };
            task.opened = !task.opened;
        }
        KeyBindings::MoveSubtaskLevelUp => {
            let Some(FindParentResult {
                tasks: parent_tasks,
                parent_index,
                task_local_offset: local_index,
            }) = app.task_store.find_parent(*selected_index)
            else {
                return Ok(PostEvent::noop(true));
            };

            if parent_tasks.is_empty() {
                return Ok(PostEvent::noop(true));
            }

            if local_index == 0 {
                return Ok(PostEvent::noop(true));
            }

            let prev_local_index = local_index - 1;
            let prev_global_index =
                TaskStore::local_index_to_global(prev_local_index, parent_tasks, parent_index);

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return Ok(PostEvent::noop(true));
            };

            let Some(prev_task) = app.task_store.task_mut(prev_global_index) else {
                return Ok(PostEvent::noop(true));
            };

            if !prev_task.opened {
                prev_task.opened = true;
                // Have to remove the task when adding
                *selected_index += prev_task.find_task_draw_size() - 1;
            }
            prev_task.sub_tasks.push(task);

            // FIXME: refactor this to ideally not clone
            if app.task_store.auto_sort {
                let Some(task) = app.task_store.task(*selected_index).cloned() else {
                    return Err(AppError::InvalidState(
                        "There is no task selected.".to_string(),
                    ));
                };
                app.task_store.sort();
                if let Some(task_pos) = app.task_store.task_position(&task) {
                    *selected_index = task_pos;
                }
            }
        }
        KeyBindings::MoveSubtaskLevelDown => {
            let Some(FindParentResult { parent_index, .. }) =
                app.task_store.find_parent(*selected_index)
            else {
                return Ok(PostEvent::noop(true));
            };

            let Some(parent_index) = parent_index else {
                return Ok(PostEvent::noop(true));
            };

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return Ok(PostEvent::noop(true));
            };

            let Some(FindParentResult {
                tasks: grandparent_subtasks,
                parent_index: grandparent_index,
                ..
            }) = app.task_store.find_parent(parent_index)
            else {
                return Ok(PostEvent::noop(true));
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

            let Some(grandparent_subtasks) = app.task_store.subtasks(grandparent_index) else {
                return Ok(PostEvent::noop(true));
            };

            let new_index = parent_local_index + 1;
            grandparent_subtasks.insert(new_index, task);
            *selected_index = TaskStore::local_index_to_global(
                new_index,
                grandparent_subtasks,
                grandparent_index,
            );
            // FIXME: refactor this to ideally not clone
            if app.task_store.auto_sort {
                let Some(task) = app.task_store.task(*selected_index).cloned() else {
                    return Err(AppError::InvalidState(
                        "Invalid selected index for this task.".to_string(),
                    ));
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
    Ok(PostEvent::noop(false))
}

fn task_list_help_entry(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![
        HelpEntry::new(config.add_key, "Adds a task"),
        HelpEntry::new(config.complete_key, "Completes the selected task"),
        HelpEntry::new(config.delete_key, "Delete the selected task"),
        HelpEntry::new(config.edit_key, "Edits the selected task"),
        HelpEntry::new(
            config.tag_menu,
            "Add or remove the tags from this task or project",
        ),
        HelpEntry::new(
            config.change_priority_key,
            "Gives selected task lower priority",
        ),
        HelpEntry::new(
            config.move_task_down,
            "Moves the task down on the task list",
        ),
        HelpEntry::new(config.move_task_up, "Moves the task up on the task list"),
        HelpEntry::new_multiple(config.down_keys, "Moves down one task"),
        HelpEntry::new_multiple(config.down_keys, "Moves up one task"),
        HelpEntry::new(config.sort_key, "Sorts tasks (by priority)"),
        HelpEntry::new(config.enable_autosort_key, "Toggles automatic task sort"),
        HelpEntry::new(config.flip_subtask_key, "Open/closes the subtask"),
        HelpEntry::new(
            config.move_subtask_level_up,
            "Make the selected task a subtask of above",
        ),
        HelpEntry::new(
            config.move_subtask_level_down,
            "Make the selected task not a subtask of the parent",
        ),
    ]
}

fn completed_list_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    let result = utils::handle_key_movement(
        &app.config,
        key_event,
        &mut app.completed_list.selected_index,
        app.task_store.completed_tasks.len(),
    );

    if !result.propegate_further {
        return Ok(result);
    }

    if app.config.restore_key.is_pressed(key_event) {
        CompletedList::restore_task(app);
        Ok(PostEvent::noop(false))
    } else {
        Ok(PostEvent::noop(true))
    }
}

fn completed_list_help_entries(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![HelpEntry::new(
        config.restore_key,
        "Restores the selected task",
    )]
}

fn universal_input(app: &mut App, key_event: KeyEvent) -> PostEvent {
    // Global keybindings
    return match KeyBindings::from_event(&app.config, key_event) {
        KeyBindings::AddKey => {
            let add_input_dialog = InputBoxBuilder::default()
                .title(String::from("Add a task"))
                .callback(move |app, word| {
                    app.task_store
                        .add_task(Task::from_string(word.trim().to_string()));

                    Ok(PostEvent::noop(false))
                })
                .save_mode(app)
                .build_overlay();
            return PostEvent::push_overlay(add_input_dialog);
        }
        KeyBindings::TasksMenuKey => {
            app.mode = Mode::CurrentTasks;
            PostEvent::noop(false)
        }
        KeyBindings::CompletedTasksMenuKey => {
            app.mode = Mode::CompletedTasks;
            PostEvent::noop(false)
        }
        KeyBindings::OpenHelpKey => {
            return app.create_help_menu();
        }
        KeyBindings::QuitKey => {
            app.shutdown();
            PostEvent::noop(false)
        }
        KeyBindings::SortKey => {
            app.task_store.sort();
            PostEvent::noop(false)
        }
        KeyBindings::EnableAutosortKey => {
            app.task_store.auto_sort = !app.task_store.auto_sort;
            app.task_store.sort();
            PostEvent::noop(false)
        }
        _ => PostEvent::noop(true),
    };
}

impl Mode {
    pub fn help_entries(&self, config: &Config) -> Vec<HelpEntry<'_>> {
        match self {
            Mode::CurrentTasks => task_list_help_entry(config),
            Mode::CompletedTasks => completed_list_help_entries(config),
            Mode::Overlay => vec![],
        }
    }
}
