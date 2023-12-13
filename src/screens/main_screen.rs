use std::{cell::RefCell, rc::Rc};

use crate::{
    actions,
    app::{App, Mode},
    component::{
        completed_list::CompletedList, input::input_box::InputBoxBuilder, task_list::TaskList,
        viewer::Viewer,
    },
    draw::{DrawableComponent, Drawer, EventResult},
    task::Task,
    theme::KeyBindings,
    utils,
};
use crossterm::event::MouseEvent;
use tui::layout::{Constraint, Direction, Layout, Rect};

const MINIMUM_SCREEN: u16 = 100;

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
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        drawer.draw_component(app, &self.task_list);
        drawer.draw_component(app, &self.completed_list);
        drawer.draw_component(app, &self.viewer);
    }

    fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let event_result = match app.mode {
            Mode::CurrentTasks => self.task_list.key_event(app, key_event),
            Mode::CompletedTasks => self.completed_list.key_event(app, key_event),
            _ => EventResult::Ignored,
        };

        if event_result == EventResult::Consumed {
            return event_result;
        }

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
        let (task_layout, completed_layout, viewer_layout) = if layout.width < MINIMUM_SCREEN {
            let main_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(layout);
            (main_chunk[1], main_chunk[2], main_chunk[0])
        } else {
            let main_chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(layout);

            let layout_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                .split(main_chunk[0]);

            (layout_chunk[0], layout_chunk[1], main_chunk[1])
        };

        self.task_list.update_layout(task_layout);
        self.completed_list.update_layout(completed_layout);
        self.viewer.update_layout(viewer_layout);
    }
}
