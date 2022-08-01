use crossterm::event::KeyCode;

use tui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::App,
    utils::{self, centered_rect},
};

pub struct Action {
    name: String,
    function: Box<dyn Fn(&mut App)>,
}

impl Action {
    pub fn new<F: 'static>(name: String, function: F) -> Action
    where
        F: Fn(&mut App),
    {
        Action {
            name,
            function: Box::new(function),
        }
    }
}
pub struct DialogComponent {
    title: String,
    index: usize,
    options: Vec<Action>,
}

impl DialogComponent {
    pub fn new(title: String, options: Vec<Action>) -> DialogComponent {
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

impl DialogComponent {
    pub fn handle_event(&mut self, app: &mut App, key_code: KeyCode) -> Option<()> {
        utils::handle_movement(key_code, &mut self.index, self.options.len());
        match key_code {
            KeyCode::Enter => {
                (self.options[self.index].function)(app);
                // app.popup_stack.retain(|x| x != PopUpComponents::DialogBox(self));
                return None;
            }
            KeyCode::Esc => {
                // May be better to have a custom escape function
                return None;
            }
            _ => {}
        }
        Some(())
    }

    pub fn draw<B: tui::backend::Backend>(&self, app: &App, _: Rect, f: &mut tui::Frame<B>) {
        let area = centered_rect(
            Constraint::Percentage(70),
            Constraint::Length(self.options.len() as u16 + 2),
            f.size(),
        );

        // Clone is not the best :(
        let list = List::new(
            self.options
                .iter()
                .map(|action| ListItem::new(Spans::from(action.name.clone())))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .title(self.title.clone())
                .border_style(Style::default().fg(tui::style::Color::Green)),
        );

        let mut list_state = ListState::default();
        list_state.select(Some(self.index));

        f.render_widget(Clear, area);
        f.render_stateful_widget(list, area, &mut list_state);
    }
}
