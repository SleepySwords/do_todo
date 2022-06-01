use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{ListItem, StatefulWidget, List, Block, Borders, BorderType, ListState},
};

use crate::app::{App, Windows};

struct TaskList<'a> {
    app: &'a App,
}

struct TaskState;

impl StatefulWidget for TaskList<'_> {
    type State = TaskState;

    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer, state: &mut TaskState) {
        let theme = &self.app.theme;
        let selected_index = Some(0);
        let tasks: Vec<ListItem> = self
            .app
            .task_data
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let style = if selected_index == Some(i) {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[-] " } else { "[ ] " },
                    style.fg(if selected_index == Some(i) {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }),
                );

                let priority = Span::styled(
                    format!("{}", task.priority.get_short_hand()),
                    style.fg(task.priority.get_colour(theme)),
                );

                let content = Span::styled(
                    task.title.as_str(),
                    // style.fg(task.priority.get_colour(theme)),
                    style,
                );

                let content = Spans::from(vec![progress, priority, content]);
                ListItem::new(content)
            })
            .collect();

        let border_colour = match self.app.selected_window {
            Windows::CurrentTasks(_) => theme.selected_border_colour,
            _ => theme.default_border_colour,
        };

        let current = List::new(tasks).block(
            Block::default()
                .title("Current List")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_colour)),
        );

        let mut state = ListState::default();
        state.select(
            if let Windows::CurrentTasks(selected) = self.app.selected_window {
                Some(selected)
            } else {
                None
            },
        );

        StatefulWidget::render(current, area, buf, &mut state);
    }
}
