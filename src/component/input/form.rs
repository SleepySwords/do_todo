use crossterm::event::KeyCode;

use crate::app::App;

pub struct Form {
    // name: String,
    // selected_input: usize,
    // inputs: Vec<UserInputType>,
}

impl Form {
    pub fn handle_event(_app: &mut App, _key_code: KeyCode) {
        // let context = if let Some(UserInputComponents::Form(context)) = app.popup_context() {
        //     context
        // } else {
        //     return
        // };
        // context.inputs[context.selected_input].handle_event(app, key_code);
    }
}
