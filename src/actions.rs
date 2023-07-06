use std::{cell::RefCell, rc::Rc};

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::style::Color;

use crate::{
    app::{App, Mode},
    component::{
        input::dialog::DialogAction,
        input::{dialog::DialogBoxBuilder, input_box::InputBoxBuilder},
        message_box::MessageBox,
    },
    error::AppError,
    task::CompletedTask,
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
            app.mode = Mode::CurrentTasks;
        }),
        DialogAction::new(
            String::from("2    Change to completed task window"),
            |app| {
                app.mode = Mode::CompletedTasks;
            },
        ),
    ];
    for ac in app.mode.available_help_actions() {
        actions.push(DialogAction::new(
            format!("{}    {}", ac.short_hand, ac.description),
            move |app| app.execute_event(KeyEvent::new(ac.character, KeyModifiers::NONE)),
        ));
    }
    let help_menu = DialogBoxBuilder::default()
        .title(String::from("Help Menu"))
        .options(actions)
        .save_mode(app)
        .build();

    app.push_layer(help_menu);
}

pub fn open_delete_task_menu(app: &mut App, selected_index: Rc<RefCell<usize>>) {
    if app.task_store.tasks.is_empty() {
        return;
    }
    let delete_dialog = DialogBoxBuilder::default()
        .title("Delete selected task".to_string())
        .add_option(DialogAction::new(String::from("Delete"), move |app| {
            let mut selected_index = selected_index.borrow_mut();
            app.task_store.tasks.remove(*selected_index);
            if *selected_index == app.task_store.tasks.len() && !app.task_store.tasks.is_empty() {
                *selected_index -= 1;
            }
        }))
        .add_option(DialogAction::new(String::from("Cancel"), |_| {}))
        .save_mode(app)
        .build();
    app.push_layer(delete_dialog);
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

pub fn flip_tag_menu(app: &mut App, selected_index: usize) {
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

    tag_options.push(DialogAction::new(
        String::from("Clear all tags"),
        move |app| {
            app.task_store.tasks[selected_index].tags.clear();
        },
    ));
    tag_options.push(DialogAction::new(String::from("Cancel"), |_| {}));

    let dialog = DialogBoxBuilder::default()
        .title("Add or remove a tag".to_string())
        .options(tag_options)
        .save_mode(app)
        .build();
    app.push_layer(dialog);
}

pub fn edit_tag_menu(app: &mut App, selected_index: usize) {
    let mut tag_options: Vec<DialogAction> = Vec::new();

    tag_options.push(DialogAction::new(String::from("New tag"), move |app| {
        let tag_menu = InputBoxBuilder::default()
            .title(String::from("Tag name"))
            .callback(move |app, tag_name| {
                open_select_tag_colour(app, selected_index, tag_name);
                Ok(())
            })
            .save_mode(app)
            .build();
        app.push_layer(tag_menu)
    }));
    tag_options.push(DialogAction::new(
        String::from("Delete a tag"),
        move |app| {
            delete_tag_menu(app);
        },
    ));
    tag_options.push(DialogAction::new(String::from("Cancel"), |_| {}));

    let dialog = DialogBoxBuilder::default()
        .title("Add or remove a tag".to_string())
        .options(tag_options)
        .save_mode(app)
        .build();
    app.push_layer(dialog);
}

pub fn delete_tag_menu(app: &mut App) {
    let mut tag_options: Vec<DialogAction> = Vec::new();

    for (i, tag) in app.task_store.tags.iter() {
        let moved: u32 = *i;
        let moved_name = tag.name.clone();
        // TODO: Allow for DialogBox to support colours.
        tag_options.push(DialogAction::new(String::from(&tag.name), move |app| {
            let tag_dialog = DialogBoxBuilder::default()
                .title(format!(
                    "Do you want to permenatly delete the tag {}",
                    moved_name
                ))
                .add_option(DialogAction::new(String::from("Delete"), move |app| {
                    app.task_store.delete_tag(moved)
                }))
                .add_option(DialogAction::new(String::from("Cancel"), move |_| {}))
                .save_mode(app)
                .build();
            app.push_layer(tag_dialog);
        }));
    }
    tag_options.push(DialogAction::new(String::from("Cancel"), |_| {}));

    let delete_dialog = DialogBoxBuilder::default()
        .title("Delete a tag".to_string())
        .options(tag_options)
        .save_mode(app)
        .build();
    app.push_layer(delete_dialog);
}

fn open_select_tag_colour(app: &mut App, selected_index: usize, tag_name: String) {
    let tag = tag_name.clone();
    let colour_menu = InputBoxBuilder::default()
        .title(String::from("Tag colour"))
        .callback(move |app, tag_colour| {
            let colour = if tag_colour.starts_with('#') {
                let red = u8::from_str_radix(&tag_colour[1..3], 16)?;
                let green = u8::from_str_radix(&tag_colour[3..5], 16)?;
                let blue = u8::from_str_radix(&tag_colour[5..7], 16)?;
                Color::Rgb(red, green, blue)
            } else if let Ok(colour) = tag_colour.parse() {
                Color::Indexed(colour)
            } else {
                match tag_colour
                    .to_lowercase()
                    .replace([' ', '_', '-'], "")
                    .as_str()
                {
                    "reset" => Color::Reset,
                    "black" => Color::Black,
                    "red" => Color::Red,
                    "green" => Color::Green,
                    "yellow" => Color::Yellow,
                    "blue" => Color::Blue,
                    "magenta" => Color::Magenta,
                    "cyan" => Color::Cyan,
                    "gray" => Color::Gray,
                    "darkgray" => Color::DarkGray,
                    "lightred" => Color::LightRed,
                    "lightgreen" => Color::LightGreen,
                    "lightyellow" => Color::LightYellow,
                    "lightblue" => Color::LightBlue,
                    "lightmagenta" => Color::LightMagenta,
                    "lightcyan" => Color::LightCyan,
                    "white" => Color::White,
                    _ => return Err(AppError::InvalidColour()),
                }
            };

            let tag_id = app.task_store.tags.keys().last().map_or(0, |id| *id + 1);
            app.task_store.tags.insert(
                tag_id,
                crate::task::Tag {
                    name: tag_name,
                    colour,
                },
            );
            app.task_store.tasks[selected_index].flip_tag(tag_id);
            Ok(())
        })
        .error_callback(move |app, err| {
            let tag_name = tag.clone();
            let message_box = MessageBox::new(
                String::from("Error"),
                move |app| {
                    open_select_tag_colour(app, selected_index, tag_name);
                },
                err.to_string(),
                tui::style::Color::Red,
                0,
            )
            .save_mode(app);
            app.push_layer(message_box);
        })
        .save_mode(app)
        .build();
    app.push_layer(colour_menu);
}
