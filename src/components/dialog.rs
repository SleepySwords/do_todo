use crossterm::event::KeyCode;

use tui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Spans,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState},
};

use crate::{app::App, input::Component, utils::centered_rect};

pub type DialogOption = (String, Box<dyn Fn(&mut App)>);

pub struct DialogComponent {
    title: String,
    index: usize,
    options: Vec<DialogOption>,
}

impl DialogComponent {
    pub fn new(title: String, options: Vec<DialogOption>) -> DialogComponent {
        if options.is_empty() {
            panic!("The size of the options is 0");
        }
        DialogComponent {
            title,
            index: 0,
            options,
        }
    }
}

impl Component for DialogComponent {
    fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()> {
        match key_code {
            KeyCode::Enter => {
                self.options[self.index].1(app);
                return None;
            }
            KeyCode::Char(char) => {
                if char == 'j' {
                    if self.index == self.options.len() - 1 {
                        self.index = 0;
                    } else {
                        self.index += 1;
                    }
                }
                if char == 'k' {
                    if self.index == 0 {
                        self.index = self.options.len() - 1;
                    } else {
                        self.index -= 1;
                    }
                }
            }
            KeyCode::Esc => {
                // May be better to have a custom escape function
                return None;
            }
            _ => {}
        }
        Some(())
    }

    fn draw<B: tui::backend::Backend>(&self, _: &App, _: Rect, f: &mut tui::Frame<B>) {
        let area = centered_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            f.size(),
        );

        // Clone is not the best :(
        let list = List::new(
            self.options
                .iter()
                .map(|(name, _)| ListItem::new(Spans::from(name.clone())))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(self.title.clone())
                .border_style(Style::default().fg(tui::style::Color::Green)),
        );

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        f.render_widget(Clear, area);
        f.render_stateful_widget(list, area, &mut list_state);
    }
}
