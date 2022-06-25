use crate::{
    app::{App, SelectedComponent},
    components::dialog::{DialogComponent, DialogOption},
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
    if let SelectedComponent::CurrentTasks(selected_index) = app.selected_window {
        actions.push((
            String::from("x    Delete selected task"),
            Box::new(move |app| {
                open_delete_task_menu(app, selected_index);
            }),
        ));
    }
    app.dialog_stack
        .push_front(DialogComponent::new(String::from("Help Menu"), actions));
}

pub fn open_delete_task_menu(app: &mut App, selected_task: usize) {
    app.dialog_stack.push_front(DialogComponent::new(
        format!("Delete task {}", app.task_data.tasks[selected_task].title),
        vec![
            (
                String::from("Delete"),
                Box::new(move |app| {
                    app.task_data.tasks.remove(selected_task);
                    if selected_task == app.task_data.tasks.len() && !app.task_data.tasks.is_empty()
                    {
                        app.selected_window = SelectedComponent::CurrentTasks(selected_task - 1);
                    }
                }),
            ),
            (String::from("Cancel"), Box::new(|_| {})),
        ],
    ));
}
