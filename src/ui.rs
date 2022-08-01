use crate::{
    app::{App, PopUpComponents},
    component::{
        completed_list::CompletedList, status_line::StatusLineComponent, task_list::TaskList,
        viewer::Viewer,
    },
    utils,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub fn render_ui<B: Backend>(app: &mut App, f: &mut Frame<B>) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
        .split(f.size());

    let main_body = layout[0];
    let status_line = layout[1];

    StatusLineComponent::new(String::from("Press x for help. Press q to exit.")).draw(
        app,
        status_line,
        f,
    );
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_body);

    Viewer::draw(app, chunks[1], f);
    render_tasks(app, f, chunks[0]);

    if let Some(component) = app.popup_stack.last() {
        let area = utils::centered_rect(
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            f.size(),
        );
        match component {
            PopUpComponents::InputBox(component) => component.draw(app, area, f),
            PopUpComponents::DialogBox(component) => component.draw(app, area, f),
        }
    }
}

fn render_tasks<B>(app: &App, frame: &mut Frame<B>, layout_chunk: Rect)
where
    B: Backend,
{
    let layout_chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(layout_chunk);

    TaskList::draw(app, layout_chunk[0], frame);

    CompletedList::draw(app, layout_chunk[1], frame)
}
