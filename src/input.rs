use crossterm::event::{KeyCode, KeyEvent};
use tui::style::Color;

use crate::actions;
use crate::component::completed_list::CompletedList;
use crate::component::input::dialog::DialogBox;
use crate::component::input::form::Form;
use crate::component::input::input_box::InputBox;
use crate::component::message_box::MessageBox;
use crate::component::task_list::TaskList;
use crate::{
    app::{App, SelectedComponent, UserInputType},
    task::Task,
};

// PERF: Maybe we'll do a Component system if there is a way?

pub fn handle_key(key_event: KeyEvent, app: &mut App) {
    let key_code = key_event.code;
    if let Some(component) = app.popup_stack.last() {
        match component {
            UserInputType::InputBox(_) => {
                // TODO: more generalised error handling
                let err = InputBox::handle_event(app, key_code);
                if err.is_err() {
                    app.append_layer(UserInputType::MessageBox(MessageBox::new(
                        String::from("Error"),
                        err.err().unwrap().to_string(),
                        Color::Red,
                    )))
                }
            }
            UserInputType::DialogBox(_) => DialogBox::handle_event(app, key_code),
            UserInputType::MessageBox(_) => MessageBox::handle_event(app, key_code),
            UserInputType::Form(_) => Form::handle_event(app, key_code),
        }
        return;
    }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => app.append_layer(UserInputType::InputBox(InputBox::new(
            String::from("Add a task"),
            |app, mut word| {
                app.task_store.tasks.push(Task::from_string(
                    word.drain(..).collect::<String>().trim().to_string(),
                ));
                Ok(())
            },
        ))),
        KeyCode::Char('1') => app.selected_component = SelectedComponent::CurrentTasks,
        KeyCode::Char('2') => app.selected_component = SelectedComponent::CompletedTasks,
        KeyCode::Char('x') => actions::open_help_menu(app),
        KeyCode::Char('q') => app.shutdown(),
        _ => {}
    }

    match app.selected_component {
        SelectedComponent::CurrentTasks => TaskList::handle_event(app, key_event),
        SelectedComponent::CompletedTasks => CompletedList::handle_event(app, key_code),
        SelectedComponent::PopUpComponent => None,
    };
}
