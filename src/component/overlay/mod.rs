use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    error::AppError, framework::{component::Component, event::PostEvent, screen_manager::ScreenManager},
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
    pub fn component(&self) -> &dyn Component {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy,
            Overlay::Input(input) => input,
            Overlay::Dialog(dialog) => dialog,
            Overlay::Message(message) => message,
        }
    }

    pub fn component_mut(&mut self) -> &mut dyn Component {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy,
            Overlay::Input(input) => input,
            Overlay::Dialog(dialog) => dialog,
            Overlay::Message(message) => message,
        }
    }

    pub fn key_event(
        screen_manager: &mut ScreenManager,
        key_event: KeyEvent,
    ) -> Result<PostEvent, AppError> {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return Ok(overlay
                .component_mut()
                .key_event(&mut screen_manager.app, key_event));
        }
        Ok(PostEvent::noop(true))
    }

    pub fn mouse_event(screen_manager: &mut ScreenManager, mouse_event: MouseEvent) -> PostEvent {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return overlay
                .component_mut()
                .mouse_event(&mut screen_manager.app, mouse_event);
        }
        PostEvent::noop(true)
    }
}
