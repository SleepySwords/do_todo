use chrono::Local;
use crossterm::event::KeyEvent;
use tui::style::{Color, Style};

use crate::{
    app::{App, Mode},
    component::{
        message_box::MessageBox,
        overlay::{dialog::DialogAction, Overlay},
        overlay::{dialog::DialogBoxBuilder, fuzzy::FuzzyBoxBuilder, input_box::InputBoxBuilder},
    },
    draw::PostEvent,
    error::AppError,
    input,
    key::Key,
    task::CompletedTask,
};

// Action class maybe?!!
pub struct HelpEntry<'a> {
    character: Key,
    short_hand: String,
    description: &'a str,
}

impl HelpEntry<'_> {
    pub fn new(character: Key, description: &str) -> HelpEntry<'_> {
        HelpEntry {
            character,
            short_hand: character.to_string(),
            description,
        }
    }
    pub fn new_multiple(character: [Key; 2], description: &str) -> HelpEntry<'_> {
        HelpEntry {
            character: character[0],
            short_hand: itertools::intersperse(
                character.iter().map(|f| f.to_string()),
                " ".to_string(),
            )
            .collect::<String>(),
            description,
        }
    }
}

impl App {
    fn create_dialog_or_fuzzy(
        &mut self,
        title: &str,
        options: Vec<DialogAction<'static>>,
    ) -> PostEvent {
        if self.config.use_fuzzy {
            let fuzzy = FuzzyBoxBuilder::default()
                .title(title.to_string())
                .options(options)
                .save_mode(self)
                .build();
            PostEvent::push_layer(false, fuzzy)
        } else {
            let dialog = DialogBoxBuilder::default()
                .title(title.to_string())
                .options(options)
                .save_mode(self)
                .build();
            PostEvent::push_layer(false, dialog)
        }
    }

    pub fn create_help_menu(&mut self) -> PostEvent {
        // Actions that are universal, should use a table?
        let mut actions: Vec<DialogAction> = vec![
            DialogAction::new(
                format!(
                    "{: <15}Change to current task window",
                    self.config.tasks_menu_key.to_string()
                ),
                |app| {
                    app.mode = Mode::CurrentTasks;
                    PostEvent::noop(false)
                },
            ),
            DialogAction::new(
                format!(
                    "{: <15}Change to completed task window",
                    self.config.completed_tasks_menu_key.to_string()
                ),
                |app| {
                    app.mode = Mode::CompletedTasks;
                    PostEvent::noop(false)
                },
            ),
        ];
        for ac in self.mode.help_entries(&self.config) {
            actions.push(DialogAction::new(
                format!("{: <15}{}", ac.short_hand, ac.description),
                move |app| {
                    // FIXME: oh well
                    let result = input::help_input(
                        app,
                        KeyEvent::new(ac.character.code, ac.character.modifiers),
                    );
                    if let Err(AppError::InvalidState(msg)) = result {
                        let prev_mode = app.mode;
                        app.mode = Mode::Overlay;
                        return PostEvent::push_layer(
                            false,
                            Overlay::Message(MessageBox::new(
                                "An error occured".to_string(),
                                move |app| app.mode = prev_mode,
                                msg,
                                Color::Red,
                                0,
                            )),
                        );
                    } else {
                        PostEvent::noop(false)
                    }
                },
            ));
        }

        self.create_dialog_or_fuzzy("Help menu", actions)
    }

    pub fn create_delete_selected_task_menu(&mut self) -> PostEvent {
        if self.task_store.tasks.is_empty() {
            return PostEvent::noop(false);
        }
        let delete_dialog = DialogBoxBuilder::default()
            .title("Delete selected task".to_string())
            .add_option(DialogAction::new(String::from("Delete"), move |app| {
                let selected_index = &mut app.task_list.selected_index;
                app.task_store.delete_task(*selected_index);
                if *selected_index == app.task_store.find_tasks_draw_size()
                    && !app.task_store.tasks.is_empty()
                {
                    *selected_index -= 1;
                }
                PostEvent::noop(false)
            }))
            .add_option(DialogAction::new(String::from("Cancel"), |_| {
                PostEvent::noop(false)
            }))
            .save_mode(self)
            .build();
        PostEvent::push_layer(false, delete_dialog)
    }

