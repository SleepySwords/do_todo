use std::{cell::RefCell, rc::Rc};

use crate::{
    actions,
    app::{App, SelectedComponent},
    component::{
        completed_list::CompletedList, input::input_box::InputBox, task_list::TaskList,
        viewer::Viewer,
    },
    task::Task,
    view::{DrawableComponent, Drawer, EventResult, WidgetComponent},
};
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    Frame,
};

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
            viewer: Viewer::new(task_index.clone(), completed_task_index.clone()),
        }
    }

    fn draw_tasks<B>(app: &App, frame: &mut Frame<B>, layout_chunk: Rect)
    where
        B: Backend,
    {
        // let layout_chunk = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        //     .split(layout_chunk);

        // TaskList::draw(app, layout_chunk[0], frame);

        // CompletedList::draw(app, layout_chunk[1], frame)
    }

    pub fn draw<B: Backend>(app: &mut App, f: &mut Frame<B>) {
        // let layout = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
        //     .split(f.size());

        // let main_body = layout[0];
        // let status_line = layout[1];

        // app.status_line.draw(app, status_line, f);

        // let chunks = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        //     .split(main_body);

        // Viewer::draw(app, chunks[1], f);
        // Self::draw_tasks(app, f, chunks[0]);

        // if let Some(component) = app.popup_stack.last() {
        //     match component {
        //         UserInputType::Input(component) => {
        //             let layout_chunk = utils::centre_rect(
        //                 Constraint::Percentage(70),
        //                 Constraint::Length(
        //                     (component.user_input.len() as u16).max(1) + input_box::PADDING as u16,
        //                 ),
        //                 f.size(),
        //             );
        //             component.draw(app, layout_chunk, f)
        //         }
        //         UserInputType::Dialog(component) => {
        //             let layout_chunk = utils::centre_rect(
        //                 Constraint::Percentage(70),
        //                 Constraint::Length(component.options.len() as u16 + 2),
        //                 f.size(),
        //             );
        //             component.draw(app, layout_chunk, f)
        //         }
        //         UserInputType::Message(component) => {
        //             let layout_chunk = utils::centre_rect(
        //                 Constraint::Percentage(70),
        //                 Constraint::Percentage(30),
        //                 f.size(),
        //             );
        //             component.draw(app, layout_chunk, f)
        //         }
        //     }
        // }
    }
}

impl DrawableComponent for MainScreenLayer {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut Drawer) {
        let layout_chunk = Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(draw_area);
        let layout_chunk2 = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(layout_chunk[0]);
        drawer.draw_component(app, &self.task_list, layout_chunk2[0]);
        drawer.draw_component(app, &self.completed_list, layout_chunk2[1]);
        drawer.draw_component(app, &self.viewer, layout_chunk[1]);
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
            KeyCode::Char('a') => app.append_stack(Box::new(InputBox::new(
                String::from("Add a task"),
                |app, mut word| {
                    app.task_store.tasks.push(Task::from_string(
                        word.drain(..).collect::<String>().trim().to_string(),
                    ));
                    Ok(())
                },
            ))),
            KeyCode::Char('1') => app.selected_component = SelectedComponent::CurrentTasks,
            KeyCode::Char('2') => app.selected_component = SelectedComponent::CompletedTasks,
            KeyCode::Char('x') => actions::open_help_menu(app),
            KeyCode::Char('q') => app.shutdown(),
            _ => {}
        }

        if key_code == KeyCode::Char('-') {
            app.append_stack(Box::new(WidgetComponent::new(Rect::new(5, 5, 5, 5))));
            return EventResult::Consumed;
        }
        if key_code == KeyCode::Char('0') {
            app.pop_stack();
            return EventResult::Consumed;
        }
        EventResult::Consumed
    }
}
