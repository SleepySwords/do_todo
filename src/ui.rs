pub mod popup;

use crate::{
    app::{App, Mode, Windows},
    task::Task,
    theme::Theme,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table,
    },
    Frame,
};

pub fn render_ui<B: Backend>(app: &mut App, f: &mut Frame<B>) {
    let menu = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let help = Text::styled(
        "Press q to exit.",
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let paragraph = Paragraph::new(help);

    f.render_widget(paragraph, menu[1]);

    match app.selected_window {
        Windows::CurrentTasks(i) => {
            if !app.tasks.is_empty() {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(menu[0]);
                render_tasks(app, f, chunks[0]);
                render_selected_task(&app.tasks[i], &app.theme, f, chunks[1]);
            } else {
                render_tasks(app, f, menu[0]);
            }
        }
        _ => render_tasks(app, f, menu[0]),
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

    if let Mode::Add = app.mode {
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

    if let Mode::Delete(task_index, index) = app.mode {
        let text = vec![
            ListItem::new(Spans::from("Delete")),
            ListItem::new(Spans::from("Cancel")),
        ];
        let help_message = List::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Rounded)
                    .title(format!("Delete the task {}", app.tasks[task_index].title)),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));

        let mut state = ListState::default();
        state.select(Some(index));

        let area = centered_rect(70, 20, f.size());
        f.render_widget(Clear, area);
        f.render_stateful_widget(help_message, area, &mut state);
    }
}

fn render_tasks<B>(app: &mut App, frame: &mut Frame<B>, layout_chunk: Rect)
where
    B: Backend,
{
    let layout_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(layout_chunk);

    render_current_tasks(
        app,
        frame,
        layout_chunk[0],
        if let Windows::CurrentTasks(selected) = app.selected_window {
            Some(selected)
        } else {
            None
        },
    );

    render_completed_tasks(app, frame, layout_chunk[1])
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
    let constraints = &[Constraint::Percentage(20), Constraint::Percentage(80)];

    let items = vec![
        (Span::raw("Title"), &task.title as &str, Style::default()),
        (
            Span::raw("Priority"),
            task.priority.get_display_string(),
            Style::default().fg(task.priority.get_colour(theme)),
        ),
    ];

    let rows = items.iter().map(|item| {
        let text = textwrap::fill(&item.1, constraints[1].apply(layout_chunk.width) as usize - 2);
        let height = text
            .chars()
            .filter(|c| *c == '\n').count()
            + 1;
        // To owned is not that efficient is it
        let cells = vec![Cell::from(item.0.to_owned()), Cell::from(Text::styled(text, item.2))];
        Row::new(cells).height(height as u16).bottom_margin(1)
    });

    let rows = Table::new(rows)
        .block(
            Block::default()
                .title(task.title.as_str())
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);
    // .alignment(Alignment::Center)
    // .wrap(Wrap { trim: true });

    frame.render_widget(rows, layout_chunk)
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
