use crossterm::event::{KeyEvent, MouseEvent};
use tui::prelude::Rect;

use crate::{
    app::{MainApp, Mode},
    draw::{Action, Drawer, PostEvent},
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
    pub fn key_event(main_app: &mut MainApp, key_event: KeyEvent) -> Result<PostEvent, AppError> {
        if let Some(overlay) = main_app.overlays.last_mut() {
            return match overlay {
                Overlay::Fuzzy(fuzzy) => Ok(fuzzy.key_event(&mut main_app.app, key_event)),
                Overlay::Input(input) => Ok(input.key_event(&mut main_app.app, key_event)),
                Overlay::Dialog(dialog) => Ok(dialog.key_event(&main_app.app, key_event)),
                Overlay::Message(msg) => Ok(msg.key_event(&mut main_app.app, key_event)),
            };
        }
        Ok(PostEvent {
            propegate_further: true,
            action: Action::Noop,
        })
    }

    pub fn mouse_event(main_app: &mut MainApp, mouse_event: MouseEvent) -> PostEvent {
        if let Some(overlay) = main_app.overlays.last_mut() {
            return match overlay {
                Overlay::Fuzzy(fuzzy) => fuzzy.mouse_event(&mut main_app.app, mouse_event),
                Overlay::Input(input) => input.mouse_event(&mut main_app.app, mouse_event),
                Overlay::Dialog(dialog) => dialog.mouse_event(&mut main_app.app, mouse_event),
                Overlay::Message(message) => message.mouse_event(&mut main_app.app, mouse_event),
            };
        }

        PostEvent {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    pub fn draw(&self, main_app: &MainApp, drawer: &mut Drawer) {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy.draw(&main_app.app, drawer),
            Overlay::Input(input) => input.draw(&main_app.app, drawer),
            Overlay::Dialog(dialog) => dialog.draw(&main_app.app, drawer),
            Overlay::Message(msg) => msg.draw(&main_app.app, drawer),
        }
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
    }

    pub fn prev_mode(&self) -> Option<Mode> {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy.prev_mode,
            Overlay::Input(input) => input.prev_mode,
            Overlay::Dialog(dialog) => dialog.prev_mode,
            Overlay::Message(message) => message.mode_to_restore,
        }
    }
}
