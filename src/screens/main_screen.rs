use std::{cell::RefCell, rc::Rc, vec};

use crate::{
    actions,
    app::{App, SelectedComponent},
    component::{
        completed_list::CompletedList,
        input::input_box::InputBox,
        layout::adjacent_layout::{AdjacentLayout, Element},
        task_list::TaskList,
        viewer::Viewer,
    },
    task::Task,
    view::{DrawableComponent, Drawer, EventResult},
};
use crossterm::event::KeyCode;
use tui::layout::{Constraint, Direction, Rect};

pub struct MainScreenLayer {
    task_list: TaskList,
    completed_list: CompletedList,
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
            viewer: Viewer::new(task_index, completed_task_index),
        }
    }
}

impl DrawableComponent for MainScreenLayer {
    fn draw<'a>(&'a self, app: &App, draw_area: Rect, drawer: &mut Drawer) {
        let layout = AdjacentLayout {
            children: vec![
                (
                    Constraint::Percentage(50),
                    Element::owned(AdjacentLayout {
                        children: vec![
                            (Constraint::Percentage(70), Element::borrow(&self.task_list)),
                            (
                                Constraint::Percentage(30),
                                Element::borrow(&self.completed_list),
                            ),
                        ],
                        direction: Direction::Vertical,
                    }),
                ),
                (Constraint::Percentage(50), Element::borrow(&self.viewer)),
            ],
            direction: Direction::Horizontal,
        };

        drawer.draw_component(app, &layout, draw_area);
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        let result = match app.selected_component {
            crate::app::SelectedComponent::CurrentTasks => {
                self.task_list.key_pressed(app, key_code)
            }
            crate::app::SelectedComponent::CompletedTasks => {
                self.completed_list.key_pressed(app, key_code)
            }
            _ => crate::view::EventResult::Ignored,
        };

        // TODO: Simplify logic here.

        if result == EventResult::Consumed {
            return result;
        }

        match key_code {
            KeyCode::Char('a') => app.append_stack(InputBox::new(
                String::from("Add a task"),
                |app, mut word| {
                    app.task_store.tasks.push(Task::from_string(
                        word.drain(..).collect::<String>().trim().to_string(),
                    ));
                    Ok(())
                },
            )),
            KeyCode::Char('1') => app.selected_component = SelectedComponent::CurrentTasks,
            KeyCode::Char('2') => app.selected_component = SelectedComponent::CompletedTasks,
            KeyCode::Char('x') => actions::open_help_menu(app),
            KeyCode::Char('q') => app.shutdown(),
            _ => {}
        }

        EventResult::Consumed
    }
}
