use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::actions::HelpAction;
use crate::app::{App, SelectedComponent};
use crate::view::{DrawableComponent, EventResult};
use crate::{actions, utils};

const COMPONENT_TYPE: SelectedComponent = SelectedComponent::CompletedTasks;

pub struct CompletedList {
    selected_index: Rc<RefCell<usize>>,
}

impl CompletedList {
    pub fn new(selected_index: Rc<RefCell<usize>>) -> Self {
        Self { selected_index }
    }

    fn selected(&self) -> Ref<usize> {
        self.selected_index.borrow()
    }

    fn selected_mut(&self) -> RefMut<usize> {
        self.selected_index.borrow_mut()
    }

    pub fn available_actions() -> Vec<HelpAction<'static>> {
        vec![HelpAction::new(
            KeyCode::Char('r'),
            "r",
            "Restores the selected task",
        )]
    }

    pub fn handle_event(app: &mut App, key_code: KeyCode) -> Option<()> {
        Some(())
    }
}

impl DrawableComponent for CompletedList {
    fn draw(&self, app: &App, draw_area: Rect, drawer: &mut crate::view::Drawer) {
        let theme = &app.theme;

        let selected_index = *self.selected();

        let completed_tasks: Vec<ListItem> = app
            .task_store
            .completed_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let colour = if let SelectedComponent::CompletedTasks = app.selected_component {
                    if selected_index == i {
                        theme.selected_task_colour
                    } else {
                        Color::White
                    }
                } else {
                    Color::White
                };
                let content = Spans::from(Span::styled(
                    format!(
                        "{} {}",
                        task.time_completed.format("%d/%m/%y %-I:%M:%S %p"),
                        task.task.title
                    ),
                    Style::default().fg(colour),
                ));
                ListItem::new(content)
            })
            .collect();

        let completed_list = List::new(completed_tasks)
            .block(utils::generate_default_block(
                "Completed tasks",
                COMPONENT_TYPE,
                app,
            ))
            .style(Style::default().fg(Color::White));

        let mut completed_state = ListState::default();
        if !app.task_store.completed_tasks.is_empty() {
            let index = match app.selected_component {
                SelectedComponent::CompletedTasks => selected_index,
                _ => app.task_store.completed_tasks.len() - 1,
            };
            completed_state.select(Some(index));
        }

        drawer.draw_stateful_widget(completed_list, &mut completed_state, draw_area);
    }

    fn key_pressed(&mut self, app: &mut App, key_code: crossterm::event::KeyCode) -> EventResult {
        let result = utils::handle_movement(
            key_code,
            &mut self.selected_mut(),
            app.task_store.completed_tasks.len(),
        );

        if result == EventResult::Consumed {
            return result;
        }

        let mut selected_index = self.selected_mut();

        if let KeyCode::Char('r') = key_code {
            actions::restore_task(app, &mut selected_index);
            return EventResult::Consumed;
        }

        EventResult::Ignored
    }
}
