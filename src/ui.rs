use crate::{
    app::{App, Mode, Windows},
    task::Task,
    theme::Theme,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn render_ui<B: Backend>(app: &mut App, f: &mut Frame<B>) {
    match app.selected_window {
        Windows::CurrentTasks(i) => {
            if !app.tasks.is_empty() {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(f.size());
                render_tasks(app, f, chunks[0]);
                render_selected_task(&app.tasks[i], &app.theme, f, chunks[1]);
            } else {
                render_tasks(app, f, f.size())
            }
        }
        _ => render_tasks(app, f, f.size()),
    }

    if let Mode::Edit(task_index) = app.mode {
        let text = Text::from(Spans::from(app.words.as_ref()));
        let help_message = Paragraph::new(text);
        let help_message = help_message.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!("Edit the task {}", app.tasks[task_index].title)),
        );
        let area = centered_rect(70, 20, f.size());
        f.render_widget(Clear, area);
        f.render_widget(help_message, area);
        f.set_cursor(area.x + 1 + app.words.len() as u16, area.y + 1)
    }

    if let Mode::Input = app.mode {
        let text = Text::from(Spans::from(app.words.as_ref()));
        let help_message = Paragraph::new(text);
        let help_message = help_message.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title("Add a task"),
        );
        let area = centered_rect(70, 20, f.size());
        f.render_widget(Clear, area);
        f.render_widget(help_message, area);
        f.set_cursor(area.x + 1 + app.words.len() as u16, area.y + 1)
    }
}

fn render_tasks<B>(app: &mut App, frame: &mut Frame<B>, layout_chunk: Rect)
where
    B: Backend,
{
    let task_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(layout_chunk);

    render_current_tasks(
        app,
        frame,
        task_layout[0],
        if let Windows::CurrentTasks(selected) = app.selected_window {
            Some(selected)
        } else {
            None
        },
    );
    render_completed_tasks(app, frame, task_layout[1]);
}

fn render_current_tasks<B>(
    app: &mut App,
    frame: &mut Frame<B>,
    layout_chunk: Rect,
    selected_index: Option<usize>,
) where
    B: Backend,
{
    let theme = &app.theme;
    let tasks: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let progess = Span::styled(
                if task.progress { "[-] " } else { "[ ] " },
                Style::default().fg(if selected_index == Some(i) {
                    theme.selected_task_colour
                } else {
                    Color::White
                }),
            );
            let content = Span::styled(
                task.title.as_str(),
                Style::default().fg(task.priority.get_colour(theme)),
            );
            let content = Spans::from(vec![progess, content]);
            ListItem::new(content)
        })
        .collect();

    let border_colour = match app.selected_window {
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
        if let Windows::CurrentTasks(selected) = app.selected_window {
            Some(selected)
        } else {
            None
        },
    );

    frame.render_stateful_widget(current, layout_chunk, &mut state);
}

fn render_completed_tasks<B>(app: &mut App, frame: &mut Frame<B>, layout_chunk: Rect)
where
    B: Backend,
{
    let theme = &app.theme;

    let border_colour = match app.selected_window {
        Windows::CompletedTasks(_) => theme.selected_border_colour,
        _ => theme.default_border_colour,
    };

    let completed_tasks: Vec<ListItem> = app
        .completed_tasks
        .iter()
        .enumerate()
        .map(|(ind, task)| {
            let colour = if let Windows::CompletedTasks(i) = app.selected_window {
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
                    task.time_completed.format("%-I:%M:%S %p"),
                    task.title
                ),
                Style::default().fg(colour),
            ));
            ListItem::new(content)
        })
        .collect();

    let recently_competed = List::new(completed_tasks)
        .block(
            Block::default()
                .title("Completed")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(border_colour)),
        )
        .style(Style::default().fg(Color::White));

    let mut completed_state = ListState::default();
    if !app.completed_tasks.is_empty() {
        let index = match app.selected_window {
            Windows::CompletedTasks(i) => i,
            _ => app.completed_tasks.len() - 1,
        };
        completed_state.select(Some(index));
    }

    frame.render_stateful_widget(recently_competed, layout_chunk, &mut completed_state);
}

fn render_selected_task<B>(task: &Task, theme: &Theme, frame: &mut Frame<B>, layout_chunk: Rect)
where
    B: Backend,
{
    let text = vec![
        Spans::default(),
        Spans::from(vec![
            Span::raw("Priority: "),
            Span::styled(
                task.priority.get_display_string(),
                Style::default().fg(task.priority.get_colour(theme)),
            ),
            Span::raw("."),
        ]),
        Spans::from(Span::styled("Second line", Style::default().fg(Color::Red))),
    ];
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(task.title.as_str())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        // .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, layout_chunk)
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
