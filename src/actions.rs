use chrono::{Local, NaiveDate};
use crossterm::event::KeyEvent;
use itertools::Itertools;
use tui::style::{Color, Stylize};

use crate::{
    app::{App, Mode},
    component::{
        message_box::MessageBoxBuilder,
        overlay::{
            dialog::{DialogAction, DialogBoxBuilder},
            fuzzy::FuzzyBoxBuilder,
            input_box::InputBoxBuilder,
            vim::VimMode,
        },
    },
    error::AppError,
    framework::event::PostEvent,
    input,
    task::{FindParentResult, Task},
    utils::{self, str_to_colour},
};

// Universal functions
impl App {
    pub fn create_add_task_menu(&mut self) -> Result<PostEvent, AppError> {
        let add_input_dialog = InputBoxBuilder::default()
            .title("Add a task")
            .on_submit(move |app, word| {
                app.task_store
                    .add_task(Task::from_string(word.trim()), None);
                if app.mode == Mode::CurrentTasks {
                    app.task_list.selected_index = app.task_store.find_tasks_draw_size() - 1;
                }
                PostEvent::noop(false)
            })
            .use_vim(&self.config, VimMode::Insert)
            .build();
        Ok(PostEvent::push_layer(add_input_dialog))
    }

    pub fn go_to_task_list(&mut self) -> Result<PostEvent, AppError> {
        self.mode = Mode::CurrentTasks;
        Ok(PostEvent::noop(false))
    }

    pub fn go_to_completed_list(&mut self) -> Result<PostEvent, AppError> {
        self.mode = Mode::CompletedTasks;
        Ok(PostEvent::noop(false))
    }

    pub fn sort(&mut self) -> Result<PostEvent, AppError> {
        self.task_store.sort();
        Ok(PostEvent::noop(false))
    }

