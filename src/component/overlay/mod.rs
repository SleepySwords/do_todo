use crossterm::event::{KeyEvent, KeyEventKind, MouseEvent};
use tui::prelude::Rect;

use crate::{
    app::App,
    draw::{Drawer, EventResult},
};

use self::{dialog::DialogBox, fuzzy::FuzzyBox, input_box::InputBox};

use super::message_box::MessageBox;

pub mod dialog;
pub mod fuzzy;
pub mod input_box;

pub enum Overlay<'a> {
    FuzzyBox(FuzzyBox<'a>),
    InputBox(InputBox),
    DialogBox(DialogBox<'a>),
    MessageBox(MessageBox),
}

impl Overlay<'_> {
    pub fn key_event(app: &mut App, key_event: KeyEvent) -> EventResult {
        if FuzzyBox::key_event(app, key_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if InputBox::key_event(app, key_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if DialogBox::key_event(app, key_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if MessageBox::key_event(app, key_event) != EventResult::Ignored {
            return EventResult::Consumed;
        }
        EventResult::Ignored
    }

    pub fn mouse_event(app: &mut App, mouse_event: MouseEvent) -> EventResult {
        if FuzzyBox::mouse_event(app, mouse_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if InputBox::mouse_event(app, mouse_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if DialogBox::mouse_event(app, mouse_event) != EventResult::Ignored {
            return EventResult::Consumed;
        } else if MessageBox::mouse_event(app, mouse_event) != EventResult::Ignored {
            return EventResult::Consumed;
        }
        EventResult::Ignored
    }

    pub fn draw(&self, app: &App, drawer: &mut Drawer) {
        FuzzyBox::draw(app, drawer);
        InputBox::draw(app, drawer);
        DialogBox::draw(app, drawer);
        MessageBox::draw(app, drawer);
    }

    pub fn update_layout(&mut self, draw_area: Rect) {
        match self {
            Overlay::InputBox(input) => {
                input.update_layout(draw_area);
            }
            Overlay::DialogBox(dialog) => dialog.update_layout(draw_area),
            Overlay::FuzzyBox(fuzzy) => {
                fuzzy.update_layout(draw_area);
            }
            Overlay::MessageBox(message) => message.update_layout(draw_area),
        }
        // FuzzyBox::update_layout(app, key_event)
    }
}
