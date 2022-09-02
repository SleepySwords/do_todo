use chrono::Local;
use crossterm::event::{KeyCode, KeyModifiers};

use crate::{
    app::{App, PopUpComponents, SelectedComponent},
    component::dialog::{DialogAction, DialogComponent},
    input::handle_key,
    task::{CompletedTask, Task},
};

// Action class maybe?!!
pub struct HelpAction<'a> {
    character: KeyCode,
    short_hand: &'a str,
    description: &'a str,
}

impl HelpAction<'_> {
    pub fn new<'a>(
        character: KeyCode,
        short_hand: &'a str,
        description: &'a str,
    ) -> HelpAction<'a> {
        HelpAction {
            character,
            short_hand,
            description,
        }
    }
}

pub fn open_help_menu(app: &mut App) {
    // Tasks that are universal
    let mut actions: Vec<DialogAction> = vec![
        DialogAction::new(String::from("1    Change to current task window"), |app| {
            app.selected_component = SelectedComponent::CurrentTasks;
        }),
        DialogAction::new(
            String::from("2    Change to completed task window"),
            |app| {
                app.selected_component = SelectedComponent::CompletedTasks;
            },
        ),
    ];
    for ac in app.selected_component.available_help_actions() {
        actions.push(DialogAction::new(
            String::from(format!("{}    {}", ac.short_hand, ac.description)),
            move |app| {
                handle_key(
                    crossterm::event::KeyEvent {
                        code: ac.character,
                        modifiers: KeyModifiers::NONE,
                    },
                    app,
                );
            },
        ));
    }

    app.popup_stack
        .push(PopUpComponents::DialogBox(DialogComponent::new(
            String::from("Help Menu"),
            actions,
        )));
}

pub fn open_delete_task_menu(app: &mut App, selected_task: usize) {
    if app.task_data.tasks.is_empty() {
        return;
    }
    app.popup_stack
        .push(PopUpComponents::DialogBox(DialogComponent::new(
            "Delete selected task".to_string(),
            vec![
                DialogAction::new(String::from("Delete"), move |app| {
                    app.task_data.tasks.remove(selected_task);
                    if selected_task == app.task_data.tasks.len() && !app.task_data.tasks.is_empty()
                    {
                        app.selected_task_index -= 1;
                    }
                }),
                DialogAction::new(String::from("Cancel"), |_| {}),
            ],
        )));
}

pub fn restore_task(app: &mut App, selected_task: usize) {
    if app.task_data.completed_tasks.is_empty() {
        return;
    }
    app.task_data.tasks.push(Task::from_completed_task(
        app.task_data.completed_tasks.remove(selected_task),
    ));
    if selected_task == app.task_data.completed_tasks.len()
        && !app.task_data.completed_tasks.is_empty()
    {
        app.selected_completed_task_index -= 1;
    }
}

pub fn complete_task(app: &mut App, selected_task: usize) {
    if app.task_data.tasks.is_empty() {
        return;
    }
    let local = Local::now();
    let time_completed = local.naive_local();
    let task = app.task_data.tasks.remove(selected_task);
    app.task_data
        .completed_tasks
        .push(CompletedTask::from_task(task, time_completed));
    if selected_task == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
        app.selected_task_index -= 1;
    }
}
