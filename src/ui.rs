use crate::app::App;
use crate::task::Task;
use chrono::prelude::Local;
use crossterm::event::{self, Event, KeyCode};
use std::io;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

pub fn ui<B: Backend>(app: &mut App, f: &mut Frame<B>) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.size());
    f.render_widget(Block::default().title("Todo!!!\n".to_string()), chunks[0]);

    let task_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(chunks[1]);

    render_tasks(app, f, task_layout[0]);
    render_completed_tasks(app, f, task_layout[1]);

    if app.add_mode {
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

pub fn run_app<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> io::Result<()> {
    loop {
        terminal.draw(|mut f| ui(app, &mut f))?;

        // This function blocks
        if let Event::Key(key) = event::read()? {
            if app.add_mode {
                match key.code {
                    KeyCode::Char(c) => app.words.push(c),
                    KeyCode::Backspace => {
                        app.words.pop();
                    }
                    KeyCode::Enter => {
                        app.tasks.push(Task::new(app.words.drain(..).collect()));
                        app.add_mode = !app.add_mode
                    }
                    KeyCode::Esc => app.add_mode = !app.add_mode,
                    _ => {}
                }
                continue;
            }
            match key.code {
                KeyCode::Char('a') => app.add_mode = !app.add_mode,
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('j') => {
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    if app.selected_index == app.tasks.len() - 1 {
                        app.selected_index = 0;
                    } else {
                        app.selected_index += 1;
                    }
                }
                KeyCode::Char('d') => {
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    app.tasks.remove(app.selected_index);
                    if app.selected_index == app.tasks.len() && app.tasks.len() != 0 {
                        app.selected_index -= 1;
                    }
                }
                KeyCode::Char('h') => {
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    app.tasks[app.selected_index].priority =
                        app.tasks[app.selected_index].priority.get_next();
                }
                KeyCode::Char('p') => {
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    app.tasks[app.selected_index].progress =
                        !app.tasks[app.selected_index].progress;
                }
                KeyCode::Char('c') => {
                    let local = Local::now();
                    let time = local.time().format("%-I:%M:%S %p").to_string();
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    let mut task = app.tasks.remove(app.selected_index);
                    task.content = format!("{} {}", time, task.content);
                    app.completed_tasks.push(task);
                    if app.selected_index == app.tasks.len() && app.tasks.len() != 0 {
                        app.selected_index -= 1;
                    }
                }
                KeyCode::Char('k') => {
                    if app.tasks.len() == 0 {
                        continue;
                    }
                    if app.selected_index == 0 {
                        app.selected_index = app.tasks.len() - 1;
                    } else {
                        app.selected_index -= 1;
                    }
                }
                _ => {}
            }
        }
    }
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

fn render_tasks<B>(app: &mut App, frame: &mut Frame<B>, chunk: Rect)
where
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
                Style::default().fg(task.priority.get_colour()),
            );
            let content = Span::styled(
                task.content.as_str(),
                Style::default().fg(if i == app.selected_index {
                    theme.selected_colour
                } else {
                    Color::White
                }),
            );
            let content = Spans::from(vec![progess, content]);
            ListItem::new(content)
        })
        .collect();

    let current = List::new(tasks).block(
        Block::default()
            .title("Current List")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.default_colour)),
    );

    let mut state = ListState::default();
    state.select(Some(app.selected_index));

    frame.render_stateful_widget(current, chunk, &mut state);
}

fn render_completed_tasks<B>(app: &mut App, frame: &mut Frame<B>, layout: Rect)
where
    B: Backend,
{
    let theme = &app.theme;

    let completed_tasks: Vec<ListItem> = app
        .completed_tasks
        .iter()
        .map(|task| {
            let content = Spans::from(Span::raw(format!("{}", task.content)));
            ListItem::new(content)
        })
        .collect();

    let recently_competed = List::new(completed_tasks)
        .block(
            Block::default()
                .title("Completed")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.default_colour)),
        )
        .style(Style::default().fg(Color::White));

    let mut completed_state = ListState::default();
    if app.completed_tasks.len() != 0 {
        completed_state.select(Some(app.completed_tasks.len() - 1));
    }

    frame.render_stateful_widget(recently_competed, layout, &mut completed_state);
}
