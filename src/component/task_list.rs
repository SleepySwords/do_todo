use itertools::Itertools;
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    framework::{
        component::{Component, Drawer},
        event::PostEvent,
    },
    task::Task,
    utils::{self, handle_mouse_movement_app},
};

const COMPONENT_TYPE: Mode = Mode::CurrentTasks;

pub struct TaskList {
    pub area: Rect,
}

#[derive(Default)]
pub struct TaskListContext {
    pub selected_index: usize,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            area: Rect::default(),
        }
    }

    fn is_task_selected(app: &App, current_index: &usize) -> bool {
        COMPONENT_TYPE == app.mode && app.task_list.selected_index == *current_index
    }

    fn draw_task<'a>(
        app: &'a App,
        task: &'a Task,
        nested_level: usize,
        task_index: &mut usize,
    ) -> Vec<Line<'a>> {
        let config = &app.config;

        let mut spans = Vec::new();

        let style = if Self::is_task_selected(app, task_index) {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let progress = Span::styled(
            if task.progress { "[~] " } else { "[ ] " },
            style.fg(if Self::is_task_selected(app, task_index) {
                config.selected_task_colour
            } else {
                Color::White
            }),
        );
        spans.push(progress);

        let padding = config.nested_padding.repeat(nested_level);
        spans.push(Span::styled(padding, Style::default().fg(Color::DarkGray)));

        if task.sub_tasks.is_empty() {
            let priority = Span::styled(
                task.priority.short_hand(),
                style.fg(task.priority.colour(config)),
            );
            spans.push(priority);
        } else {
            let sub_tasks = Span::styled(
                if task.opened {
                    &config.open_subtask
                } else {
                    &config.closed_subtask
                },
                style.fg(task.priority.colour(config)),
            );
            spans.push(sub_tasks);
        }

        let content = Span::styled(
            task.title.split('\n').next().unwrap(),
            style.fg(if Self::is_task_selected(app, task_index) {
                config.selected_task_colour
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

        if let Some(due_date) = task.due_date {
            let due_label = Span::styled(
                due_date.format(" [%-d %b %C%y]").to_string(),
                config.date_colour(due_date),
            );
            spans.push(due_label);
        }

        *task_index += 1;

        if task.opened {
            let mut drawn_tasks = task
                .sub_tasks
                .iter()
                .flat_map(|sub_task| {
                    let drawn_task = Self::draw_task(app, sub_task, nested_level + 1, task_index);
                    drawn_task
                })
                .collect_vec();
            (drawn_tasks).insert(0, Line::from(spans));
            drawn_tasks
        } else {
            vec![Line::from(spans)]
        }
    }
}

impl Component for TaskList {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let mut current_index = 0;
        let tasks: Vec<ListItem> = app
            .task_store
            .tasks
            .iter()
            .flat_map(|task| Self::draw_task(app, task, 0, &mut current_index))
            .map(ListItem::from)
            .collect();

        let current = List::new(tasks).block(utils::ui::generate_default_block(
            app,
            "Current List",
            COMPONENT_TYPE,
        ));

        let mut state = ListState::default();
        state.select(if COMPONENT_TYPE == app.mode {
            Some(app.task_list.selected_index)
        } else {
            None
        });

        drawer.draw_stateful_widget(current, &mut state, self.area);
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        handle_mouse_movement_app(
            app,
            self.area,
            COMPONENT_TYPE,
            app.task_store.find_tasks_draw_size(),
            mouse_event,
        )
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
