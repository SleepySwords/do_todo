use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use crate::{
    actions::HelpAction,
    app::{App, Mode},
    draw::{DrawableComponent, EventResult},
    config::Config,
    utils::{self, handle_mouse_movement},
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

    pub fn available_actions(config: &Config) -> Vec<HelpAction<'static>> {
        vec![
            HelpAction::new(config.add_key, "Adds a task"),
            HelpAction::new(config.complete_key, "Completes the selected task"),
            HelpAction::new(config.delete_key, "Delete the selected task"),
            HelpAction::new(config.edit_key, "Edits the selected task"),
            HelpAction::new(
                config.tag_menu,
                "Add or remove the tags from this task or project",
            ),
            HelpAction::new(
                config.change_priority_key,
                "Gives selected task lower priority",
            ),
            HelpAction::new(config.move_task_down, "Moves the task down on the task list"),
            HelpAction::new(config.move_task_up, "Moves the task up on the task list"),
            HelpAction::new_multiple(config.down_keys, "Moves down one task"),
            HelpAction::new_multiple(config.down_keys, "Moves up one task"),
            HelpAction::new(config.sort_key, "Sorts tasks (by priority)"),
            HelpAction::new(config.enable_autosort_key, "Toggles automatic task sort"),
        ]
    }
}

impl DrawableComponent for TaskList {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let theme = &app.config;
        let tasks: Vec<ListItem> = app
            .task_store
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let mut spans = Vec::new();

                let style = if COMPONENT_TYPE == app.mode && app.task_list.selected_index == i {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[~] " } else { "[ ] " },
                    style.fg(
                        if COMPONENT_TYPE == app.mode && app.task_list.selected_index == i {
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
                let content = Span::styled(
                    task.title.split('\n').next().unwrap(),
                    style.fg(
                        if COMPONENT_TYPE == app.mode && app.task_list.selected_index == i {
                            theme.selected_task_colour
                        } else {
                            Color::White
                        },
                    ),
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
    ) -> EventResult {
        handle_mouse_movement(
            app,
            self.area,
            COMPONENT_TYPE,
            app.task_store.tasks.len(),
            mouse_event,
        )
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
