use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crossterm::event::KeyCode;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use super::input::input_box::InputBoxBuilder;
use crate::{
    actions::{self, HelpAction},
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    theme::KeyBindings,
    utils::{self, handle_mouse_movement},
};

const COMPONENT_TYPE: Mode = Mode::CurrentTasks;

pub struct TaskList {
    pub area: Rect,
    selected_index: Rc<RefCell<usize>>,
}

impl TaskList {
    pub fn new(selected_index: Rc<RefCell<usize>>) -> Self {
        Self {
            selected_index,
            area: Rect::default(),
        }
    }

    fn selected(&self) -> Ref<usize> {
        self.selected_index.borrow()
    }

    fn selected_mut(&self) -> RefMut<usize> {
        self.selected_index.borrow_mut()
    }

    pub fn available_actions() -> Vec<HelpAction<'static>> {
        vec![
            HelpAction::new(KeyCode::Char('a'), "a", "Adds a task"),
            HelpAction::new(KeyCode::Char('c'), "c", "Completes the selected task"),
            HelpAction::new(KeyCode::Char('d'), "d", "Delete the selected task"),
            HelpAction::new(KeyCode::Char('e'), "e", "Edits the selected task"),
            HelpAction::new(KeyCode::Char('f'), "f", "Flip a tag to the selected task"),
            HelpAction::new(
                KeyCode::Char('t'),
                "t",
                "Add or remove the tags for this project",
            ),
            HelpAction::new(
                KeyCode::Char('h'),
                "h",
                "Gives selected task lower priority",
            ),
            HelpAction::new(
                KeyCode::Char('J'),
                "J",
                "Moves the task down on the task list",
            ),
            HelpAction::new(
                KeyCode::Char('K'),
                "K",
                "Moves the task up on the task list",
            ),
            HelpAction::new(KeyCode::Char('j'), "j", "Moves down one task"),
            HelpAction::new(KeyCode::Char('k'), "k", "Moves up one task"),
            HelpAction::new(KeyCode::Char('s'), "s", "Sorts tasks (by priority)"),
            HelpAction::new(KeyCode::Char('S'), "S", "Toggles automatic task sort"),
        ]
    }
}

impl DrawableComponent for TaskList {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let theme = &app.theme;
        let tasks: Vec<ListItem> = app
            .task_store
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let mut spans = Vec::new();

                let style = if COMPONENT_TYPE == app.mode && *self.selected() == i {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[~] " } else { "[ ] " },
                    style.fg(if COMPONENT_TYPE == app.mode && *self.selected() == i {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }),
                );
                spans.push(progress);

                let priority = Span::styled(
                    task.priority.short_hand(),
                    style.fg(task.priority.colour(theme)),
                );
                spans.push(priority);

                // TODO: Rewrite to store as an array in the task
                let content = Span::styled(
                    task.title.split('\n').next().unwrap(),
                    style.fg(if COMPONENT_TYPE == app.mode && *self.selected() == i {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }),
                );
                spans.push(content);

                for tag in task.iter_tags(app) {
                    let tag_label =
                        Span::styled(format!(" ({})", tag.name), Style::default().fg(tag.colour));
                    spans.push(tag_label);
                }

                let content = Line::from(spans);
                ListItem::new(content)
            })
            .collect();

        let current = List::new(tasks).block(utils::ui::generate_default_block(
            app,
            "Current List",
            COMPONENT_TYPE,
        ));

        let mut state = ListState::default();
        state.select(if COMPONENT_TYPE == app.mode {
            Some(*self.selected())
        } else {
            None
        });

        drawer.draw_stateful_widget(current, &mut state, self.area);
    }

    fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let theme = &app.theme;
        let mut selected_index = self.selected_mut();

        // Move this to the actions class
        match KeyBindings::from_event(theme, key_event) {
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
            KeyBindings::DeleteKey => {
                actions::open_delete_task_menu(app, self.selected_index.clone())
            }
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
            KeyBindings::TagMenu => actions::open_tag_menu(app, *selected_index),
            KeyBindings::FlipProgressKey => {
                if app.task_store.tasks.is_empty() {
                    return EventResult::Ignored;
                }
                app.task_store.tasks[*selected_index].progress =
                    !app.task_store.tasks[*selected_index].progress;
            }
            KeyBindings::CompleteKey => actions::complete_task(app, &mut selected_index),
            _ => {
                return utils::handle_key_movement(
                    theme,
                    key_event,
                    &mut selected_index,
                    app.task_store.tasks.len(),
                );
            }
        }
        EventResult::Consumed
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        handle_mouse_movement(
            app,
            self.area,
            Some(COMPONENT_TYPE),
            app.task_store.tasks.len(),
            &mut self.selected_index.borrow_mut(),
            mouse_event,
        )
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
