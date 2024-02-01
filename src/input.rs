use crossterm::event::KeyEvent;

use crate::{
    actions::HelpEntry,
    app::{App, Mode, ScreenManager},
    component::{completed_list::CompletedList, overlay::Overlay},
    config::Config,
    draw::PostEvent,
    error::AppError,
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
    return Ok(PostEvent::noop(true));
}

fn task_list_help_entry(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![
        HelpEntry::register_key(
            config.complete_key,
            "Completes the selected task",
            App::complete_selected_task,
        ),
        HelpEntry::register_key(
            config.delete_key,
            "Delete the selected task",
            App::create_delete_selected_task_menu,
        ),
        HelpEntry::register_key(
            config.edit_key,
            "Edits the selected task",
            App::open_edit_selected_task,
        ),
        HelpEntry::register_key(
            config.tag_menu,
            "Add or remove the tags from this task or project",
            App::create_tag_menu,
        ),
        HelpEntry::register_key(
            config.change_priority_key,
            "Gives selected task lower priority",
            App::change_priority,
        ),
        HelpEntry::register_key(
            config.move_task_down,
            "Moves the task down on the task list",
            App::move_task_down,
        ),
        HelpEntry::register_key(
            config.move_task_up,
            "Moves the task up on the task list",
            App::move_task_up,
        ),
        HelpEntry::new_multiple(config.down_keys, "Moves down one task"),
        HelpEntry::new_multiple(config.down_keys, "Moves up one task"),
        HelpEntry::register_key(
            config.flip_subtask_key,
            "Open/closes the subtask",
            App::open_subtasks_key,
        ),
        HelpEntry::register_key(
            config.move_subtask_level_up,
            "Make the selected task a subtask of above",
            App::move_subtask_level_up,
        ),
        HelpEntry::register_key(
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

fn completed_list_help_entries(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![HelpEntry::new(
        config.restore_key,
        "Restores the selected task",
    )]
}

fn universal_input(app: &mut App, key_event: KeyEvent) -> Result<PostEvent, AppError> {
    // Global keybindings
    for entry in universal_input_keys(&app.config) {
        if entry.character.is_pressed(key_event) {
            if let Some(function) = entry.function {
                return function(app);
            }
        }
    }
    return Ok(PostEvent::noop(true));
}

// Make this lazy static or something
fn universal_input_keys(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![
        HelpEntry::register_key(config.add_key, "Adds a task", App::create_add_task_dialog),
        HelpEntry::register_key(
            config.tasks_menu_key,
            "Goes to the task menu",
            App::go_to_task_list,
        ),
        HelpEntry::register_key(
            config.completed_tasks_menu_key,
            "Goes to the task menu",
            App::go_to_completed_list,
        ),
        HelpEntry::register_key(
            config.open_help_key,
            "Goes to the task menu",
            App::create_help_menu,
        ),
        HelpEntry::register_key(
            config.quit_key,
            "Quits the app",
            App::shutdown,
        ),
        HelpEntry::register_key(
            config.sort_key,
            "Goes to the task menu",
            App::sort,
        ),
        HelpEntry::register_key(
            config.enable_autosort_key,
            "Goes to the task menu",
            App::enable_auto_sort,
        ),
    ]
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
