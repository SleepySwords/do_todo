use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::actions::{self, HelpAction};
use crate::app::{App, SelectedComponent};
use crate::utils;
use crate::view::{DrawableComponent, EventResult};

use super::input::input_box::InputBox;

const COMPONENT_TYPE: SelectedComponent = SelectedComponent::CurrentTasks;

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
            HelpAction::new(KeyCode::Char('t'), "t", "Add tags to the task"),
        ]
    }
}

impl DrawableComponent for TaskList {
    fn draw(&self, app: &App, _: Rect, drawer: &mut crate::view::Drawer) {
        let theme = &app.theme;
        let tasks: Vec<ListItem> = app
            .task_store
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let mut spans = Vec::new();

                let style = if COMPONENT_TYPE == app.selected_component && *self.selected() == i {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[~] " } else { "[ ] " },
                    style.fg(
                        if COMPONENT_TYPE == app.selected_component && *self.selected() == i {
                            theme.selected_task_colour
                        } else {
                            Color::White
                        },
                    ),
                );
                spans.push(progress);

                let priority = Span::styled(
                    task.priority.short_hand(),
                    style.fg(task.priority.colour(theme)),
                );
                spans.push(priority);

                // TODO: Rewrite to store as an array in the task
                let content = Span::styled(task.title.split('\n').next().unwrap(), style);
                spans.push(content);

                for tag in task.iter_tags(app) {
                    let tag_label =
                        Span::styled(format!(" ({})", tag.name), Style::default().fg(tag.colour));
                    spans.push(tag_label);
                }

                let content = Spans::from(spans);
                ListItem::new(content)
            })
            .collect();

        let current = List::new(tasks).block(utils::generate_default_block(
            "Current List",
            COMPONENT_TYPE,
            app,
        ));

        let mut state = ListState::default();
        state.select(if COMPONENT_TYPE == app.selected_component {
            Some(*self.selected())
        } else {
            None
        });

        drawer.draw_stateful_widget(current, &mut state, self.area);
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        let mut selected_index = self.selected_mut();

        match key_code {
            // Move this to the actions class
            KeyCode::Char('h') => {
                if app.task_store.tasks.is_empty() {
                    return EventResult::Ignored;
                }
                app.task_store.tasks[*selected_index].priority = app.task_store.tasks
                    [*selected_index]
                    .priority
                    .next_priority();
            }
            KeyCode::Char('J') => {
                let task_length = app.task_store.tasks.len();
                let task = app.task_store.tasks.remove(*selected_index);
                app.task_store
                    .tasks
                    .insert((*selected_index + 1) % task_length, task);
                *selected_index = (*selected_index + 1) % task_length;
            }
            KeyCode::Char('K') => {
                let task_length = app.task_store.tasks.len();
                let task = app.task_store.tasks.remove(*selected_index);
                if *selected_index == 0 {
                    app.task_store.tasks.insert(task_length - 1, task);
                    *selected_index = task_length - 1;
                } else {
                    app.task_store
                        .tasks
                        .insert((*selected_index - 1) % task_length, task);
                    *selected_index = (*selected_index - 1) % task_length;
                }
            }
            KeyCode::Char('d') => actions::open_delete_task_menu(app, self.selected_index.clone()),
            KeyCode::Char('e') => {
                let index = *selected_index;
                app.push_layer(InputBox::filled(
                    String::from("Edit the selected task"),
                    app.task_store.tasks[*selected_index].title.as_str(),
                    Box::new(move |app, mut word| {
                        app.task_store.tasks[index].title =
                            word.drain(..).collect::<String>().trim().to_string();
                        Ok(())
                    }),
                ))
            }
            KeyCode::Char('f') => actions::flip_tag_menu(app, *selected_index),
            KeyCode::Char('t') => actions::edit_tag_menu(app, *selected_index),
            KeyCode::Enter => {
                if app.task_store.tasks.is_empty() {
                    return EventResult::Ignored;
                }
                app.task_store.tasks[*selected_index].progress =
                    !app.task_store.tasks[*selected_index].progress;
            }
            KeyCode::Char('c') => actions::complete_task(app, &mut selected_index),
            _ => {
                utils::handle_movement(key_code, &mut selected_index, app.task_store.tasks.len());
            }
        }
        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        MouseEvent { row, kind, .. }: crossterm::event::MouseEvent,
    ) -> EventResult {
        if let MouseEventKind::ScrollUp = kind {
            if *self.selected_index.borrow() != 0 {
                *self.selected_index.borrow_mut() -= 1;
            }
        }

        if let MouseEventKind::ScrollDown = kind {
            if *self.selected_index.borrow() < app.task_store.tasks.len() - 1 {
                *self.selected_index.borrow_mut() += 1;
            }
        }

        if let MouseEventKind::Down(_) = kind {
            if let COMPONENT_TYPE = app.selected_component {
            } else {
                app.selected_component = COMPONENT_TYPE;
            }
            if row == 0 {
                return EventResult::Ignored;
            }
            if *self.selected_index.borrow() > self.area.height as usize - 2 {
                let new_index =
                    *self.selected_index.borrow() - (self.area.height as usize - 2) + row as usize;
                *self.selected_index.borrow_mut() = new_index;
            } else {
                if row as usize > app.task_store.tasks.len() {
                    *self.selected_index.borrow_mut() = app.task_store.tasks.len() - 1;
                    return EventResult::Ignored;
                }
                *self.selected_index.borrow_mut() = row as usize - 1;
            }
        }
        EventResult::Ignored
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
