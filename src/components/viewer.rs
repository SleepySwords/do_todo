use tui::{
    layout::Constraint,
    style::Style,
    text::{Span, Text},
    widgets::{Block, Borders, Cell, Row, Table},
};

// A viewer of a task/something
use crate::{input::Component, task::Task};

struct Viewer;

impl Component for Viewer {
    fn handle_event(
        &mut self,
        _app: &mut crate::app::App,
        _key_code: crossterm::event::KeyCode,
    ) -> Option<()> {
        todo!()
    }

    fn draw<B: tui::backend::Backend>(
        &self,
        app: &crate::app::App,
        layout_chunk: tui::layout::Rect,
        f: &mut tui::Frame<B>,
    ) {
        let theme = &app.theme;
        let task = Task::from_string("aaa".to_string());
        let constraints = &[Constraint::Percentage(20), Constraint::Percentage(80)];

        let items = vec![
            (Span::raw("Title"), &task.title as &str, Style::default()),
            (
                Span::raw("Priority"),
                task.priority.display_string(),
                Style::default().fg(task.priority.colour(theme)),
            ),
        ];

        let rows = items.iter().map(|item| {
            let text = textwrap::fill(
                item.1,
                constraints[1].apply(layout_chunk.width) as usize - 2,
            );
            let height = text.chars().filter(|c| *c == '\n').count() + 1;
            // Clone (actually crying tho)
            let cells = vec![
                Cell::from(item.0.to_owned()),
                Cell::from(Text::styled(text, item.2)),
            ];
            Row::new(cells).height(height as u16).bottom_margin(1)
        });

        let rows = Table::new(rows)
            .block(
                Block::default()
                    .title("Task information")
                    .borders(Borders::ALL)
                    .border_type(theme.border_style.border_type),
            )
            .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)]);
        // .alignment(Alignment::Center)
        // .wrap(Wrap { trim: true });

        f.render_widget(rows, layout_chunk)
    }
}