    pub fn enable_auto_sort(&mut self) -> Result<PostEvent, AppError> {
        self.task_list.auto_sort = !self.task_list.auto_sort;
        self.task_store.sort();
        Ok(PostEvent::noop(false))
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
                .title(title)
                .options(options)
                .build();
            PostEvent::push_layer(fuzzy)
        } else {
            let dialog = DialogBoxBuilder::default()
                .title(title)
                .options(options)
                .build();
            PostEvent::push_layer(dialog)
        }
    }

    pub fn create_help_menu(&mut self) -> Result<PostEvent, AppError> {
        // Actions that are universal, should use a table?
        let mut keys = input::universal_input_keys(&self.config);
        keys.append(&mut self.mode.help_entries(&self.config));
        let actions: Vec<DialogAction> = keys
            .into_iter()
            .map(|ac| {
                DialogAction::new(
                    format!("{: <15}{}", ac.short_hand, ac.description),
                    move |app| {
                        // HACK: This technically does not consider overlay,
                        // Should be fine, since they don't show up in Help
                        // Menus anyway
                        let result = input::help_input(
                            app,
                            KeyEvent::new(ac.character.code, ac.character.modifiers),
                        );

                        match result {
                            Ok(result) => result,
                            Err(AppError::InvalidState(msg)) => {
                                let prev_mode = app.mode; // FIXME: why is this here???
                                app.mode = Mode::Overlay;
                                let message = MessageBoxBuilder::default()
                                    .title("An error occured")
                                    .message(msg)
                                    .on_close(move |app| {
                                        app.mode = prev_mode;
                                        PostEvent::noop(false)
                                    })
                                    .colour(Color::Red);
                                PostEvent::push_layer(message.build())
                            }
                            _ => PostEvent::noop(false),
                        }
                    },
                )
            })
            .collect_vec();

        Ok(self.create_dialog_or_fuzzy("Help menu", actions))
    }

    pub fn create_delete_selected_task_menu(&mut self) -> Result<PostEvent, AppError> {
        if self.task_store.root_tasks().is_empty() {
            return Ok(PostEvent::noop(false));
        }
        let delete_dialog = DialogBoxBuilder::default()
            .title("Delete selected task")
            .add_option("Delete", move |app| {
                let selected_index = &mut app.task_list.selected_index;
                if let Some(task_to_delete) = app.task_store.global_pos_to_task(*selected_index) {
                    app.task_store.delete_task(&task_to_delete);
                }

                if *selected_index == app.task_store.find_tasks_draw_size()
                    && !app.task_store.root_tasks().is_empty()
                {
                    *selected_index -= 1;
                }
                PostEvent::noop(false)
            })
            .add_option("Cancel", |_| PostEvent::noop(false))
            .build();
        Ok(PostEvent::push_layer(delete_dialog))
    }

    pub fn complete_selected_task(&mut self) -> Result<PostEvent, AppError> {
        if self.task_store.root_tasks().is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let selected_index = &mut self.task_list.selected_index;
        let local = Local::now();
        let time_completed = local.naive_local();
        if let Some(completed_task) = self.task_store.global_pos_to_task(*selected_index) {
            self.task_store
                .complete_task(&completed_task, time_completed);
            if *selected_index == self.task_store.find_tasks_draw_size()
                && !self.task_store.root_tasks().is_empty()
            {
                *selected_index -= 1;
            }
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_tag_menu(&mut self) -> Result<PostEvent, AppError> {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        let selected_index = self.task_list.selected_index;
        let Some(task_id) = self.task_store.global_pos_to_task(selected_index) else {
            // FIXME: should probs error?
            return Ok(PostEvent::noop(true));
        };

        if !self.task_store.root_tasks().is_empty() && self.mode == Mode::CurrentTasks {
            // Loops through the tags and adds them to the menu.
            for (i, tag) in self.task_store.tags().iter() {
                let moved = i.to_string();
                let moved_name = task_id.clone();
                tag_options.push(DialogAction::new(
                    tag.name.to_owned().fg(tag.colour),
                    move |app| {
                        if let Some(task) = app.task_store.task_mut(&moved_name) {
                            task.flip_tag(moved);
                        };
                        PostEvent::noop(false)
                    },
                ));
            }
        }

        let new_task = task_id.clone();
        tag_options.push(DialogAction::new(String::from("New tag"), move |app| {
            let tag_menu = InputBoxBuilder::default()
                .title("Tag name")
                .on_submit(move |app, tag_name| {
                    let new_task = new_task.clone();
                    app.create_select_tag_colour("".to_string(), move |app, tag_colour| {
                        let colour = str_to_colour(&tag_colour)?;

                        let tag_id = app.task_store.tags().keys().len().to_string();
                        app.task_store.tags_mut().insert(
                            tag_id.clone(),
                            crate::task::Tag {
                                name: tag_name.clone(),
                                colour,
                            },
                        );
                        if app.task_store.find_tasks_draw_size() > selected_index {
                            if let Some(task) = app.task_store.task_mut(&new_task) {
                                task.flip_tag(tag_id);
                            }
                        }
                        Ok(PostEvent::noop(false))
                    })
                })
                .use_vim(&app.config, VimMode::Insert)
                .build();
            PostEvent::push_layer(tag_menu)
        }));

        if !self.task_store.root_tasks().is_empty() && self.mode == Mode::CurrentTasks {
            tag_options.push(DialogAction::new(
                String::from("Clear all tags"),
                move |app| {
                    if let Some(task) = app.task_store.task_mut(&task_id) {
                        task.tags.clear();
                    };
                    PostEvent::noop(false)
                },
            ));
        }

        tag_options.push(DialogAction::new(String::from("Edit a tag"), move |app| {
            app.create_edit_tag_menu()
        }));
        tag_options.push(DialogAction::new(
            String::from("Delete a tag (permanently)"),
            move |app| app.create_delete_tag_menu(),
        ));
        tag_options.push(DialogAction::new(String::from("Cancel"), |_| {
            PostEvent::noop(false)
        }));

        Ok(self.create_dialog_or_fuzzy("Add or remove a tag", tag_options))
    }

    pub fn create_edit_tag_menu(&mut self) -> PostEvent {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        for (i, tag) in self.task_store.tags().iter() {
            let tag_id = i.to_string();
            let tag_name = tag.name.clone();
            let tag_colour = tag.colour;
            tag_options.push(DialogAction::new(
                tag.name.to_owned().fg(tag.colour),
                move |_app| {
                    let edit_name = InputBoxBuilder::default()
                        .title("Edit tag name")
                        .fill(&tag_name)
                        .use_vim(&_app.config, VimMode::Normal)
                        .on_submit(move |app, tag_name| {
                            let tag_id = tag_id.to_string();
                            app.create_select_tag_colour(
                                tag_colour.to_string(),
                                move |app, tag_colour| {
                                    let colour = utils::str_to_colour(&tag_colour)?;
                                    app.task_store.tags_mut().insert(
                                        tag_id.to_string(),
                                        crate::task::Tag {
                                            name: tag_name.clone(),
                                            colour,
                                        },
                                    );
                                    Ok(PostEvent::noop(false))
                                },
                            )
                        });
                    PostEvent::push_layer(edit_name.build())
                },
            ));
        }
        tag_options.push(DialogAction::new("Cancel", |_| PostEvent::noop(false)));

        self.create_dialog_or_fuzzy("Edit a tag", tag_options)
    }

    fn create_select_tag_colour<T>(&mut self, default_string: String, callback: T) -> PostEvent
    where
        T: Fn(&mut App, String) -> Result<PostEvent, AppError> + Clone + 'static,
    {
        let tag_colour = InputBoxBuilder::default()
            .fill(&default_string.clone())
            .title("Select a colour")
            .use_vim(
                &self.config,
                if default_string.is_empty() {
                    VimMode::Insert
                } else {
                    VimMode::Normal
                },
            )
            .on_submit(move |app, input| {
                let result = callback(app, input);
                let str = default_string.clone();
                let cloned_callback = callback.clone();
                match result {
                    Ok(r) => r,
                    Err(err) => {
                        let message_box = MessageBoxBuilder::default()
                            .title("Error")
                            .on_close(move |app| {
                                app.create_select_tag_colour(str.clone(), cloned_callback.clone())
                            })
                            .colour(Color::Red)
                            .message(err.to_string())
                            .build();
                        PostEvent::push_layer(message_box)
                    }
                }
            });

        PostEvent::push_layer(tag_colour.build())
    }

    pub fn create_delete_tag_menu(&mut self) -> PostEvent {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        for (i, tag) in self.task_store.tags().iter() {
            let tag_id = i.to_string();
            let tag_name = tag.name.clone();
            tag_options.push(DialogAction::new(
                tag.name.to_owned().fg(tag.colour),
                move |_app| {
                    let tag_dialog = DialogBoxBuilder::default()
                        .title(format!(
                            "Do you want to permenatly delete the tag {}",
                            tag_name
                        ))
                        .add_option("Delete", move |app| {
                            app.task_store.delete_tag(&tag_id);
                            PostEvent::noop(false)
                        })
                        .add_option("Cancel", move |_| PostEvent::noop(false))
                        .build();
                    PostEvent::push_layer(tag_dialog)
                },
            ));
        }
        tag_options.push(DialogAction::new("Cancel", |_| PostEvent::noop(false)));

        self.create_dialog_or_fuzzy("Delete a tag", tag_options)
    }

    pub fn cycle_priority(&mut self) -> Result<PostEvent, AppError> {
        if self.task_store.root_tasks().is_empty() {
            return Ok(PostEvent::noop(true));
        }

        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            return Ok(PostEvent::noop(true));
        };
        let Some(task) = self.task_store.task_mut(&task_id) else {
            return Ok(PostEvent::noop(true));
        };
        task.priority = task.priority.next_priority();

        if self.task_list.auto_sort {
            self.task_store.sort();
            let Some(new_pos) = self.task_store.task_to_global_pos(&task_id) else {
                return Ok(PostEvent::noop(false));
            };
            self.task_list.selected_index = new_pos;
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_add_subtask_menu(&mut self) -> Result<PostEvent, AppError> {
        let index = self.task_list.selected_index;
        let Some(task_id) = self.task_store.global_pos_to_task(index) else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };
        let Some(task) = self.task_store.task_mut(&task_id) else {
            return Ok(PostEvent::noop(false));
        };
        let add_input_dialog = InputBoxBuilder::default()
            .title(format!("Add a subtask to {}", task.title))
            .use_vim(&self.config, VimMode::Insert)
            .on_submit(move |app, word| {
                app.task_store
                    .add_task(Task::from_string(word.trim()), Some(&task_id));
                if let Some(task) = app.task_store.task_mut(&task_id) {
                    task.opened = true;
                    app.task_list.selected_index +=
                        app.task_store.subtasks(&task_id).map_or(0, |f| f.len());
                }
                PostEvent::noop(false)
            });
        Ok(PostEvent::push_layer(add_input_dialog.build()))
    }

    pub fn move_selected_task_down(&mut self) -> Result<PostEvent, AppError> {
        let autosort = self.task_list.auto_sort;

        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            todo!()
        };

        let Some(FindParentResult {
            parent_id: parent_index,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(&task_id)
        else {
            return Ok(PostEvent::noop(true));
        };

        let parent_subtasks = if let Some(parent) = &parent_index {
            self.task_store.subtasks(parent).unwrap()
        } else {
            self.task_store.root_tasks()
        };

        let new_index = (local_index + 1) % parent_subtasks.len();

        let task = &parent_subtasks[local_index];
        let task_above = &parent_subtasks[new_index];

        if self.task_store.task(task).unwrap().priority
            == self.task_store.task(task_above).unwrap().priority
            || !autosort
        {
            self.task_store.move_task(&task_id, None, new_index, None);
            self.task_list.selected_index = self.task_store.task_to_global_pos(&task_id).unwrap();
        }

        Ok(PostEvent::noop(false))
    }

    pub fn move_selected_task_up(&mut self) -> Result<PostEvent, AppError> {
        let autosort = self.task_list.auto_sort;

        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            return Err(AppError::invalid_state("Could not find task to move."));
        };

        let Some(FindParentResult {
            parent_id,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(&task_id)
        else {
            return Ok(PostEvent::noop(true));
        };

        let parent_subtasks = if let Some(parent) = &parent_id {
            self.task_store
                .subtasks(parent)
                .ok_or_else(|| AppError::invalid_state("Subtasks are not found in parent"))?
        } else {
            self.task_store.root_tasks()
        };

        let new_index =
            (local_index as isize - 1).rem_euclid(parent_subtasks.len() as isize) as usize;

        let task = self
            .task_store
            .task(&parent_subtasks[local_index])
            .ok_or_else(|| AppError::invalid_state("Subtasks are not found in parent"))?;
        let task_above = self
            .task_store
            .task(&parent_subtasks[new_index])
            .ok_or_else(|| AppError::invalid_state("Subtasks are not found in parent"))?;

        if task.priority == task_above.priority || !autosort {
            self.task_store.move_task(&task_id, None, new_index, None);
            self.task_list.selected_index = self
                .task_store
                .task_to_global_pos(&task_id)
                .ok_or_else(|| AppError::invalid_state("Did not find global position of task"))?;
        }

        Ok(PostEvent::noop(false))
    }

    pub fn create_edit_selected_task_menu(&mut self) -> Result<PostEvent, AppError> {
        let index = self.task_list.selected_index;

        let Some(task_id) = self.task_store.global_pos_to_task(index) else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };
        let Some(task) = self.task_store.task(&task_id) else {
            return Ok(PostEvent::noop(true));
        };
        let edit_box = InputBoxBuilder::default()
            .title("Edit the selected task")
            .use_vim(&self.config, VimMode::Normal)
            .fill(task.title.as_str())
            .on_submit(move |app, word| {
                let Some(task) = app.task_store.task_mut(&task_id) else {
                    return PostEvent::noop(false);
                };
                task.title = word.trim().to_string();
                PostEvent::noop(false)
            })
            .use_vim(&self.config, VimMode::Normal)
            .build();
        Ok(PostEvent::push_layer(edit_box))
    }

    pub fn flip_selected_progress(&mut self) -> Result<PostEvent, AppError> {
        if self.task_store.root_tasks().is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };
        let Some(task) = self.task_store.task_mut(&task_id) else {
            return Ok(PostEvent::noop(true));
        };
        task.progress = !task.progress;
        Ok(PostEvent::noop(false))
    }

    pub fn flip_subtasks(&mut self) -> Result<PostEvent, AppError> {
        let _selected_index = &mut self.task_list.selected_index;
        if self.task_store.root_tasks().is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };
        let Some(task) = self.task_store.task_mut(&task_id) else {
            return Ok(PostEvent::noop(true));
        };
        task.opened = !task.opened;
        Ok(PostEvent::noop(false))
    }

    pub fn move_subtask_level_up(&mut self) -> Result<PostEvent, AppError> {
        let selected_index = &mut self.task_list.selected_index;

        let Some(task_id) = self.task_store.global_pos_to_task(*selected_index) else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };

        // FIXME: this should also probs return subtasks?
        let Some(FindParentResult {
            parent_id,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(&task_id)
        else {
            return Ok(PostEvent::noop(true));
        };

        // FIXME: should be refactored into a singular subtasks thing.
        let subtasks = if let Some(parent_id) = parent_id {
            self.task_store
                .subtasks(&parent_id)
                .ok_or_else(|| AppError::invalid_state("Parent does not have subtasks"))?
        } else {
            self.task_store.root_tasks()
        };

        if subtasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }

        if local_index == 0 {
            return Ok(PostEvent::noop(true));
        }

        let prev_local_index = local_index - 1;
        let prev_task_id = subtasks[prev_local_index].to_string();
        let order = self
            .task_store
            .subtasks(&prev_task_id)
            .map_or(0, |sub| sub.len());

        self.task_store
            .move_task(&task_id, Some(prev_task_id.to_string()), order, None);

        let Some(prev_task) = self.task_store.task_mut(&prev_task_id) else {
            return Ok(PostEvent::noop(false));
        };
        if !prev_task.opened {
            prev_task.opened = true;
            // Have to remove the task when adding
            *selected_index += self.task_store.find_task_draw_size(&prev_task_id)
                - self.task_store.find_task_draw_size(&task_id)
                - 1;
        }

        if self.task_list.auto_sort {
            self.task_store.sort();
            if let Some(task_pos) = self.task_store.task_to_global_pos(&task_id) {
                *selected_index = task_pos;
            }
        }
        Ok(PostEvent::noop(false))
    }

    pub fn move_subtask_level_down(&mut self) -> Result<PostEvent, AppError> {
        let selected_index = &mut self.task_list.selected_index;
        let Some(task_id) = self.task_store.global_pos_to_task(*selected_index) else {
            return Err(AppError::invalid_state("Task does not exist"));
        };
        let Some(FindParentResult { parent_id, .. }) = self.task_store.find_parent(&task_id) else {
            return Ok(PostEvent::noop(true));
        };

        let Some(parent_id) = parent_id else {
            return Ok(PostEvent::noop(true));
        };

        // let Some(task) = self.task_store.delete_task(*selected_index) else {
        //     return Ok(PostEvent::noop(true));
        // };

        let Some(FindParentResult {
            parent_id: grand_parent_id,
            task_local_offset: parent_local_index,
            ..
        }) = self.task_store.find_parent(&parent_id)
        else {
            return Ok(PostEvent::noop(true));
        };

        if let Some(grand_parent_id) = grand_parent_id {
            self.task_store.move_task(
                &task_id,
                Some(grand_parent_id),
                parent_local_index + 1,
                None,
            );
        } else {
            self.task_store
                .move_task(&task_id, None, parent_local_index + 1, Some(()));
        }

        if self.task_list.auto_sort {
            self.task_store.sort();
            if let Some(task_pos) = self.task_store.task_to_global_pos(&task_id) {
                *selected_index = task_pos;
            }
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_due_date_dialog(&mut self) -> Result<PostEvent, AppError> {
        let Some(task_id) = self
            .task_store
            .global_pos_to_task(self.task_list.selected_index)
        else {
            // FIXME: panic!
            return Ok(PostEvent::noop(true));
        };
        let date_dialog = InputBoxBuilder::default()
            .title("Add date or specify \"none\" to remove".to_string())
            .on_submit(move |app, date_str| {
                if date_str.to_lowercase() == "none" {
                    if let Some(task) = app.task_store.task_mut(&task_id) {
                        task.due_date = None;
                    }
                    return PostEvent::noop(false);
                }
                let date = NaiveDate::parse_from_str(&date_str, "%d/%m/%y")
                    .or_else(|_| NaiveDate::parse_from_str(&date_str, "%d/%m/%Y"))
                    .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"));

                if let Some(task) = app.task_store.task_mut(&task_id) {
                    match date {
                        Ok(due) => {
                            task.due_date = Some(due);
                        }
                        Err(err) => {
                            let error_message = MessageBoxBuilder::default()
                                .title("An error occured")
                                .message(format!("Could not parse the date: {}", err))
                                .colour(Color::Red)
                                .on_close(|app| {
                                    app.create_due_date_dialog()
                                        .expect("Should always be ok...")
                                })
                                .build();
                            return PostEvent::push_layer(error_message);
                        }
                    }
                }
                PostEvent::noop(false)
            })
            .use_vim(&self.config, VimMode::Insert)
            .build();
        Ok(PostEvent::push_layer(date_dialog))
    }
}
