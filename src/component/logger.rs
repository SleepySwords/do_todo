use std::sync::{Arc, Mutex};

use crossterm::event::KeyCode;
use tracing::{Level, Subscriber};
use tracing_subscriber::{
    fmt::{format::Pretty, FormatFields},
    Layer,
};
use tui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Clear, List, ListState},
};

use crate::{
    framework::{
        component::{Component, Drawer},
        event::PostEvent,
    },
    utils::{self},
};

#[derive(Default, Clone)]
pub struct Logger {
    opened: bool,
    draw_area: Rect,
    logs: Arc<Mutex<Vec<String>>>,
}

impl<S: Subscriber> Layer<S> for Logger {
    fn enabled(
        &self,
        metadata: &tracing::Metadata<'_>,
        _: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        metadata.level() <= &Level::DEBUG
    }

    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        let mut log_msg = String::new();
        let _ = std::fmt::write(&mut log_msg, format_args!("{}: ", event.metadata().level()));

        let writer = tracing_subscriber::fmt::format::Writer::new(&mut log_msg);

        let pretty = Pretty::default();
        let _ = pretty.format_fields(writer, event);

        let mut logs = self.logs.lock().unwrap();
        logs.push(log_msg);
    }
}

impl Component for Logger {
    fn draw(&self, app: &crate::app::App, drawer: &mut Drawer) {
        if self.opened {
            let style = Style::default().fg(Color::Red);

            let logs = self.logs.lock().unwrap();

            let border_block = Block::default()
                .borders(Borders::ALL)
                .border_type(app.config.border_type)
                .title("Logger")
                .border_style(style);

            let rows = logs
                .iter()
                .map(|msg| {
                    utils::wrap::wrap_text(msg.as_str(), border_block.inner(self.draw_area).width)
                })
                .collect::<Vec<Text>>();

            // Add multiline support.
            let mut list_state = ListState::default();
            list_state.select_last();
            let list = List::new(rows);
            let list = list.block(border_block);
            drawer.draw_widget(Clear, self.draw_area);
            drawer.draw_stateful_widget(list, &mut list_state, self.draw_area);
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
