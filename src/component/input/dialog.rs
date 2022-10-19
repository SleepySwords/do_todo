use crossterm::event::KeyCode;

use tui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::{
    app::{App, UserInputType},
    utils,
};

pub struct DialogAction {
    name: String,
    function: Box<dyn Fn(&mut App)>,
}

impl DialogAction {
    pub fn new<F: 'static>(name: String, function: F) -> DialogAction
    where
        F: Fn(&mut App),
    {
        DialogAction {
            name,
            function: Box::new(function),
        }
    }
}
pub struct DialogBox {
    title: String,
    index: usize,
    pub options: Vec<DialogAction>,
}

impl DialogBox {
    pub fn new(title: String, options: Vec<DialogAction>) -> DialogBox {
        if options.is_empty() {
            panic!("The size of the options is 0");
        }
        DialogBox {
            title,
            index: 0,
            options,
        }
    }
}

impl DialogBox {
    pub fn handle_event(app: &mut App, key_code: KeyCode) {
        // TODO: This is somewhat ugly, and pretty weird to get.
        let context = if let Some(UserInputType::Dialog(context)) = app.popup_context_mut() {
            context
        } else {
            return;
        };
        if let KeyCode::Char(char) = key_code {
            if char == 'q' {
                return;
            }
        }
        utils::handle_movement(key_code, &mut context.index, context.options.len());
        match key_code {
            KeyCode::Enter => {
                if let Some(UserInputType::Dialog(context)) = app.pop_popup() {
                    (context.options[context.index].function)(app);
                }
            }
            KeyCode::Esc => {
                // May be better to have a custom escape function
                app.pop_popup();
            }
            _ => {}
        }
    }

    pub fn draw<B: tui::backend::Backend>(
        &self,
        app: &App,
        draw_area: Rect,
        f: &mut tui::Frame<B>,
    ) {
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

        f.render_widget(Clear, draw_area);
        f.render_stateful_widget(list, draw_area, &mut list_state);
    }
}
