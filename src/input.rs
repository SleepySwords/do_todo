use crossterm::event::KeyEvent;

use crate::{
    app::{App, Mode, ScreenManager},
    component::{
        completed_list::CompletedList,
        overlay::{input_box::InputBoxBuilder, vim::VimMode, Overlay},
    },
    component::{
        completed_list::CompletedList,
        overlay::{input_box::InputBoxBuilder, Overlay},
    },
    config::{Config, KeyBindings},
    component::{completed_list::CompletedList, overlay::Overlay},
    config::Config,
    draw::PostEvent,
    error::AppError,
    key::KeyBinding,
    utils,
};

fn task_list_help_entry(config: &Config) -> Vec<KeyBinding<'static>> {
    vec![
        KeyBinding::register_key(
            config.complete_key,
            "Completes the selected task",
            App::complete_selected_task,
        ),
        KeyBinding::register_key(
            config.delete_key,
            "Delete the selected task",
            App::create_delete_selected_task_menu,
        ),
        KeyBinding::register_key(
            config.edit_key,
            "Edits the selected task",
            App::create_edit_selected_task_menu,
        ),
        KeyBinding::register_key(
            config.add_subtask_key,
            "Adds a subtask to the selected task",
            App::create_add_subtask_menu,
        ),
        KeyBinding::register_key(
            config.tag_menu,
            "Add or remove the tags from this task or project",
            App::create_tag_menu,
        ),
        KeyBinding::register_key(
            config.change_priority_key,
            "Gives selected task lower priority",
            App::cycle_priority,
        ),
        KeyBinding::register_key(
            config.move_task_down,
            "Moves the task down on the task list",
            App::move_selected_task_down,
        ),
        KeyBinding::register_key(
            config.move_task_up,
            "Moves the task up on the task list",
            App::move_selected_task_up,
        ),
        KeyBinding::new_multiple(config.down_keys, "Moves down one task"),
        KeyBinding::new_multiple(config.up_keys, "Moves up one task"),
        KeyBinding::register_key(
            config.flip_progress_key,
            "Open/closes the subtask",
            App::flip_selected_task_progress,
        ),
        KeyBinding::register_key(
            config.flip_subtask_key,
            "Open/closes the subtask",
            App::flip_subtasks,
        ),
        KeyBinding::register_key(
            config.move_subtask_level_up,
            "Make the selected task a subtask of above",
            App::move_subtask_level_up,
        ),
        KeyBinding::register_key(
            config.move_subtask_level_down,
            "Make the selected task not a subtask of the parent",
            App::move_subtask_level_down,
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

// Global keybinds
pub fn universal_input_keys(config: &Config) -> Vec<KeyBinding<'static>> {
    vec![
        KeyBinding::register_key(config.add_key, "Adds a task", App::create_add_task_menu),
        KeyBinding::register_key(
            config.tasks_menu_key,
            "Goes to the task menu",
            App::go_to_task_list,
        ),
        KeyBinding::register_key(
            config.completed_tasks_menu_key,
            "Goes to the completed task menu",
            App::go_to_completed_list,
        ),
        KeyBinding::register_key(
            config.open_help_key,
            "Opens the help menu",
            App::create_help_menu,
        ),
        KeyBinding::register_key(config.quit_key, "Quits the app", App::shutdown),
        KeyBinding::register_key(config.sort_key, "Sorts tasks (by priority)", App::sort),
        KeyBinding::register_key(
            config.enable_autosort_key,
            "Toggles automatic task sort",
            App::enable_auto_sort,
        ),
    ]
}

fn completed_list_help_entries(config: &Config) -> Vec<KeyBinding<'static>> {
    vec![KeyBinding::new(
        config.restore_key,
        "Restores the selected task",
    )]
}

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
        universal_input(&mut screen_manager.app, key_event)
    } else {
        event
    }
}

pub fn help_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    let event = match app.mode {
        Mode::CurrentTasks => task_list_input(app, key_event),
        Mode::CompletedTasks => completed_list_input(app, key_event),
        _ => Ok(PostEvent::noop(true)),
    };
    if let Ok(PostEvent {
        propegate_further: true,
        ..
    }) = event
    {
        universal_input(app, key_event)
    } else {
        event
    }
}

fn task_list_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    let result = utils::handle_key_movement(
        &app.config,
        key_event,
        &mut app.task_list.selected_index,
        app.task_store.find_tasks_draw_size(),
    );

    if !result.propegate_further {
        return Ok(result);
    }

    for entry in task_list_help_entry(&app.config) {
        if entry.character.is_pressed(key_event) {
            if let Some(function) = entry.function {
                return function(app);
            }
        }
    }
    Ok(PostEvent::noop(true))
}

fn universal_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    for entry in universal_input_keys(&app.config) {
        if entry.character.is_pressed(key_event) {
            if let Some(function) = entry.function {
                return function(app);
            }
        }
    }
    Ok(PostEvent::noop(true))
}

impl Mode {
    pub fn help_entries(&self, config: &Config) -> Vec<KeyBinding<'_>> {
        match self {
            Mode::CurrentTasks => task_list_help_entry(config),
            Mode::CompletedTasks => completed_list_help_entries(config),
            Mode::Overlay => vec![],
        }
    }
}
