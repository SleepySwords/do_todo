use std::{cell::RefCell, rc::Rc};

use tui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders},
};

// A viewer of a task/something
use crate::{
    app::{App, Mode},
    draw::{DrawableComponent, Drawer, EventResult},
    task::Task,
    utils,
};

#[derive(Default)]
pub struct Viewer {
    area: Rect,
    task_index: Rc<RefCell<usize>>,
    completed_task_index: Rc<RefCell<usize>>,
}

impl Viewer {
    pub fn new(task_index: Rc<RefCell<usize>>, completed_task_index: Rc<RefCell<usize>>) -> Viewer {
        Viewer {
            area: Rect::default(),
            task_index,
            completed_task_index,
        }
    }

    fn draw_task_viewer(&self, app: &App, block: Block, drawer: &mut Drawer) {
        let theme = &app.theme;
        let index = *self.task_index.borrow();
        let task: &Task = &app.task_store.tasks[index];

        let constraints = [Constraint::Percentage(20), Constraint::Percentage(80)];

        let items = vec![
            (
                Span::raw("Title"),
                Line::from(Span::from(task.title.as_str())),
            ),
            (
                Span::raw("Priority"),
                Line::from(Span::styled(
                    task.priority.display_string(),
                    Style::default().fg(task.priority.colour(theme)),
                )),
            ),
            (Span::raw("Tags"), tag_names(app, task)),
        ];

        // NOTE: I have no idea why the width must be three less, should probably investigate.
        let table =
            utils::ui::generate_table(items, constraints[1].apply(self.area.width) as usize - 3)
                .block(block)
                .widths(&constraints);

        drawer.draw_widget(table, self.area)
    }

    fn draw_completed_task_viewer(
        &self,
        app: &App,
        block: Block,
        draw_area: Rect,
        drawer: &mut Drawer,
    ) {
        let completed_task = &app.task_store.completed_tasks[*self.completed_task_index.borrow()];
        let completed_time = completed_task
            .time_completed
            .format("%d/%m/%y %-I:%M:%S %p")
            .to_string();

        let constraints = [Constraint::Percentage(25), Constraint::Percentage(75)];
        let items = vec![
            (
                Span::raw("Title"),
                Line::from(
                    completed_task
                        .task
                        .title
                        .split('\n')
                        .map(Span::from)
                        .collect::<Vec<Span>>(),
                ),
            ),
            (
                Span::raw("Date Completed"),
                Line::from(&completed_time as &str),
            ),
            (
                Span::raw("Priority"),
                Line::from(Span::styled(
                    completed_task.task.priority.display_string(),
                    Style::default().fg(completed_task.task.priority.colour(&app.theme)),
                )),
            ),
            (Span::raw("Tags"), tag_names(app, &completed_task.task)),
        ];

        let table =
            utils::ui::generate_table(items, constraints[1].apply(draw_area.width) as usize - 2)
                .block(block)
                .widths(&constraints);

        drawer.draw_widget(table, draw_area)
    }
}

fn tag_names<'a>(app: &'a App, task: &'a Task) -> Line<'a> {
    if task.tags.is_empty() {
        Line::from("None")
    } else {
        // FIX: Can be replaced once https://github.com/rust-lang/rust/issues/79524 stabilises
        let line = itertools::intersperse(
            task.iter_tags(app).map(|tag| {
                let name = &tag.name;
                let colour = Style::default().fg(tag.colour);
                Span::styled(name, colour)
            }),
            Span::raw(", "),
        )
        .collect::<Vec<Span>>();
        Line::from(line)
    }
}

impl DrawableComponent for Viewer {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let theme = &app.theme;
        let draw_area = self.area;
        let block = Block::default()
            .title("Task information")
            .borders(Borders::ALL)
            .border_type(theme.border_style.border_type);

        match app.mode {
            Mode::CurrentTasks => {
                if !app.task_store.tasks.is_empty() {
                    self.draw_task_viewer(app, block, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
            Mode::CompletedTasks => {
                if !app.task_store.completed_tasks.is_empty() {
                    self.draw_completed_task_viewer(app, block, draw_area, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
            Mode::Overlay => {
                if !app.task_store.tasks.is_empty() {
                    self.draw_task_viewer(app, block, drawer)
                } else {
                    drawer.draw_widget(block, draw_area);
                }
            }
        }
    }

    fn key_event(&mut self, _app: &mut App, _key_code: crossterm::event::KeyEvent) -> EventResult {
        EventResult::Ignored
    }

    fn update_layout(&mut self, area: Rect) {
        self.area = area;
    }
}
