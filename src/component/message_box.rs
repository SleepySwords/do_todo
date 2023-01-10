use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, Clear, List, ListItem, ListState};

use crate::app::App;
use crate::utils::centre_rect;
use crate::view::DrawableComponent;

pub struct MessageBox {
    title: String,
    callback: Box<dyn Fn(&mut App)>,
    message: Vec<String>,
    colour: Color,
    selected_index: usize,
}

impl MessageBox {
    pub fn new<T: Fn(&mut App) + 'static>(
        title: String,
        callback: T,
        words: String,
        colour: Color,
        selected_index: usize,
    ) -> MessageBox {
        MessageBox {
            title,
            callback: Box::new(callback),
            message: words
                .split('\n')
                .map(|f| f.to_string())
                .collect::<Vec<String>>(),
            colour,
            selected_index,
        }
    }

    pub fn new_by_list<T: Fn(&mut App) + 'static>(
        title: String,
        callback: T,
        words: Vec<String>,
        colour: Color,
    ) -> MessageBox {
        MessageBox {
            title,
            callback: Box::new(callback),
            message: words,
            colour,
            selected_index: 0,
        }
    }
}

impl DrawableComponent for MessageBox {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut crate::view::Drawer) {
        let style = Style::default().fg(self.colour);
        let text = self
            .message
            .iter()
            .map(|msg| ListItem::new(Span::styled(msg, style)))
            .collect::<Vec<ListItem>>();
        // Add multiline support.
        let height = ((text.len() + 2) as u16).min(Constraint::Percentage(70).apply(app.app_size.height));
        let list = List::new(text);
        let list = list.block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(app.theme.border_style.border_type)
                .border_style(style)
                .title(self.title.as_ref()),
        );
        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_index));
        let draw_area = centre_rect(
            Constraint::Percentage(70),
            Constraint::Length(height),
            draw_area,
        );
        drawer.draw_widget(Clear, draw_area);
        drawer.draw_stateful_widget(list, &mut list_state, draw_area);
    }

    fn key_pressed(
        &mut self,
        app: &mut App,
        _: crossterm::event::KeyCode,
    ) -> crate::view::EventResult {
        app.pop_layer();
        (self.callback)(app);
        crate::view::EventResult::Consumed
    }
}
