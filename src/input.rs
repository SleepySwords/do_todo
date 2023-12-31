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
    task::Task,
    utils,
};

pub fn key_event(app: &mut App, key_event: KeyEvent) -> EventResult {
    let event = match app.mode {
        Mode::Overlay => Overlay::key_event(app, key_event),
        Mode::CurrentTasks => task_list_input(app, key_event),
        Mode::CompletedTasks => completed_list_input(app, key_event),
    };
    if event == EventResult::Ignored {
        universal_input(app, key_event)
    } else {
        EventResult::Consumed
    }
}

fn task_list_input(app: &mut App, key_event: KeyEvent) -> EventResult {
    let theme = &app.config;

    let selected_index = &mut app.task_list.selected_index;

    // Move this to the actions class
    match KeyBindings::from_event(theme, key_event) {
        KeyBindings::ChangePriorityKey => {
            if app.task_store.tasks.is_empty() {
                return EventResult::Ignored;
            }

            let old_task = {
                let Some(task) = app.task_store.task_mut(*selected_index) else {
                    return EventResult::Ignored;
                };
                task.priority = task.priority.next_priority();

                task.clone()
            };

            if app.task_store.auto_sort {
                app.task_store.sort();
            }

            *selected_index = app
                .task_store
                .task_position(&old_task)
                .expect("getting task index after sorting")
                .to_owned();
        }
        KeyBindings::MoveTaskDown => {
            let autosort = app.task_store.auto_sort;

            let Some((parent_tasks, _, _)) = app.task_store.find_parent(*selected_index) else {
                return EventResult::Ignored;
            };

            let local_index = parent_tasks
                .iter()
                .position(|f| Some(f) == app.task_store.task(*selected_index))
                .expect("Invalid offset?");

            let new_index = (local_index + 1) % parent_tasks.len();

            let Some((mut_parent_tasks, offset, is_global)) =
                app.task_store.find_parent_mut(*selected_index)
            else {
                return EventResult::Ignored;
            };

            let task = &mut_parent_tasks[local_index];
            let task_above = &mut_parent_tasks[new_index];

            // FIXME: potential refactor into another method
            if task.priority == task_above.priority || !autosort {
                let task = mut_parent_tasks.remove(local_index);

                mut_parent_tasks.insert(new_index, task);

                *selected_index = offset
                    + mut_parent_tasks
                        .iter()
                        .take(new_index)
                        .map(|task| task.find_task_draw_size())
                        .sum::<usize>() + if is_global { 0 } else { 1 };
            }
        }
        KeyBindings::MoveTaskUp => {
            let autosort = app.task_store.auto_sort;

            let Some((parent_tasks, _, _)) = app.task_store.find_parent(*selected_index) else {
                return EventResult::Ignored;
            };

            if parent_tasks.len() == 0 {
                return EventResult::Ignored;
            }

            let local_index = parent_tasks
                .iter()
                .position(|f| Some(f) == app.task_store.task(*selected_index))
                .expect("Invalid offset?");

            let new_index =
                (local_index as isize - 1).rem_euclid(parent_tasks.len() as isize) as usize;

            let Some((mut_parent_tasks, offset, is_global)) =
                app.task_store.find_parent_mut(*selected_index)
            else {
                return EventResult::Ignored;
            };

            let task = &mut_parent_tasks[local_index];
            let task_above = &mut_parent_tasks[new_index];

            if task.priority == task_above.priority || !autosort {
                let task = mut_parent_tasks.remove(local_index);

                mut_parent_tasks.insert(new_index, task);

                *selected_index = offset
                    + mut_parent_tasks
                        .iter()
                        .take(new_index)
                        .map(|tsk| tsk.find_task_draw_size())
                        .sum::<usize>() + if is_global { 0 } else { 1 };
            }
        }
        KeyBindings::DeleteKey => actions::open_delete_task_menu(app),
        KeyBindings::EditKey => {
            let index = *selected_index;
            let Some(task) = app.task_store.task(index) else {
                return EventResult::Ignored;
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
                return EventResult::Ignored;
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return EventResult::Ignored;
            };
            task.progress = !task.progress;
        }
        KeyBindings::CompleteKey => actions::complete_task(app),
        KeyBindings::OpenSubtasksKey => {
            if app.task_store.tasks.is_empty() {
                return EventResult::Ignored;
            }
            let Some(task) = app.task_store.task_mut(*selected_index) else {
                return EventResult::Ignored;
            };
            task.opened = !task.opened;
        }
        KeyBindings::MoveSubtaskLevelUp => {
            let Some((parent_tasks, offset, is_global)) =
                app.task_store.find_parent(*selected_index)
            else {
                return EventResult::Ignored;
            };

            if parent_tasks.len() == 0 {
                return EventResult::Ignored;
            }

            let local_index = parent_tasks
                .iter()
                .position(|f| Some(f) == app.task_store.task(*selected_index))
                .expect("Invalid offset?");

            if local_index == 0 {
                return EventResult::Ignored;
            }

            let prev_index = local_index - 1;
            let prev_global_offset = offset
                + parent_tasks
                    .iter()
                    .take(prev_index)
                    .map(|tsk| tsk.find_task_draw_size())
                    // Focus the correct element, if it's global however
                    // no need to do this.
                    .sum::<usize>() + if is_global { 0 } else { 1 };

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return EventResult::Ignored;
            };

            let Some(prev_task) = app.task_store.task_mut(prev_global_offset) else {
                return EventResult::Ignored;
            };

            prev_task.opened = true;
            prev_task.sub_tasks.push(task);
        }
        KeyBindings::MoveSubtaskLevelDown => {
            let Some((_, offset, is_global)) = app.task_store.find_parent(*selected_index) else {
                return EventResult::Ignored;
            };

            if is_global {
                return EventResult::Ignored;
            }

            let Some(task) = app.task_store.delete_task(*selected_index) else {
                return EventResult::Ignored;
            };

            let Some((prev_task_list, _, _)) = app.task_store.find_parent(offset) else {
                return EventResult::Ignored;
            };

            let parent_local_index = prev_task_list
                .iter()
                .position(|t| {
                    t == app
                        .task_store
                        .task(offset)
                        .expect("This is definately a task")
                })
                .expect("This is not possible?");

            if let Some((prev_task_list, parent_global_index, is_global)) =
                app.task_store.find_parent_mut(offset)
            {
                let new_index = parent_local_index + 1;
                prev_task_list.insert(new_index, task);
                *selected_index = parent_global_index
                    + prev_task_list
                        .iter()
                        .take(new_index)
                        .map(|tsk| tsk.find_task_draw_size())
                        .sum::<usize>() + if is_global { 0 } else { 1 }
            }
        }
        _ => {
            return utils::handle_key_movement(
                theme,
                key_event,
                selected_index,
                app.task_store.find_task_size(),
            );
        }
    }
    EventResult::Consumed
}

fn completed_list_input(app: &mut App, key_event: KeyEvent) -> EventResult {
    let result = utils::handle_key_movement(
        &app.config,
        key_event,
        &mut app.completed_list.selected_index,
        app.task_store.completed_tasks.len(),
    );

    if result == EventResult::Consumed {
        return EventResult::Consumed;
    }

    if app.config.restore_key.is_pressed(key_event) {
        CompletedList::restore_task(app);
        EventResult::Consumed
    } else {
        EventResult::Ignored
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
