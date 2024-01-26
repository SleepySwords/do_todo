use crossterm::event::{KeyEvent, MouseEvent};
use tui::prelude::Rect;

use crate::{
    app::{ScreenManager, Mode},
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
    pub fn key_event(screen_manager: &mut ScreenManager, key_event: KeyEvent) -> Result<PostEvent, AppError> {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return match overlay {
                Overlay::Fuzzy(fuzzy) => Ok(fuzzy.key_event(&mut screen_manager.app, key_event)),
                Overlay::Input(input) => Ok(input.key_event(&mut screen_manager.app, key_event)),
                Overlay::Dialog(dialog) => Ok(dialog.key_event(&screen_manager.app, key_event)),
                Overlay::Message(msg) => Ok(msg.key_event(&mut screen_manager.app, key_event)),
            };
        }
        Ok(PostEvent {
            propegate_further: true,
            action: Action::Noop,
        })
    }

    pub fn mouse_event(screen_manager: &mut ScreenManager, mouse_event: MouseEvent) -> PostEvent {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return match overlay {
                Overlay::Fuzzy(fuzzy) => fuzzy.mouse_event(&mut screen_manager.app, mouse_event),
                Overlay::Input(input) => input.mouse_event(&mut screen_manager.app, mouse_event),
                Overlay::Dialog(dialog) => dialog.mouse_event(&mut screen_manager.app, mouse_event),
                Overlay::Message(message) => message.mouse_event(&mut screen_manager.app, mouse_event),
            };
        }

        PostEvent {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    pub fn draw(&self, screen_manager: &ScreenManager, drawer: &mut Drawer) {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy.draw(&screen_manager.app, drawer),
            Overlay::Input(input) => input.draw(&screen_manager.app, drawer),
            Overlay::Dialog(dialog) => dialog.draw(&screen_manager.app, drawer),
            Overlay::Message(msg) => msg.draw(&screen_manager.app, drawer),
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
