use crossterm::event::{KeyCode, KeyEvent};
use tui::style::Color;

use crate::actions;
use crate::component::completed_list::CompletedList;
use crate::component::input::dialog::DialogBox;
use crate::component::input::input_box::InputBox;
use crate::component::message_box::MessageBox;
use crate::component::task_list::TaskList;
use crate::{
    app::{App, SelectedComponent},
    task::Task,
};

pub fn handle_key(key_event: KeyEvent, app: &mut App) {
    //     let key_code = key_event.code;
    //     if let Some(component) = app.popup_stack.last() {
    //         let err = match component {
    //             UserInputType::Input(_) => InputBox::handle_event(app, key_code),
    //             UserInputType::Dialog(_) => DialogBox::handle_event(app, key_code),
    //             UserInputType::Message(_) => MessageBox::handle_event(app, key_code),
    //         };
    //         if err.is_err() {
    //             app.append_layer(UserInputType::Message(MessageBox::new(
    //                 String::from("Error"),
    //                 err.err().unwrap().to_string(),
    //                 Color::Red,
    //             )))
    //         }
    //         return;
    //     }

    //     match app.selected_component {
    //         SelectedComponent::CurrentTasks => TaskList::handle_event(app, key_event),
    //         SelectedComponent::CompletedTasks => CompletedList::handle_event(app, key_code),
    //         SelectedComponent::PopUpComponent => None,
    //     };

    //     // Universal keyboard shortcuts (should also be customisable)
    //     match key_code {
    //         KeyCode::Char('a') => app.append_layer(UserInputType::Input(InputBox::new(
    //             String::from("Add a task"),
    //             |app, mut word| {
    //                 app.task_store.tasks.push(Task::from_string(
    //                     word.drain(..).collect::<String>().trim().to_string(),
    //                 ));
    //                 Ok(())
    //             },
    //         ))),
    //         KeyCode::Char('1') => app.selected_component = SelectedComponent::CurrentTasks,
    //         KeyCode::Char('2') => app.selected_component = SelectedComponent::CompletedTasks,
    //         KeyCode::Char('x') => actions::open_help_menu(app),
    //         KeyCode::Char('q') => app.shutdown(),
    //         _ => {}
    //     }
}
