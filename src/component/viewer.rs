use tui::{
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Block,
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
}

impl Viewer {
    pub fn new() -> Viewer {
        Viewer {
            area: Rect::default(),
        }
    }

    fn draw_task_viewer(&self, app: &App, block: Block, drawer: &mut Drawer) {
        let theme = &app.config;
        let index = app.task_list.selected_index;
        let Some(task) = app.task_store.task(index) else {
            return;
        };

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
                .widths(constraints);

        drawer.draw_widget(table, self.area)
    }

    fn draw_completed_task_viewer(
        &self,
        app: &App,
        block: Block,
        draw_area: Rect,
        drawer: &mut Drawer,
    ) {
        let completed_task = &app.task_store.completed_tasks[app.completed_list.selected_index];
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
                    Style::default().fg(completed_task.task.priority.colour(&app.config)),
                )),
            ),
            (Span::raw("Tags"), tag_names(app, &completed_task.task)),
        ];

        let table =
            utils::ui::generate_table(items, constraints[1].apply(draw_area.width) as usize - 2)
                .block(block)
                .widths(constraints);

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
        let theme = &app.config;
        let draw_area = self.area;
        let block = theme.styled_block("Task information", theme.default_border_colour);

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
