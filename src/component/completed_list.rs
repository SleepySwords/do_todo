use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::app::{App, SelectedComponent};
use crate::{actions, utils};

const COMPONENT_TYPE: SelectedComponent = SelectedComponent::CompletedTasks;

pub struct CompletedList;

impl CompletedList {
    fn selected(app: &App) -> &usize {
        &app.selected_completed_task_index
    }

    pub fn handle_event(app: &mut App, key_code: KeyCode) -> Option<()> {
        utils::handle_movement(
            key_code,
            &mut app.selected_completed_task_index,
            app.task_data.completed_tasks.len(),
        );
        let selected_index = *Self::selected(app);
        if let KeyCode::Char('r') = key_code {
            actions::restore_task(app, selected_index)
        }
        Some(())
    }

    pub fn draw<B: tui::backend::Backend>(
        app: &App,
        layout_chunk: Rect,
        frame: &mut tui::Frame<B>,
    ) {
        let theme = &app.theme;

        let completed_tasks: Vec<ListItem> = app
            .task_data
            .completed_tasks
            .iter()
            .enumerate()
            .map(|(ind, task)| {
                let colour = if let SelectedComponent::CompletedTasks = app.selected_component {
                    let i = app.selected_completed_task_index;
                    if i == ind {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }
                } else {
                    Color::White
                };
                let content = Spans::from(Span::styled(
                    format!(
                        "{} {}",
                        task.time_completed.format("%d/%m/%y %-I:%M:%S %p"),
                        task.title
                    ),
                    Style::default().fg(colour),
                ));
                ListItem::new(content)
            })
            .collect();

        let recently_competed = List::new(completed_tasks)
            .block(utils::generate_block(
                "Completed tasks",
                COMPONENT_TYPE,
                app,
            ))
            .style(Style::default().fg(Color::White));

        let mut completed_state = ListState::default();
        if !app.task_data.completed_tasks.is_empty() {
            let index = match app.selected_component {
                SelectedComponent::CompletedTasks => app.selected_completed_task_index,
                _ => app.task_data.completed_tasks.len() - 1,
            };
            completed_state.select(Some(index));
        }

        frame.render_stateful_widget(recently_competed, layout_chunk, &mut completed_state);
    }
}
