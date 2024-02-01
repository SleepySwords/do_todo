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
    return match KeyBindings::from_event(theme, key_event) {
        KeyBindings::ChangePriorityKey => change_priority(app),
        KeyBindings::AddSubtaskKey => add_subtask(app),
        KeyBindings::MoveTaskDown => move_task_down(app),
        KeyBindings::MoveTaskUp => move_task_up(app),
        KeyBindings::DeleteKey => app.create_delete_selected_task_menu(),
        KeyBindings::EditKey => edit_selected_task(app),
        KeyBindings::TagMenu => app.create_tag_menu(),
        KeyBindings::FlipProgressKey => flip_progress_key(app),
        KeyBindings::CompleteKey => app.complete_selected_task(),
        KeyBindings::OpenSubtasksKey => open_subtasks_key(app),
        KeyBindings::MoveSubtaskLevelUp => move_subtask_level_up(app),
        KeyBindings::MoveSubtaskLevelDown => move_subtask_level_down(app),
        _ => {
            return Ok(utils::handle_key_movement(
                theme,
                key_event,
                selected_index,
                app.task_store.find_tasks_draw_size(),
            ));
        }
    }
}

fn change_priority(app: &mut App) -> Result<PostEvent, AppError> {
    if app.task_store.tasks.is_empty() {
        return Ok(PostEvent::noop(true));
    }

    let old_task = {
        let Some(task) = app.task_store.task_mut(app.task_list.selected_index) else {
            return Ok(PostEvent::noop(true));
        };
        task.priority = task.priority.next_priority();

        task.clone()
    };

    if app.task_store.auto_sort {
        app.task_store.sort();
    }

    app.task_list.selected_index = app.task_store.task_position(&old_task).ok_or_else(|| {
        AppError::InvalidState("Cannot find the selected tasks index.".to_string())
    })?;
    return Ok(PostEvent::noop(false));
}

fn add_subtask(app: &mut App) -> Result<PostEvent, AppError> {
    let index = app.task_list.selected_index;
    let add_input_dialog = InputBoxBuilder::default()
        .title(String::from("Add a task"))
        .callback(move |app, word| {
            if let Some(task) = app.task_store.task_mut(index) {
                task.sub_tasks
                    .push(Task::from_string(word.trim().to_string()));
                task.opened = true;
                app.task_list.selected_index += task.sub_tasks.len();
            }
            Ok(PostEvent::noop(false))
        })
        .save_mode(app)
        .build_overlay();
    return Ok(PostEvent::push_overlay(add_input_dialog));
}

fn move_task_down(app: &mut App) -> Result<PostEvent, AppError> {
    let autosort = app.task_store.auto_sort;

    let Some(FindParentResult {
        tasks: parent_tasks,
        parent_index,
        task_local_offset: local_index,
    }) = app.task_store.find_parent(app.task_list.selected_index)
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

        app.task_list.selected_index =
            TaskStore::local_index_to_global(new_index, parent_subtasks, parent_index);
    }
    Ok(PostEvent::noop(false))
}

fn move_task_up(app: &mut App) -> Result<PostEvent, AppError> {
    let auto_sort = app.task_store.auto_sort;

    let Some(FindParentResult {
        tasks: parent_subtasks,
        parent_index,
        task_local_offset: local_index,
    }) = app.task_store.find_parent(app.task_list.selected_index)
    else {
        return Ok(PostEvent::noop(true));
    };

    if parent_subtasks.is_empty() {
        return Ok(PostEvent::noop(true));
    }

    let new_index = (local_index as isize - 1).rem_euclid(parent_subtasks.len() as isize) as usize;

    let Some(mut_parent_subtasks) = app.task_store.subtasks(parent_index) else {
        return Ok(PostEvent::noop(true));
    };

    let task = &mut_parent_subtasks[local_index];
    let task_above = &mut_parent_subtasks[new_index];

    if task.priority == task_above.priority || !auto_sort {
        let task = mut_parent_subtasks.remove(local_index);

        mut_parent_subtasks.insert(new_index, task);

        app.task_list.selected_index =
            TaskStore::local_index_to_global(new_index, mut_parent_subtasks, parent_index);
    }
    Ok(PostEvent::noop(false))
}

