use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders};
use tui::widgets::{List, ListItem, ListState};

use crate::utils;
use crate::{
    app::{App, SelectedComponent},
    input::Component,
};

pub struct TaskList {
    index: usize,
}

// Movement macro

impl Component for TaskList {
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()> {
        utils::handle_movement(key_code, &mut self.index, app.task_data.tasks.len());
        Some(())
    }

    fn draw<B: tui::backend::Backend>(
        &self,
        app: &App,
        layout_chunk: Rect,
        frame: &mut tui::Frame<B>,
    ) {
        let theme = &app.theme;
        let selected_index = self.index;
        let tasks: Vec<ListItem> = app
            .task_data
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let style = if selected_index == i {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[-] " } else { "[ ] " },
                    style.fg(if selected_index == i {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }),
                );

                let priority = Span::styled(
                    task.priority.short_hand(),
                    style.fg(task.priority.colour(theme)),
                );

                let content = Span::styled(
                    task.title.as_str(),
                    // style.fg(task.priority.colour(theme)),
                    style,
                );

                let content = Spans::from(vec![progress, priority, content]);
                ListItem::new(content)
            })
            .collect();

        let border_colour = match app.selected_window {
            SelectedComponent::CurrentTasks(_) => theme.selected_border_colour,
            _ => theme.default_border_colour,
        };

        let current = List::new(tasks).block(
            Block::default()
                .title("Current List")
                .borders(Borders::ALL)
                .border_type(theme.border_style.border_type)
                .border_style(Style::default().fg(border_colour)),
        );

        let mut state = ListState::default();
        state.select(
            if let SelectedComponent::CurrentTasks(selected) = app.selected_window {
                Some(selected)
            } else {
                None
            },
        );

        frame.render_stateful_widget(current, layout_chunk, &mut state);
    }
}
