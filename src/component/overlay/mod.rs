use crossterm::event::{KeyEvent, MouseEvent};
use tui::prelude::Rect;

use crate::{
    app::App,
    draw::{Action, Drawer, PostAction},
    error::AppError,
};

use self::{dialog::DialogBox, fuzzy::FuzzyBox, input_box::InputBox};

use super::message_box::MessageBox;

pub mod dialog;
pub mod fuzzy;
pub mod input_box;

pub enum Overlay<'a> {
    Fuzzy(FuzzyBox<'a>),
    Input(InputBox),
    Dialog(DialogBox<'a>),
    Message(MessageBox),
}

impl Overlay<'_> {
    pub fn key_event(app: &mut App, key_event: KeyEvent) -> Result<PostAction, AppError> {
        // FIXME: This does not actually consider what the action is...
        if !FuzzyBox::key_event(app, key_event).propegate_further
            || !InputBox::key_event(app, key_event).propegate_further
            || !DialogBox::key_event(app, key_event).propegate_further
            || !MessageBox::key_event(app, key_event).propegate_further
        {
            return Ok(PostAction {
                propegate_further: false,
                action: Action::Noop,
            });
        }
        Ok(PostAction {
            propegate_further: true,
            action: Action::Noop,
        })
    }

    pub fn mouse_event(app: &mut App, mouse_event: MouseEvent) -> PostAction {
        if !FuzzyBox::mouse_event(app, mouse_event).propegate_further
            || !InputBox::mouse_event(app, mouse_event).propegate_further
            || !DialogBox::mouse_event(app, mouse_event).propegate_further
            || !MessageBox::mouse_event(app, mouse_event).propegate_further
        {
            return PostAction {
                propegate_further: false,
                action: Action::Noop,
            };
        }
        PostAction {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    pub fn draw(&self, app: &App, drawer: &mut Drawer) {
        FuzzyBox::draw(app, drawer);
        InputBox::draw(app, drawer);
        DialogBox::draw(app, drawer);
        MessageBox::draw(app, drawer);
    }

    pub fn update_layout(&mut self, draw_area: Rect) {
        match self {
            Overlay::Input(input) => {
                input.update_layout(draw_area);
            }
            Overlay::Dialog(dialog) => dialog.update_layout(draw_area),
            Overlay::Fuzzy(fuzzy) => {
                fuzzy.update_layout(draw_area);
            }
            Overlay::Message(message) => message.update_layout(draw_area),
        }
        // FuzzyBox::update_layout(app, key_event)
    }
}
