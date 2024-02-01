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

    // Move this to the actions class
    return match KeyBindings::from_event(theme, key_event) {
        KeyBindings::ChangePriorityKey => app.change_priority(),
        KeyBindings::AddSubtaskKey => app.add_subtask(),
        KeyBindings::MoveTaskDown => app.move_task_down(),
        KeyBindings::MoveTaskUp => app.move_task_up(),
        KeyBindings::DeleteKey => app.create_delete_selected_task_menu(),
        KeyBindings::EditKey => app.edit_selected_task(),
        KeyBindings::TagMenu => app.create_tag_menu(),
        KeyBindings::FlipProgressKey => app.flip_progress_key(),
        KeyBindings::CompleteKey => app.complete_selected_task(),
        KeyBindings::OpenSubtasksKey => app.open_subtasks_key(),
        KeyBindings::MoveSubtaskLevelUp => app.move_subtask_level_up(),
        KeyBindings::MoveSubtaskLevelDown => app.move_subtask_level_down(),
        _ => {
            return Ok(utils::handle_key_movement(
                theme,
                key_event,
                &mut app.task_list.selected_index,
                app.task_store.find_tasks_draw_size(),
            ));
        }
    };
}

fn task_list_help_entry(config: &Config) -> Vec<HelpEntry<'static>> {
    vec![
        HelpEntry::new(config.add_key, "Adds a task"),
        HelpEntry::new(config.complete_key, "Completes the selected task"),
        HelpEntry::register_function(
            config.delete_key,
            "Delete the selected task",
            App::create_delete_selected_task_menu,
        ),
        HelpEntry::register_function(
            config.edit_key,
            "Edits the selected task",
            App::edit_selected_task,
        ),
        HelpEntry::register_function(
            config.tag_menu,
            "Add or remove the tags from this task or project",
            App::create_tag_menu,
        ),
        HelpEntry::register_function(
            config.change_priority_key,
            "Gives selected task lower priority",
            App::change_priority,
        ),
        HelpEntry::register_function(
            config.move_task_down,
            "Moves the task down on the task list",
            App::move_task_down,
        ),
        HelpEntry::register_function(
            config.move_task_up,
            "Moves the task up on the task list",
            App::move_task_up,
        ),
        HelpEntry::new_multiple(config.down_keys, "Moves down one task"),
        HelpEntry::new_multiple(config.down_keys, "Moves up one task"),
        HelpEntry::new(config.sort_key, "Sorts tasks (by priority)"),
        HelpEntry::new(config.enable_autosort_key, "Toggles automatic task sort"),
        HelpEntry::register_function(
            config.flip_subtask_key,
            "Open/closes the subtask",
            App::open_subtasks_key,
        ),
        HelpEntry::register_function(
            config.move_subtask_level_up,
            "Make the selected task a subtask of above",
            App::move_subtask_level_up,
        ),
        HelpEntry::register_function(
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
