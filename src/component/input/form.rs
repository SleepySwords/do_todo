use crossterm::event::KeyCode;

use crate::app::{App, UserInputType};

pub struct Form {
    name: String,
    selected_input: usize,
    inputs: Vec<UserInputType>,
}

impl Form {
    pub fn handle_event(app: &mut App, key_code: KeyCode) {
        let context = if let Some(UserInputType::Form(context)) = app.popup_context() {
            context
        } else {
            return
        };
        context.inputs[context.selected_input].handle_event(app, key_code);
    }

}
