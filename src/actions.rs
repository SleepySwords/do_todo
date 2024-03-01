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
    task::{CompletedTask, FindParentResult, Task, TaskStore},
    utils::{self, str_to_colour},
};

// Universal functions
impl App {
    pub fn create_add_task_menu(&mut self) -> Result<PostEvent, AppError> {
        let add_input_dialog = InputBoxBuilder::default()
            .title("Add a task")
            .on_submit(move |app, word| {
                app.task_store.add_task(Task::from_string(word.trim()));
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
        self.task_store.auto_sort = !self.task_store.auto_sort;
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
        if self.task_store.tasks.is_empty() {
            return Ok(PostEvent::noop(false));
        }
        let delete_dialog = DialogBoxBuilder::default()
            .title("Delete selected task")
            .add_option("Delete", move |app| {
                let selected_index = &mut app.task_list.selected_index;
                app.task_store.delete_task(*selected_index);

                if *selected_index == app.task_store.find_tasks_draw_size()
                    && !app.task_store.tasks.is_empty()
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
        if self.task_store.tasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let selected_index = &mut self.task_list.selected_index;
        let local = Local::now();
        let time_completed = local.naive_local();
        let Some(task) = self.task_store.delete_task(*selected_index) else {
            return Ok(PostEvent::noop(true));
        };
        self.task_store
            .completed_tasks
            .push(CompletedTask::from_task(task, time_completed));
        if *selected_index == self.task_store.find_tasks_draw_size()
            && !self.task_store.tasks.is_empty()
        {
            *selected_index -= 1;
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_tag_menu(&mut self) -> Result<PostEvent, AppError> {
        let mut tag_options: Vec<DialogAction> = Vec::new();

        let selected_index = self.task_list.selected_index;

        if !self.task_store.tasks.is_empty() && self.mode == Mode::CurrentTasks {
            // Loops through the tags and adds them to the menu.
            for (i, tag) in self.task_store.tags.iter() {
                let moved: usize = *i;
                tag_options.push(DialogAction::new(
                    tag.name.to_owned().fg(tag.colour),
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
                .title("Tag name")
                .on_submit(move |app, tag_name| {
                    app.create_select_tag_colour("".to_string(), move |app, tag_colour| {
                        let colour = str_to_colour(&tag_colour)?;

                        let tag_id = app.task_store.tags.keys().last().map_or(0, |id| *id + 1);
                        app.task_store.tags.insert(
                            tag_id,
                            crate::task::Tag {
                                name: tag_name.clone(),
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
                })
                .use_vim(&app.config, VimMode::Insert)
                .build();
            PostEvent::push_layer(tag_menu)
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

        for (i, tag) in self.task_store.tags.iter() {
            let tag_id: usize = *i;
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
                            app.create_select_tag_colour(
                                tag_colour.to_string(),
                                move |app, tag_colour| {
                                    let colour = utils::str_to_colour(&tag_colour)?;
                                    app.task_store.tags.insert(
                                        tag_id,
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

    fn create_select_tag_colour<T: 'static>(
        &mut self,
        default_string: String,
        callback: T,
    ) -> PostEvent
    where
        T: Fn(&mut App, String) -> Result<PostEvent, AppError> + Clone,
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

        for (i, tag) in self.task_store.tags.iter() {
            let tag_id: usize = *i;
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
                            app.task_store.delete_tag(tag_id);
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
        if self.task_store.tasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }

        let old_task = {
            let Some(task) = self.task_store.task_mut(self.task_list.selected_index) else {
                return Ok(PostEvent::noop(true));
            };
            task.priority = task.priority.next_priority();

            task.clone()
        };

        if self.task_store.auto_sort {
            self.task_store.sort();
        }

        self.task_list.selected_index =
            self.task_store.task_position(&old_task).ok_or_else(|| {
                AppError::InvalidState("Cannot find the selected tasks index.".to_string())
            })?;
        Ok(PostEvent::noop(false))
    }

    pub fn create_add_subtask_menu(&mut self) -> Result<PostEvent, AppError> {
        let index = self.task_list.selected_index;
        let Some(task) = self.task_store.task_mut(index) else {
            return Ok(PostEvent::noop(false));
        };
        let add_input_dialog = InputBoxBuilder::default()
            .title(format!("Add a subtask to {}", task.title))
            .use_vim(&self.config, VimMode::Insert)
            .on_submit(move |app, word| {
                if let Some(task) = app.task_store.task_mut(index) {
                    task.sub_tasks.push(Task::from_string(word.trim()));
                    task.opened = true;
                    app.task_list.selected_index += task.sub_tasks.len();
                }
                PostEvent::noop(false)
            })
            .build();
        Ok(PostEvent::push_layer(add_input_dialog))
    }

    pub fn move_selected_task_down(&mut self) -> Result<PostEvent, AppError> {
        let autosort = self.task_store.auto_sort;

        let Some(FindParentResult {
            tasks: parent_tasks,
            parent_index,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(self.task_list.selected_index)
        else {
            return Ok(PostEvent::noop(true));
        };

        let new_index = (local_index + 1) % parent_tasks.len();

        let Some(parent_subtasks) = self.task_store.subtasks(parent_index) else {
            return Ok(PostEvent::noop(true));
        };

        let task = &parent_subtasks[local_index];
        let task_above = &parent_subtasks[new_index];

        if task.priority == task_above.priority || !autosort {
            let task = parent_subtasks.remove(local_index);

            parent_subtasks.insert(new_index, task);

            self.task_list.selected_index =
                TaskStore::local_index_to_global(new_index, parent_subtasks, parent_index);
        }
        Ok(PostEvent::noop(false))
    }

    pub fn move_selected_task_up(&mut self) -> Result<PostEvent, AppError> {
        let auto_sort = self.task_store.auto_sort;

        let Some(FindParentResult {
            tasks: parent_subtasks,
            parent_index,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(self.task_list.selected_index)
        else {
            return Ok(PostEvent::noop(true));
        };

        if parent_subtasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }

        let new_index =
            (local_index as isize - 1).rem_euclid(parent_subtasks.len() as isize) as usize;

        let Some(mut_parent_subtasks) = self.task_store.subtasks(parent_index) else {
            return Ok(PostEvent::noop(true));
        };

        let task = &mut_parent_subtasks[local_index];
        let task_above = &mut_parent_subtasks[new_index];

        if task.priority == task_above.priority || !auto_sort {
            let task = mut_parent_subtasks.remove(local_index);

            mut_parent_subtasks.insert(new_index, task);

            self.task_list.selected_index =
                TaskStore::local_index_to_global(new_index, mut_parent_subtasks, parent_index);
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_edit_selected_task_menu(&mut self) -> Result<PostEvent, AppError> {
        let index = self.task_list.selected_index;
        let Some(task) = self.task_store.task(index) else {
            return Ok(PostEvent::noop(true));
        };
        let edit_box = InputBoxBuilder::default()
            .title("Edit the selected task")
            .use_vim(&self.config, VimMode::Normal)
            .fill(task.title.as_str())
            .on_submit(move |app, word| {
                let Some(task) = app.task_store.task_mut(index) else {
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
        if self.task_store.tasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let Some(task) = self.task_store.task_mut(self.task_list.selected_index) else {
            return Ok(PostEvent::noop(true));
        };
        task.progress = !task.progress;
        Ok(PostEvent::noop(false))
    }

    pub fn flip_subtasks(&mut self) -> Result<PostEvent, AppError> {
        let selected_index = &mut self.task_list.selected_index;
        if self.task_store.tasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }
        let Some(task) = self.task_store.task_mut(*selected_index) else {
            return Ok(PostEvent::noop(true));
        };
        task.opened = !task.opened;
        Ok(PostEvent::noop(false))
    }

    pub fn move_subtask_level_up(&mut self) -> Result<PostEvent, AppError> {
        let selected_index = &mut self.task_list.selected_index;
        let Some(FindParentResult {
            tasks: parent_tasks,
            parent_index,
            task_local_offset: local_index,
        }) = self.task_store.find_parent(*selected_index)
        else {
            return Ok(PostEvent::noop(true));
        };

        if parent_tasks.is_empty() {
            return Ok(PostEvent::noop(true));
        }

        if local_index == 0 {
            return Ok(PostEvent::noop(true));
        }

        let prev_local_index = local_index - 1;
        let prev_global_index =
            TaskStore::local_index_to_global(prev_local_index, parent_tasks, parent_index);

        let Some(task) = self.task_store.delete_task(*selected_index) else {
            return Ok(PostEvent::noop(true));
        };

        let Some(prev_task) = self.task_store.task_mut(prev_global_index) else {
            return Ok(PostEvent::noop(true));
        };

        if !prev_task.opened {
            prev_task.opened = true;
            // Have to remove the task when adding
            *selected_index += prev_task.find_task_draw_size() - 1;
        }
        prev_task.sub_tasks.push(task);

        // FIXME: refactor this to ideally not clone
        if self.task_store.auto_sort {
            let Some(task) = self.task_store.task(*selected_index).cloned() else {
                return Err(AppError::InvalidState(
                    "There is no task selected.".to_string(),
                ));
            };
            self.task_store.sort();
            if let Some(task_pos) = self.task_store.task_position(&task) {
                *selected_index = task_pos;
            }
        }
        Ok(PostEvent::noop(false))
    }

    pub fn move_subtask_level_down(&mut self) -> Result<PostEvent, AppError> {
        let selected_index = &mut self.task_list.selected_index;
        let Some(FindParentResult { parent_index, .. }) =
            self.task_store.find_parent(*selected_index)
        else {
            return Ok(PostEvent::noop(true));
        };

        let Some(parent_index) = parent_index else {
            return Ok(PostEvent::noop(true));
        };

        let Some(task) = self.task_store.delete_task(*selected_index) else {
            return Ok(PostEvent::noop(true));
        };

        let Some(FindParentResult {
            tasks: grandparent_subtasks,
            parent_index: grandparent_index,
            ..
        }) = self.task_store.find_parent(parent_index)
        else {
            return Ok(PostEvent::noop(true));
        };

        let parent_local_index = grandparent_subtasks
            .iter()
            .position(|t| {
                t == self
                    .task_store
                    .task(parent_index)
                    .expect("This is definately a task")
            })
            .ok_or_else(|| {
                AppError::InvalidState("Cannot find the parent tasks index.".to_string())
            })?;

        let Some(grandparent_subtasks) = self.task_store.subtasks(grandparent_index) else {
            return Ok(PostEvent::noop(true));
        };

        let new_index = parent_local_index + 1;
        grandparent_subtasks.insert(new_index, task);
        *selected_index =
            TaskStore::local_index_to_global(new_index, grandparent_subtasks, grandparent_index);
        // FIXME: refactor this to ideally not clone
        if self.task_store.auto_sort {
            let Some(task) = self.task_store.task(*selected_index).cloned() else {
                return Err(AppError::InvalidState(
                    "Invalid selected index for this task.".to_string(),
                ));
            };
            self.task_store.sort();
            if let Some(task_pos) = self.task_store.task_position(&task) {
                *selected_index = task_pos;
            }
        }
        Ok(PostEvent::noop(false))
    }

    pub fn create_due_date_dialog(&mut self) -> Result<PostEvent, AppError> {
        let index = self.task_list.selected_index;
        let date_dialog = InputBoxBuilder::default()
            .title("Add date or specify \"none\" to remove".to_string())
            .on_submit(move |app, date_str| {
                if date_str.to_lowercase() == "none" {
                    if let Some(task) = app.task_store.task_mut(index) {
                        task.due_date = None;
                    }
                    return PostEvent::noop(false);
                }
                let date = NaiveDate::parse_from_str(&date_str, "%d/%m/%y")
                    .or_else(|_| NaiveDate::parse_from_str(&date_str, "%d/%m/%Y"))
                    .or_else(|_| NaiveDate::parse_from_str(&date_str, "%Y-%m-%d"));

                if let Some(task) = app.task_store.task_mut(index) {
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
