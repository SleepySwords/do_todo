use crate::{data::data_store::DataTaskStore, utils::task_position::cursor_to_completed_task};
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};

use crate::{
    app::{App, Mode},
    framework::{
        component::{Component, Drawer},
        event::PostEvent,
    },
    utils,
};

const COMPONENT_TYPE: Mode = Mode::CompletedTasks;

pub struct CompletedList {
    pub area: Rect,
}

#[derive(Default)]
pub struct CompletedListContext {
    pub selected_index: usize,
}

impl CompletedList {
    pub fn new() -> Self {
        Self {
            area: Rect::default(),
        }
    }

    pub fn restore_task(app: &mut App) {
        if app.task_store.completed_root_tasks().is_empty() {
            return;
        }

        if let Some(completed_task) =
            cursor_to_completed_task(&app.task_store, app.completed_list.selected_index)
        {
            app.task_store.restore(&completed_task);
            if app.completed_list.selected_index >= app.task_store.completed_root_tasks().len()
                && !app.task_store.completed_root_tasks().is_empty()
            {
                app.completed_list.selected_index -= 1;
            }
        }
    }
}

impl Component for CompletedList {
    fn draw(&self, app: &App, drawer: &mut Drawer) {
        let theme = &app.config;

        let selected_index = app.completed_list.selected_index;

        let completed_tasks: Vec<ListItem> = app
            .task_store
            .completed_root_tasks()
            .iter()
            .enumerate()
            .map(|(i, task_id)| {
                let Some(task) = app.task_store.completed_task(task_id) else {
                    // FIXME: Error gracefyly
                    panic!("ok");
                };
                let colour = if let Mode::CompletedTasks = app.mode {
                    if selected_index == i {
                        theme.selected_task_colour
                    } else {
                        theme.default_task_colour
                    }
                } else {
                    theme.default_task_colour
                };
                let content = Line::from(Span::styled(
                    format!(
                        "{} {}",
                        task.time_completed.format("%d/%m/%y %-I:%M:%S %p"),
                        task.task.title
                    ),
                    Style::default().fg(colour),
                ));
                ListItem::new(content)
            })
            .collect();

        let completed_list = List::new(completed_tasks)
            .block(utils::ui::generate_default_block(
                app,
                "Completed tasks",
                COMPONENT_TYPE,
            ))
            .style(Style::default().fg(Color::White));

        let mut completed_state = ListState::default();
        if !app.task_store.completed_root_tasks().is_empty() {
            completed_state.select(Some(selected_index));
        }

        drawer.draw_stateful_widget(completed_list, &mut completed_state, self.area);
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> PostEvent {
        utils::handle_mouse_movement_app(
            app,
            self.area,
            COMPONENT_TYPE,
            app.task_store.completed_root_tasks().len(),
            mouse_event,
        )
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
