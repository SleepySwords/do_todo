use crossterm::event::{KeyCode, KeyEvent};

use crate::actions;
use crate::component::completed_list::CompletedList;
use crate::component::input_box::InputBoxComponent;
use crate::component::task_list::TaskList;
use crate::{
    app::{App, PopUpComponents, SelectedComponent},
    task::Task,
};

// Maybe we'll do a Component system if there is a way?

pub fn handle_input(key_event: KeyEvent, app: &mut App) {
    // Popping off the stack and the pushing back on is pretty jank just to avoid the errors from
    // borrow checker
    let key_code = key_event.code;
    if let Some(component) = app.popup_stack.pop() {
        match component {
            PopUpComponents::InputBox(mut component) => {
                if component.handle_event(app, key_code).is_none() {
                    return;
                }
                app.popup_stack.push(PopUpComponents::InputBox(component));
            }
            PopUpComponents::DialogBox(mut component) => {
                if component.handle_event(app, key_code).is_none() {
                    return;
                }
                if let KeyCode::Char(char) = key_code {
                    if char == 'q' {
                        return;
                    }
                }
                app.popup_stack.push(PopUpComponents::DialogBox(component));
            }
        }
        return;
    }

    // Universal keyboard shortcuts (should also be customisable)
    match key_code {
        KeyCode::Char('a') => {
            app.popup_stack
                .push(PopUpComponents::InputBox(InputBoxComponent::new(
                    String::from("Add a task"),
                    |app, mut word| {
                        app.task_data.tasks.push(Task::from_string(
                            word.drain(..).collect::<String>().trim().to_string(),
                        ));
                    },
                )))
        }
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
