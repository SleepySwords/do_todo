use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use crossterm::event::KeyCode;

use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{List, ListItem, ListState};

use crate::actions::HelpAction;
use crate::app::{App, Mode};
use crate::draw::{DrawableComponent, EventResult};
use crate::task::Task;
use crate::utils;

const COMPONENT_TYPE: Mode = Mode::CompletedTasks;

pub struct CompletedList {
    pub area: Rect,
    selected_index: Rc<RefCell<usize>>,
}

impl CompletedList {
    pub fn new(selected_index: Rc<RefCell<usize>>) -> Self {
        Self {
            area: Rect::default(),
            selected_index,
        }
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

    pub fn restore_task(&mut self, app: &mut App) {
        if app.task_store.completed_tasks.is_empty() {
            return;
        }

        let current_selected_task = app.task_store.completed_tasks.remove(*self.selected());

        app.task_store.add_task(Task::from_completed_task(
                current_selected_task
        ));

        if *self.selected() == app.task_store.completed_tasks.len()
            && !app.task_store.completed_tasks.is_empty()
        {
            *self.selected_mut() -= 1;
        }
    }
}

impl DrawableComponent for CompletedList {
    fn draw(&self, app: &App, drawer: &mut crate::draw::Drawer) {
        let theme = &app.theme;

        let selected_index = *self.selected();

        let completed_tasks: Vec<ListItem> = app
            .task_store
            .completed_tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let colour = if let Mode::CompletedTasks = app.mode {
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
            completed_state.select(Some(selected_index));
        }

        drawer.draw_stateful_widget(completed_list, &mut completed_state, self.area);
    }

    fn key_event(&mut self, app: &mut App, key_event: crossterm::event::KeyEvent) -> EventResult {
        let key_code = key_event.code;

        let result = utils::handle_key_movement(
            key_code,
            &mut self.selected_mut(),
            app.task_store.completed_tasks.len(),
        );

        if result == EventResult::Consumed {
            return result;
        }

        if let KeyCode::Char('r') = key_code {
            self.restore_task(app);
            return EventResult::Consumed;
        }

        EventResult::Ignored
    }

    fn mouse_event(
        &mut self,
        app: &mut App,
        mouse_event: crossterm::event::MouseEvent,
    ) -> EventResult {
        return utils::handle_mouse_movement(
            app,
            self.area,
            Some(COMPONENT_TYPE),
            app.task_store.completed_tasks.len(),
            &mut self.selected_index.borrow_mut(),
            mouse_event,
        );
    }

    fn update_layout(&mut self, rect: Rect) {
        self.area = rect;
    }
}
