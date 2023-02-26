use std::{cell::RefCell, rc::Rc};

use crate::{
    actions,
    app::{App, SelectedComponent},
    component::{
        completed_list::CompletedList, input::input_box::InputBox, task_list::TaskList,
        viewer::Viewer,
    },
    task::Task,
    utils,
    view::{DrawableComponent, Drawer, EventResult},
};
use crossterm::event::{KeyCode, MouseEvent};
use tui::layout::{Constraint, Direction, Layout, Rect};

pub struct MainScreenLayer {
    task_list: TaskList,
    completed_list: CompletedList,
    layout: Rect,
    viewer: Viewer,
}

impl MainScreenLayer {
    pub fn new() -> MainScreenLayer {
        // The use of a RefCell means that we have to be more carefull in where we borrow this
        // variable. Ie: No storing borrowed references.
        let task_index = Rc::new(RefCell::new(0));
        let completed_task_index = Rc::new(RefCell::new(0));
        MainScreenLayer {
            task_list: TaskList::new(task_index.clone()),
            completed_list: CompletedList::new(completed_task_index.clone()),
            layout: Rect::default(),
            viewer: Viewer::new(task_index, completed_task_index),
        }
    }
}

impl DrawableComponent for MainScreenLayer {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer) {
        drawer.draw_component(app, &self.task_list, draw_area);
        drawer.draw_component(app, &self.completed_list, draw_area);
        drawer.draw_component(app, &self.viewer, draw_area);
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        let event_result = match app.selected_component {
            SelectedComponent::CurrentTasks => self.task_list.key_pressed(app, key_code),
            SelectedComponent::CompletedTasks => self.completed_list.key_pressed(app, key_code),
            _ => EventResult::Ignored,
        };

        if event_result == EventResult::Consumed {
            return event_result;
        }

        // Global keybindings
        match key_code {
            KeyCode::Char('a') => {
                app.push_layer(InputBox::new(String::from("Add a task"), |app, word| {
                    app.task_store
                        .tasks
                        .push(Task::from_string(word.trim().to_string()));
                    Ok(())
                }));
                EventResult::Consumed
            }
            KeyCode::Char('1') => {
                app.selected_component = SelectedComponent::CurrentTasks;
                EventResult::Consumed
            }
            KeyCode::Char('2') => {
                app.selected_component = SelectedComponent::CompletedTasks;
                EventResult::Consumed
            }
            KeyCode::Char('x') => {
                actions::open_help_menu(app);
                EventResult::Consumed
            }
            KeyCode::Char('q') => {
                app.shutdown();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        let MouseEvent { row, column, .. } = mouse_event;
        if utils::inside_rect((row, column), self.task_list.area) {
            self.task_list.mouse_event(app, mouse_event);
        } else if utils::inside_rect((row, column), self.completed_list.area) {
            self.completed_list.mouse_event(app, mouse_event);
        }
        EventResult::Ignored
    }

    fn update_layout(&mut self, layout: Rect) {
        self.layout = layout;
        let main_chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout);

        let layout_chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(main_chunk[0]);

        self.task_list.update_layout(layout_chunk[0]);
        self.completed_list.update_layout(layout_chunk[1]);
        self.viewer.update_layout(main_chunk[1]);
    }
}
