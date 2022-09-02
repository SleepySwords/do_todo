use crossterm::event::{KeyCode, KeyEvent, MouseEventKind};

use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::actions::HelpAction;
use crate::app::{App, PopUpComponents, SelectedComponent};
use crate::{actions, utils};

use super::input_box::InputBoxComponent;

const COMPONENT_TYPE: SelectedComponent = SelectedComponent::CurrentTasks;

pub struct TaskList;

impl TaskList {
    fn selected(app: &App) -> &usize {
        &app.selected_task_index
    }

    pub fn available_actions() -> Vec<HelpAction<'static>> {
        vec![
            HelpAction::new(KeyCode::Char('d'), "d", "Delete the selected task"),
            HelpAction::new(
                KeyCode::Char('h'),
                "h",
                "Gives selected task lower priority",
            ),
            HelpAction::new(KeyCode::Char('c'), "c", "Completes the selected task"),
            HelpAction::new(
                KeyCode::Char('J'),
                "J",
                "Moves the task down on the task list",
            ),
            HelpAction::new(
                KeyCode::Char('K'),
                "K",
                "Moves the task up on the task list",
            ),
        ]
    }

    pub fn handle_event(app: &mut App, key_event: KeyEvent) -> Option<()> {
        let key_code = key_event.code;
        let selected_index = *Self::selected(app);
        match key_code {
            KeyCode::Char('d') => actions::open_delete_task_menu(app, selected_index),
            // todo proper deletion/popup
            // app.action = Action::Delete(selected_index, 0)
            KeyCode::Char('h') => {
                if app.task_data.tasks.is_empty() {
                    return Some(());
                }
                app.task_data.tasks[selected_index].priority =
                    app.task_data.tasks[selected_index].priority.next_priority();
            }
            KeyCode::Char('J') => {
                // if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                let task_length = app.task_data.tasks.len();
                let task = app.task_data.tasks.remove(app.selected_task_index);
                app.task_data
                    .tasks
                    .insert((app.selected_task_index + 1) % task_length, task);
                app.selected_task_index = (app.selected_task_index + 1) % task_length;
                // }
            }
            KeyCode::Char('K') => {
                // if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                let task_length = app.task_data.tasks.len();
                let task = app.task_data.tasks.remove(app.selected_task_index);
                if app.selected_task_index == 0 {
                    app.task_data.tasks.insert(task_length - 1, task);
                    app.selected_task_index = task_length - 1;
                } else {
                    app.task_data
                        .tasks
                        .insert((app.selected_task_index - 1) % task_length, task);
                    app.selected_task_index = (app.selected_task_index - 1) % task_length;
                }
                // }
            }
            KeyCode::Char('e') => {
                app.popup_stack
                    .push(PopUpComponents::InputBox(InputBoxComponent::filled(
                        // TODO: cleanup this so it doesn't use clone, perhaps use references?
                        String::from("Edit the  selected task"),
                        app.task_data.tasks[selected_index].title.clone(),
                        // This move is kinda jank not too sure, may try to find a better way
                        Box::new(move |app, mut word| {
                            app.task_data.tasks[selected_index].title =
                                word.drain(..).collect::<String>().trim().to_string();
                        }),
                    )))
            }
            KeyCode::Enter => {
                if app.task_data.tasks.is_empty() {
                    return Some(());
                }
                app.task_data.tasks[selected_index].progress =
                    !app.task_data.tasks[selected_index].progress;
            }
            KeyCode::Char('c') => actions::complete_task(app, selected_index),
            _ => {}
        }
        utils::handle_movement(
            key_code,
            &mut app.selected_task_index,
            app.task_data.tasks.len(),
        );
        Some(())
    }

    pub fn draw<B: tui::backend::Backend>(
        app: &App,
        layout_chunk: Rect,
        frame: &mut tui::Frame<B>,
    ) {
        let theme = &app.theme;
        let tasks: Vec<ListItem> = app
            .task_data
            .tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let style = if COMPONENT_TYPE == app.selected_component && *Self::selected(app) == i
                {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let progress = Span::styled(
                    if task.progress { "[-] " } else { "[ ] " },
                    style.fg(
                        if COMPONENT_TYPE == app.selected_component && *Self::selected(app) == i {
                            theme.selected_task_colour
                        } else {
                            Color::White
                        },
                    ),
                );

                let priority = Span::styled(
                    task.priority.short_hand(),
                    style.fg(task.priority.colour(theme)),
                );

                let content = Span::styled(
                    task.title.as_str(),
                    // style.fg(task.priority.colour(theme)),
                    style,
                );

                let content = Spans::from(vec![progress, priority, content]);
                ListItem::new(content)
            })
            .collect();

        let current =
            List::new(tasks).block(utils::generate_block("Current List", COMPONENT_TYPE, app));

        let mut state = ListState::default();
        state.select(if COMPONENT_TYPE == app.selected_component {
            Some(*Self::selected(app))
        } else {
            None
        });

        frame.render_stateful_widget(current, layout_chunk, &mut state);
    }
}
