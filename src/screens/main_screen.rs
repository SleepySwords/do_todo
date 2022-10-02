use crate::{
    app::{App, PopUpComponents},
    component::{completed_list::CompletedList, input_box, task_list::TaskList, viewer::Viewer},
    utils,
};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub struct MainScreenLayer;

impl MainScreenLayer {
    fn draw_tasks<B>(app: &App, frame: &mut Frame<B>, layout_chunk: Rect)
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

    pub fn draw<B: Backend>(app: &mut App, f: &mut Frame<B>) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Min(1), Constraint::Length(1)])
            .split(f.size());

        let main_body = layout[0];
        let status_line = layout[1];

        app.status_line.draw(app, status_line, f);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(main_body);

        Viewer::draw(app, chunks[1], f);
        Self::draw_tasks(app, f, chunks[0]);

        if let Some(component) = app.popup_stack.last() {
            match component {
                PopUpComponents::InputBox(component) => {
                    let layout_chunk = utils::centered_rect(
                        Constraint::Percentage(70),
                        Constraint::Length(
                            (component.words.len() as u16).max(1) + input_box::PADDING as u16,
                        ),
                        f.size(),
                    );
                    component.draw(app, layout_chunk, f)
                }
                PopUpComponents::DialogBox(component) => {
                    let layout_chunk = utils::centered_rect(
                        Constraint::Percentage(70),
                        Constraint::Length(component.options.len() as u16 + 2),
                        f.size(),
                    );
                    component.draw(app, layout_chunk, f)
                }
                PopUpComponents::MessageBox(component) => {
                    let layout_chunk = utils::centered_rect(
                        Constraint::Percentage(70),
                        Constraint::Percentage(30),
                        f.size(),
                    );
                    component.draw(app, layout_chunk, f)
                }
            }
        }
    }
}
