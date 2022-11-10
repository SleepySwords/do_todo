use std::{cell::RefCell, rc::Rc};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders},
};

// A viewer of a task/something
use crate::{
    app::{App, SelectedComponent},
    task::Task,
    utils,
    view::{DrawableComponent, Drawer, EventResult},
};

#[derive(Default)]
pub struct Viewer {
    task_index: Rc<RefCell<usize>>,
    completed_task_index: Rc<RefCell<usize>>,
}

impl Viewer {
    pub fn new(task_index: Rc<RefCell<usize>>, completed_task_index: Rc<RefCell<usize>>) -> Viewer {
        Viewer {
            task_index,
            completed_task_index,
        }
    }

    fn draw_task_viewer(&self, app: &App, block: Block, draw_area: Rect, drawer: &mut Drawer) {
        let theme = &app.theme;
        let index = *self.task_index.borrow();
        let task: &Task = &app.task_store.tasks[index];

        let constraints = [Constraint::Percentage(20), Constraint::Percentage(80)];
        let tags_name = if task.tags.is_empty() {
            Spans::from("None")
        } else {
            // FIX: Can be replaced once https://github.com/rust-lang/rust/issues/79524 stabilises
            let spans = itertools::intersperse(
                task.iter_tags(app).map(|tag| {
                    let name = &tag.name;
                    let colour = Style::default().fg(tag.colour);
                    Span::styled(name, colour)
                }),
                Span::raw(", "),
            )
            .collect::<Vec<Span>>();
            Spans::from(spans)
        };

        let items = vec![
            (Span::raw("Title"), Spans::from(&task.title as &str)),
            (
                Span::raw("Priority"),
                Spans::from(Span::styled(
                    task.priority.display_string(),
                    Style::default().fg(task.priority.colour(theme)),
                )),
            ),
            (Span::raw("Tags"), tags_name),
        ];

        let table =
            utils::generate_table(items, constraints[1].apply(draw_area.width) as usize - 2)
                .block(block)
                .widths(&constraints);

        drawer.draw_widget(table, draw_area)
    }

    fn draw_completed_task_viewer(
        &self,
        app: &App,
        block: Block,
        draw_area: Rect,
        drawer: &mut Drawer,
    ) {
        let task = &app.task_store.completed_tasks[*self.completed_task_index.borrow()];
        let completed_time = task
            .time_completed
            .format("%d/%m/%y %-I:%M:%S %p")
            .to_string();

        let constraints = [Constraint::Percentage(25), Constraint::Percentage(75)];
        let items = vec![
            (Span::raw("Title"), Spans::from(&task.task.title as &str)),
            (
                Span::raw("Date Completed"),
                Spans::from(&completed_time as &str),
            ),
        ];

        let table =
            utils::generate_table(items, constraints[1].apply(draw_area.width) as usize - 2)
                .block(block)
                .widths(&constraints);

        drawer.draw_widget(table, draw_area)
    }
}

impl DrawableComponent for Viewer {
    fn draw(&self, app: &App, draw_area: tui::layout::Rect, drawer: &mut Drawer) {
        let theme = &app.theme;
        let block = Block::default()
            .title("Task information")
            .borders(Borders::ALL)
            .border_type(theme.border_style.border_type);

        match app.selected_component {
            SelectedComponent::CurrentTasks => {
                if !app.task_store.tasks.is_empty() {
                    self.draw_task_viewer(app, block, draw_area, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
            SelectedComponent::CompletedTasks => {
                if !app.task_store.completed_tasks.is_empty() {
                    self.draw_completed_task_viewer(app, block, draw_area, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
            SelectedComponent::Overlay => {
                if !app.task_store.tasks.is_empty() {
                    self.draw_task_viewer(app, block, draw_area, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
        }
    }

    fn key_pressed(&mut self, _app: &mut App, _key_code: crossterm::event::KeyCode) -> EventResult {
        EventResult::Ignored
    }
}
