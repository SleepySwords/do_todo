use chrono::Local;

use crate::{
    app::{App, PopUpComponents, SelectedComponent},
    components::dialog::{DialogComponent, DialogOption},
    task::{CompletedTask, Task},
};

pub fn open_help_menu(app: &mut App) {
    // Tasks that are universal
    let mut actions: Vec<DialogOption> = vec![
        (
            String::from("1    Change to current task window"),
            Box::new(|app| {
                app.selected_window = SelectedComponent::CurrentTasks(0);
            }),
        ),
        (
            String::from("2    Change to completed task window"),
            Box::new(|app| {
                app.selected_window = SelectedComponent::CompletedTasks(0);
            }),
        ),
    ];
    if let SelectedComponent::CurrentTasks(selected_task) = app.selected_window {
        actions.push((
            String::from("c    Complete selected task"),
            Box::new(move |app| {
                complete_task(app, selected_task);
            }),
        ));
        actions.push((
            String::from("d    Delete selected task"),
            Box::new(move |app| {
                open_delete_task_menu(app, selected_task);
            }),
        ));
    }
    if let SelectedComponent::CompletedTasks(selected_task) = app.selected_window {
        actions.push((
            String::from("r    Restore current task"),
            Box::new(move |app| {
                restore_task(app, selected_task);
            }),
        ));
    }
    app.popup_stack
        .push_front(PopUpComponents::DialogBox(DialogComponent::new(
            String::from("Help Menu"),
            actions,
        )));
}

pub fn open_delete_task_menu(app: &mut App, selected_task: usize) {
    if app.task_data.tasks.is_empty() {
        return;
    }
    app.popup_stack
        .push_front(PopUpComponents::DialogBox(DialogComponent::new(
            format!("Delete task {}", app.task_data.tasks[selected_task].title),
            vec![
                (
                    String::from("Delete"),
                    Box::new(move |app| {
                        app.task_data.tasks.remove(selected_task);
                        if selected_task == app.task_data.tasks.len()
                            && !app.task_data.tasks.is_empty()
                        {
                            app.selected_window =
                                SelectedComponent::CurrentTasks(selected_task - 1);
                        }
                    }),
                ),
                (String::from("Cancel"), Box::new(|_| {})),
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
    if selected_task == app.task_data.tasks.len() && !app.task_data.tasks.is_empty() {
        app.selected_window = SelectedComponent::CompletedTasks(selected_task - 1);
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
        app.selected_window = SelectedComponent::CurrentTasks(selected_task - 1);
    }
}
