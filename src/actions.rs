use chrono::Local;
use crossterm::event::{KeyCode, KeyModifiers};
use tui::style::Color;

use crate::{
    app::{App, SelectedComponent, UserInputType},
    component::{
        input::dialog::{DialogAction, DialogBox},
        input::input_box::InputBox,
    },
    input::handle_key,
    task::{CompletedTask, Task},
};

// Action class maybe?!!
pub struct HelpAction<'a> {
    character: KeyCode,
    short_hand: &'a str,
    description: &'a str,
}

impl HelpAction<'_> {
    pub fn new<'a>(
        character: KeyCode,
        short_hand: &'a str,
        description: &'a str,
    ) -> HelpAction<'a> {
        HelpAction {
            character,
            short_hand,
            description,
        }
    }
}

pub fn open_help_menu(app: &mut App) {
    // Actions that are universal, should use a table?
    let mut actions: Vec<DialogAction> = vec![
        DialogAction::new(String::from("1    Change to current task window"), |app| {
            app.selected_component = SelectedComponent::CurrentTasks;
        }),
        DialogAction::new(
            String::from("2    Change to completed task window"),
            |app| {
                app.selected_component = SelectedComponent::CompletedTasks;
            },
        ),
    ];
    for ac in app.selected_component.available_help_actions() {
        actions.push(DialogAction::new(
            format!("{}    {}", ac.short_hand, ac.description),
            move |app| {
                handle_key(
                    crossterm::event::KeyEvent {
                        code: ac.character,
                        modifiers: KeyModifiers::NONE,
                    },
                    app,
                );
            },
        ));
    }

    app.append_layer(UserInputType::Dialog(DialogBox::new(
        String::from("Help Menu"),
        actions,
    )));
}

pub fn open_delete_task_menu(app: &mut App, selected_index: usize) {
    if app.task_store.tasks.is_empty() {
        return;
    }
    app.append_layer(UserInputType::Dialog(DialogBox::new(
        "Delete selected task".to_string(),
        vec![
            DialogAction::new(String::from("Delete"), move |app| {
                app.task_store.tasks.remove(selected_index);
                if selected_index == app.task_store.tasks.len() && !app.task_store.tasks.is_empty()
                {
                    app.selected_task_index -= 1;
                }
            }),
            DialogAction::new(String::from("Cancel"), |_| {}),
        ],
    )));
}

pub fn restore_task(app: &mut App, selected_index: usize) {
    if app.task_store.completed_tasks.is_empty() {
        return;
    }
    app.task_store.tasks.push(Task::from_completed_task(
        app.task_store.completed_tasks.remove(selected_index),
    ));
    if selected_index == app.task_store.completed_tasks.len()
        && !app.task_store.completed_tasks.is_empty()
    {
        app.selected_completed_task_index -= 1;
    }
}

pub fn complete_task(app: &mut App, selected_index: usize) {
    if app.task_store.tasks.is_empty() {
        return;
    }
    let local = Local::now();
    let time_completed = local.naive_local();
    let task = app.task_store.tasks.remove(selected_index);
    app.task_store
        .completed_tasks
        .push(CompletedTask::from_task(task, time_completed));
    if selected_index == app.task_store.tasks.len() && !app.task_store.tasks.is_empty() {
        app.selected_task_index -= 1;
    }
}

// TODO: Add a way to remove tags
pub fn tag_menu(app: &mut App, selected_index: usize) {
    if app.task_store.tasks.is_empty() {
        return;
    }

    let mut tags_options: Vec<DialogAction> = Vec::new();

    // Loops through the tags and adds them to the menu.
    for (i, tag) in app.task_store.tags.iter() {
        let moved: u32 = *i;
        // TODO: Allow for DialogBox to support colours.
        tags_options.push(DialogAction::new(String::from(&tag.name), move |app| {
            app.task_store.tasks[selected_index].flip_tag(moved);
        }));
    }

    // Menu to add a new tag
    // FIX: ooof this is some ugly ass code
    tags_options.push(DialogAction::new(String::from("New tag"), move |app| {
        app.append_layer(UserInputType::Input(InputBox::new(
            String::from("Tag name"),
            move |app, tag_name| {
                app.append_layer(UserInputType::Input(InputBox::new(
                    String::from("Tag colour"),
                    move |app, tag_colour| {
                        // FIX: Actually have proper error handling with an error enum
                        // TODO: Add colour word support (ie: green, blue, red, orange)
                        let red = u8::from_str_radix(&tag_colour[0..2], 16)?;
                        let green = u8::from_str_radix(&tag_colour[2..4], 16)?;
                        let blue = u8::from_str_radix(&tag_colour[4..6], 16)?;
                        let tag_id = app.task_store.tags.keys().last().map_or(0, |id| *id + 1);
                        app.task_store.tags.insert(
                            tag_id,
                            crate::task::Tag {
                                // FIX: I can't be bothered fixing this ownership problem
                                name: tag_name.to_owned(),
                                colour: Color::Rgb(red, green, blue),
                            },
                        );
                        app.task_store.tasks[selected_index].flip_tag(tag_id);
                        Ok(())
                    },
                )));
                Ok(())
            },
        )));
    }));
    tags_options.push(DialogAction::new(String::from("Clear tags"), move |app| {
        app.task_store.tasks[selected_index].tags.clear();
    }));
    tags_options.push(DialogAction::new(String::from("Cancel"), |_| {}));
    app.append_layer(UserInputType::Dialog(DialogBox::new(
        "Add or remove a tag".to_string(),
        tags_options,
    )));
}
