use crossterm::event::KeyCode;
use tui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, Row, Table, TableState},
};

use crate::{
    framework::{
        component::{Component, Drawer},
        event::PostEvent,
    },
    utils::{self, ui::generate_table},
};

#[derive(Default)]
pub struct Logger {
    opened: bool,
    draw_area: Rect,
}

impl Component for Logger {
    fn draw(&self, app: &crate::app::App, drawer: &mut Drawer) {
        if self.opened {
            let style = Style::default().fg(Color::Red);
            let rows = app
                .logs
                .iter()
                .map(|(msg, time)| (time.to_string().into(), Line::raw(msg)))
                .collect::<Vec<(Span, Line)>>();

            let border_block = Block::default()
                .borders(Borders::ALL)
                .border_type(app.config.border_type)
                .title("Logger")
                .border_style(style);

            // Add multiline support.
            let table = generate_table(rows, border_block.inner(self.draw_area).width as usize);
            let table = table.block(border_block);
            let mut table_state = TableState::default();
            if !app.logs.is_empty() {
                table_state.select(Some(app.logs.len() - 1));
            }
            drawer.draw_widget(Clear, self.draw_area);
            drawer.draw_stateful_widget(table, &mut table_state, self.draw_area);
        }
    }

    fn key_event(
        &mut self,
        _: &mut crate::app::App,
        key_event: crossterm::event::KeyEvent,
    ) -> PostEvent {
        let key_code = key_event.code;
        if self.opened {
            self.opened = false;
            return PostEvent::noop(false);
        }
        if key_code == KeyCode::Char('-') {
            self.opened = true;
            return PostEvent::noop(false);
        }
        PostEvent::noop(true)
    }

    fn update_layout(&mut self, draw_area: tui::layout::Rect) {
        self.draw_area = utils::centre_rect(
            tui::layout::Constraint::Percentage(70),
            tui::layout::Constraint::Percentage(70),
            draw_area,
        );
    }
}
