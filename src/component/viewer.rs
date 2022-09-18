use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders},
    Frame,
};

// A viewer of a task/something
use crate::{
    app::{App, SelectedComponent},
    utils,
};

#[derive(Default)]
pub struct Viewer;

impl Viewer {
    pub fn draw<B: tui::backend::Backend>(
        app: &App,
        layout_chunk: tui::layout::Rect,
        f: &mut tui::Frame<B>,
    ) {
        let theme = &app.theme;
        let block = Block::default()
            .title("Task information")
            .borders(Borders::ALL)
            .border_type(theme.border_style.border_type);

        match app.selected_component {
            SelectedComponent::CurrentTasks => {
                if !app.task_data.tasks.is_empty() {
                    draw_task_viewer(app, block, layout_chunk, f)
                } else {
                    f.render_widget(block, layout_chunk);
                }
            }
            SelectedComponent::CompletedTasks => {
                if !app.task_data.completed_tasks.is_empty() {
                    draw_completed_task_viewer(app, block, layout_chunk, f)
                } else {
                    f.render_widget(block, layout_chunk);
                }
            }
            SelectedComponent::PopUpComponent => {
                f.render_widget(block, layout_chunk);
            }
        }
    }
}

fn draw_task_viewer<B: Backend>(app: &App, block: Block, layout_chunk: Rect, f: &mut Frame<B>) {
    let theme = &app.theme;
    let task = &app.task_data.tasks[app.selected_task_index];

    let constraints = [Constraint::Percentage(20), Constraint::Percentage(80)];
    let tags_name = if task.tags.is_empty() {
        Spans::from("None")
    } else {
        let size = task.tags.len();
        let spans = task
            .tags
            .iter()
            .enumerate()
            .fold(Vec::new(), |mut acc, (i, tag)| {
                let colour = Style::default().fg(app.task_data.tags[tag].colour);
                let name = app.task_data.tags[tag].name.to_owned();
                acc.push(Span::styled(name, colour));
                if i != size - 1 {
                    acc.push(Span::raw(", "));
                }
                acc
            });
        Spans::from(spans)
    };

    let items = vec![
        (Span::raw("Title"), Spans::from(&task.title as &str)),
        (
            Span::raw("Priority"),
            Spans::from(Span::styled(
                task.priority.display_string(),
                Style::default().fg(task.priority.colour(theme)),
            )),
        ),
        (Span::raw("Tags"), tags_name),
    ];

    let table = utils::generate_table(items, constraints[1].apply(layout_chunk.width) as usize - 2)
        .block(block)
        .widths(&constraints);

    f.render_widget(table, layout_chunk)
}

fn draw_completed_task_viewer<B: Backend>(
    app: &App,
    block: Block,
    layout_chunk: Rect,
    f: &mut Frame<B>,
) {
    let task = &app.task_data.completed_tasks[app.selected_completed_task_index];
    let completed_time = app.task_data.completed_tasks[app.selected_completed_task_index]
        .time_completed
        .format("%d/%m/%y %-I:%M:%S %p")
        .to_string();

    let constraints = [Constraint::Percentage(25), Constraint::Percentage(75)];
    let items = vec![
        (Span::raw("Title"), Spans::from(&task.title as &str)),
        (
            Span::raw("Date Completed"),
            Spans::from(&completed_time as &str),
        ),
    ];

    let table = utils::generate_table(items, constraints[1].apply(layout_chunk.width) as usize - 2)
        .block(block)
        .widths(&constraints);

    f.render_widget(table, layout_chunk)
}