    pub fn complete_selected_task(&mut self) {
        if self.task_store.tasks.is_empty() {
            return;
        }
        let selected_index = &mut self.task_list.selected_index;
        let local = Local::now();
        let time_completed = local.naive_local();
        let Some(task) = self.task_store.delete_task(*selected_index) else {
            return;
        };
        self.task_store
            .completed_tasks
            .push(CompletedTask::from_task(task, time_completed));
        if *selected_index == self.task_store.find_tasks_draw_size()
            && !self.task_store.tasks.is_empty()
        {
            *selected_index -= 1;
        }
    }

    pub fn create_tag_menu(&mut self) -> PostEvent {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        let selected_index = self.task_list.selected_index;

        if !self.task_store.tasks.is_empty() && self.mode == Mode::CurrentTasks {
            // Loops through the tags and adds them to the menu.
            for (i, tag) in self.task_store.tags.iter() {
                let moved: u32 = *i;
                // TODO: Allow for DialogBox to support colours.
                tag_options.push(DialogAction::styled(
                    String::from(&tag.name),
                    Style::default().fg(tag.colour),
                    move |app| {
                        if let Some(task) = app.task_store.task_mut(selected_index) {
                            task.flip_tag(moved);
                        };
                        PostEvent::noop(false)
                    },
                ));
            }
        }

        tag_options.push(DialogAction::new(String::from("New tag"), move |app| {
            let tag_menu = InputBoxBuilder::default()
                .title(String::from("Tag name"))
                .callback(move |app, tag_name| {
                    Ok(app.open_select_tag_colour(selected_index, tag_name))
                })
                .save_mode(app)
                .build_overlay();
            PostEvent::push_layer(false, tag_menu)
        }));

        if !self.task_store.tasks.is_empty() && self.mode == Mode::CurrentTasks {
            tag_options.push(DialogAction::new(
                String::from("Clear all tags"),
                move |app| {
                    if let Some(task) = app.task_store.task_mut(selected_index) {
                        task.tags.clear();
                    };
                    PostEvent::noop(false)
                },
            ));
        }

        tag_options.push(DialogAction::new(
            String::from("Delete a tag (permanently)"),
            move |app| app.create_delete_tag_menu(),
        ));
        tag_options.push(DialogAction::new(String::from("Cancel"), |_| {
            PostEvent::noop(false)
        }));

        self.create_dialog_or_fuzzy("Add or remove a tag", tag_options)
    }

    pub fn create_delete_tag_menu(&mut self) -> PostEvent {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        for (i, tag) in self.task_store.tags.iter() {
            let moved: u32 = *i;
            let moved_name = tag.name.clone();
            // TODO: Allow for DialogBox to support colours.
            tag_options.push(DialogAction::styled(
                String::from(&tag.name),
                Style::default().fg(tag.colour),
                move |app| {
                    let tag_dialog = DialogBoxBuilder::default()
                        .title(format!(
                            "Do you want to permenatly delete the tag {}",
                            moved_name
                        ))
                        .add_option(DialogAction::new(String::from("Delete"), move |app| {
                            app.task_store.delete_tag(moved);
                            PostEvent::noop(false)
                        }))
                        .add_option(DialogAction::new(String::from("Cancel"), move |_| {
                            PostEvent::noop(false)
                        }))
                        .save_mode(app)
                        .build();
                    PostEvent::push_layer(false, tag_dialog)
                },
            ));
        }
        tag_options.push(DialogAction::new(String::from("Cancel"), |_| {
            PostEvent::noop(false)
        }));

        self.create_dialog_or_fuzzy("Delete a tag", tag_options)
    }

    fn open_select_tag_colour(&mut self, selected_index: usize, tag_name: String) -> PostEvent {
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
                        .parse::<Color>()
                    {
                        Ok(colour) => colour,
                        Err(_) => return Err(AppError::InvalidColour),
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
                if app.task_store.find_tasks_draw_size() > selected_index {
                    if let Some(task) = app.task_store.task_mut(selected_index) {
                        task.flip_tag(tag_id);
                    }
                }
                Ok(PostEvent::noop(false))
            })
            .error_callback(move |app, err| {
                let tag_name = tag.clone();
                let message_box = MessageBox::new(
                    String::from("Error"),
                    move |app| {
                        app.open_select_tag_colour(selected_index, tag_name);
                    },
                    err.to_string(),
                    tui::style::Color::Red,
                    0,
                )
                .save_mode(app);
                return PostEvent::push_layer(false, Overlay::Message(message_box));
            })
            .save_mode(self)
            .build_overlay();
        PostEvent::push_layer(false, colour_menu)
    }
}
