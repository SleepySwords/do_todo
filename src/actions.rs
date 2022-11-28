use std::{cell::RefCell, rc::Rc};

use chrono::Local;
use crossterm::event::KeyCode;
use tui::style::Color;

use crate::{
    app::{App, SelectedComponent},
    component::{
        input::dialog::{DialogAction, DialogBox},
        input::input_box::InputBox,
        message_box::MessageBox,
    },
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
                app.execute_event(ac.character);
            },
        ));
    }

    app.append_layer(DialogBox::new(String::from("Help Menu"), actions));
}

pub fn open_delete_task_menu(app: &mut App, selected_index: Rc<RefCell<usize>>) {
    if app.task_store.tasks.is_empty() {
        return;
    }
    app.append_layer(DialogBox::new(
        "Delete selected task".to_string(),
        vec![
            DialogAction::new(String::from("Delete"), move |app| {
                let mut selected_index = selected_index.borrow_mut();
                app.task_store.tasks.remove(*selected_index);
                if *selected_index == app.task_store.tasks.len() && !app.task_store.tasks.is_empty()
                {
                    *selected_index -= 1;
                }
            }),
            DialogAction::new(String::from("Cancel"), |_| {}),
        ],
    ));
}

pub fn restore_task(app: &mut App, selected_index: &mut usize) {
    if app.task_store.completed_tasks.is_empty() {
        return;
    }
    app.task_store.tasks.push(Task::from_completed_task(
        app.task_store.completed_tasks.remove(*selected_index),
    ));
    if *selected_index == app.task_store.completed_tasks.len()
        && !app.task_store.completed_tasks.is_empty()
    {
        *selected_index -= 1;
    }
}

pub fn complete_task(app: &mut App, selected_index: &mut usize) {
    if app.task_store.tasks.is_empty() {
        return;
    }
    let local = Local::now();
    let time_completed = local.naive_local();
    let task = app.task_store.tasks.remove(*selected_index);
    app.task_store
        .completed_tasks
        .push(CompletedTask::from_task(task, time_completed));
    if *selected_index == app.task_store.tasks.len() && !app.task_store.tasks.is_empty() {
        *selected_index -= 1;
    }
}

pub fn tag_menu(app: &mut App, selected_index: usize) {
    if app.task_store.tasks.is_empty() {
        return;
    }

    let mut tag_options: Vec<DialogAction> = Vec::new();

    // Loops through the tags and adds them to the menu.
    for (i, tag) in app.task_store.tags.iter() {
        let moved: u32 = *i;
        // TODO: Allow for DialogBox to support colours.
        tag_options.push(DialogAction::new(String::from(&tag.name), move |app| {
            app.task_store.tasks[selected_index].flip_tag(moved);
        }));
    }

    // FIX: ooof this is some ugly ass code
    tag_options.push(DialogAction::new(String::from("New tag"), move |app| {
        app.append_layer(InputBox::new(
            String::from("Tag name"),
            move |app, tag_name| {
                open_select_tag_colour(app, selected_index, tag_name);
                Ok(())
            },
        ));
    }));
    tag_options.push(DialogAction::new(
        String::from("Clear all tags"),
        move |app| {
            app.task_store.tasks[selected_index].tags.clear();
        },
    ));
    tag_options.push(DialogAction::new(
        String::from("Delete a tag"),
        move |app| {
            delete_tag_menu(app);
        },
    ));
    tag_options.push(DialogAction::new(String::from("Cancel"), |_| {}));

    app.append_layer(DialogBox::new(
        "Add or remove a tag".to_string(),
        tag_options,
    ));
}

pub fn delete_tag_menu(app: &mut App) {
    let mut tag_options: Vec<DialogAction> = Vec::new();

    for (i, tag) in app.task_store.tags.iter() {
        let moved: u32 = *i;
        let moved_name = tag.name.clone();
        // TODO: Allow for DialogBox to support colours.
        tag_options.push(DialogAction::new(String::from(&tag.name), move |app| {
            app.append_layer(DialogBox::new(
                format!(
                    "Do you want to permenatly delete the tag {}",
                    moved_name
                ),
                vec![
                    DialogAction::new(String::from("yes"), move |app| {
                        app.task_store.delete_tag(moved)
                    }),
                    DialogAction::new(String::from("no"), move |_| {}),
                ],
            ));
        }));
    }
    tag_options.push(DialogAction::new(String::from("Cancel"), |_| {}));

    app.append_layer(DialogBox::new("Delete a tag".to_string(), tag_options));
}

fn open_select_tag_colour(app: &mut App, selected_index: usize, tag_name: String) {
    let tag = tag_name.clone();
    app.append_layer(InputBox::new_with_error_callback(
        String::from("Tag colour"),
        move |app, tag_colour| {
            // TODO: Add colour word support (ie: green, blue, red, orange)
            let red = u8::from_str_radix(&tag_colour[0..2], 16)?;
            let green = u8::from_str_radix(&tag_colour[2..4], 16)?;
            let blue = u8::from_str_radix(&tag_colour[4..6], 16)?;
            let tag_id = app.task_store.tags.keys().last().map_or(0, |id| *id + 1);
            app.task_store.tags.insert(
                tag_id,
                crate::task::Tag {
                    // Unfortunately the `.to_owned` call is required as this is a Fn rather than a
                    // FnOnce
                    name: tag_name.to_owned(),
                    colour: Color::Rgb(red, green, blue),
                },
            );
            app.task_store.tasks[selected_index].flip_tag(tag_id);
            Ok(())
        },
        move |app, err| {
            open_select_tag_colour(app, selected_index, tag.to_owned());
            app.append_layer(MessageBox::new(
                String::from("Error"),
                err.to_string(),
                tui::style::Color::Red,
            ));
        },
    ));
}
