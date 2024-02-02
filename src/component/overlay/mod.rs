use crossterm::event::{KeyEvent, MouseEvent};

use crate::{
    app::{Mode, ScreenManager},
    draw::{Action, PostEvent, Component},
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
            return Ok(overlay.component_mut().key_event(&mut screen_manager.app, key_event));
        }
        Ok(PostEvent {
            propegate_further: true,
            action: Action::Noop,
        })
    }

    pub fn mouse_event(screen_manager: &mut ScreenManager, mouse_event: MouseEvent) -> PostEvent {
        if let Some(overlay) = screen_manager.overlays.last_mut() {
            return overlay.component_mut().mouse_event(&mut screen_manager.app, mouse_event);
        }
        PostEvent {
            propegate_further: true,
            action: Action::Noop,
        }
    }

    pub fn prev_mode(&self) -> Option<Mode> {
        match self {
            Overlay::Fuzzy(fuzzy) => fuzzy.prev_mode,
            Overlay::Input(input) => input.prev_mode,
            Overlay::Dialog(dialog) => dialog.prev_mode,
            Overlay::Message(message) => message.prev_mode,
        }
    }
}