fn edit_selected_task(app: &mut App) -> Result<PostEvent, AppError> {
    let index = app.task_list.selected_index;
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

fn flip_progress_key(app: &mut App) -> Result<PostEvent, AppError> {
    if app.task_store.tasks.is_empty() {
        return Ok(PostEvent::noop(true));
    }
    let Some(task) = app.task_store.task_mut(app.task_list.selected_index) else {
        return Ok(PostEvent::noop(true));
    };
    task.progress = !task.progress;
    Ok(PostEvent::noop(false))
}

fn open_subtasks_key(app: &mut App) -> Result<PostEvent, AppError> {
    let selected_index = &mut app.task_list.selected_index;
    if app.task_store.tasks.is_empty() {
        return Ok(PostEvent::noop(true));
    }
    let Some(task) = app.task_store.task_mut(*selected_index) else {
        return Ok(PostEvent::noop(true));
    };
    task.opened = !task.opened;
    Ok(PostEvent::noop(false))
}

fn move_subtask_level_up(app: &mut App) -> Result<PostEvent, AppError> {
    let selected_index = &mut app.task_list.selected_index;
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
    Ok(PostEvent::noop(false))
}

fn move_subtask_level_down(app: &mut App) -> Result<PostEvent, AppError> {
    let selected_index = &mut app.task_list.selected_index;
    let Some(FindParentResult { parent_index, .. }) = app.task_store.find_parent(*selected_index)
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
        .ok_or_else(|| AppError::InvalidState("Cannot find the parent tasks index.".to_string()))?;

    let Some(grandparent_subtasks) = app.task_store.subtasks(grandparent_index) else {
        return Ok(PostEvent::noop(true));
    };

    let new_index = parent_local_index + 1;
    grandparent_subtasks.insert(new_index, task);
    *selected_index =
        TaskStore::local_index_to_global(new_index, grandparent_subtasks, grandparent_index);
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
    Ok(PostEvent::noop(false))
}

fn task_list_help_entry(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![
        HelpEntry::new(config.add_key, "Adds a task"),
        HelpEntry::new(config.complete_key, "Completes the selected task"),
        HelpEntry::new(config.delete_key, "Delete the selected task"),
        HelpEntry::register_function(
            config.edit_key,
            "Edits the selected task",
            edit_selected_task,
        ),
        HelpEntry::new(
            config.tag_menu,
            "Add or remove the tags from this task or project",
        ),
        HelpEntry::register_function(
            config.change_priority_key,
            "Gives selected task lower priority",
            change_priority,
        ),
        HelpEntry::register_function(
            config.move_task_down,
            "Moves the task down on the task list",
            move_task_down,
        ),
        HelpEntry::register_function(
            config.move_task_up,
            "Moves the task up on the task list",
            move_task_up,
        ),
        HelpEntry::new_multiple(config.down_keys, "Moves down one task"),
        HelpEntry::new_multiple(config.down_keys, "Moves up one task"),
        HelpEntry::new(config.sort_key, "Sorts tasks (by priority)"),
        HelpEntry::new(config.enable_autosort_key, "Toggles automatic task sort"),
        HelpEntry::register_function(
            config.flip_subtask_key,
            "Open/closes the subtask",
            open_subtasks_key,
        ),
        HelpEntry::register_function(
            config.move_subtask_level_up,
            "Make the selected task a subtask of above",
            move_subtask_level_up,
        ),
        HelpEntry::register_function(
            config.move_subtask_level_down,
            "Make the selected task not a subtask of the parent",
            move_subtask_level_down,
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
                    if app.mode == Mode::CurrentTasks {
                        app.task_list.selected_index = app.task_store.find_tasks_draw_size() - 1;
                    }
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
