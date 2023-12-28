use crossterm::event::KeyEvent;

use crate::{
    actions,
    app::{App, Mode},
    component::{
        completed_list::CompletedList,
        overlay::{input_box::InputBoxBuilder, Overlay},
    },
    draw::EventResult,
    task::Task,
    theme::KeyBindings,
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
    let theme = &app.theme;

    let selected_index = &mut app.task_list.selected_index;

    // Move this to the actions class
    match KeyBindings::from_event(&theme, key_event) {
        KeyBindings::ChangePriorityKey => {
            if app.task_store.tasks.is_empty() {
                return EventResult::Ignored;
            }

            let old_task = {
                let task = &mut app.task_store.tasks[*selected_index];

                task.priority = task.priority.next_priority();

                task.clone()
            };

            if app.task_store.auto_sort {
                app.task_store.sort();
            }

            *selected_index = app
                .task_store
                .tasks
                .iter()
                .position(|t| *t == old_task)
                .expect("getting task index after sorting")
                .to_owned();
        }
        KeyBindings::MoveTaskDown => {
            let tasks_length = app.task_store.tasks.len();

            if tasks_length == 0 {
                return EventResult::Ignored;
            }

            let new_index = (*selected_index + 1) % tasks_length;

            let task = &app.task_store.tasks[*selected_index];
            let task_below = &app.task_store.tasks[new_index];

            if task.priority == task_below.priority || !app.task_store.auto_sort {
                let task = app.task_store.tasks.remove(*selected_index);

                app.task_store.tasks.insert(new_index, task);
                *selected_index = new_index;
            }
        }
        KeyBindings::MoveTaskUp => {
            let tasks_length = app.task_store.tasks.len();

            if tasks_length == 0 {
                return EventResult::Ignored;
            }

            let new_index =
                (*selected_index as isize - 1).rem_euclid(tasks_length as isize) as usize;

            let task = &app.task_store.tasks[*selected_index];
            let task_above = &app.task_store.tasks[new_index];

            if task.priority == task_above.priority || !app.task_store.auto_sort {
                let task = app.task_store.tasks.remove(*selected_index);

                app.task_store.tasks.insert(new_index, task);
                *selected_index = new_index;
            }
        }
        KeyBindings::DeleteKey => actions::open_delete_task_menu(app),
        KeyBindings::EditKey => {
            let index = *selected_index;
            let edit_box = InputBoxBuilder::default()
                .title(String::from("Edit the selected task"))
                .fill(app.task_store.tasks[*selected_index].title.as_str())
                .callback(move |app, word| {
                    app.task_store.tasks[index].title = word.trim().to_string();
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
            app.task_store.tasks[*selected_index].progress =
                !app.task_store.tasks[*selected_index].progress;
        }
        KeyBindings::CompleteKey => actions::complete_task(app),
        _ => {
            return utils::handle_key_movement(
                &theme,
                key_event,
                selected_index,
                app.task_store.tasks.len(),
            );
        }
    }
    EventResult::Consumed
}

fn completed_list_input(app: &mut App, key_event: KeyEvent) -> EventResult {
    let result = utils::handle_key_movement(
        &app.theme,
        key_event,
        &mut app.completed_list.selected_index,
        app.task_store.completed_tasks.len(),
    );

    if result == EventResult::Consumed {
        return EventResult::Consumed;
    }

    if app.theme.restore_key.is_pressed(key_event) {
        CompletedList::restore_task(app);
        EventResult::Consumed
    } else {
        EventResult::Ignored
    }
}

fn universal_input(app: &mut App, key_event: KeyEvent) -> EventResult {
    // if event_result == EventResult::Consumed {
    //     return event_result;
    // }

    // Global keybindings
    return match KeyBindings::from_event(&app.theme, key_event) {
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
