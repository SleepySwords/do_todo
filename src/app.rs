use std::io;

use crossterm::event::{self, Event, KeyCode};
use tui::{Frame, Terminal, backend::Backend, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Span, Spans, Text}, widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph}};
use chrono::prelude::Local;
use crate::task::Task;

pub struct App {
    add_mode: bool,
    default_colour: Color,
    selected_index: usize,
    words: String,
    tasks: Vec<Task>,
    completed_tasks: Vec<Task>
}

impl Default for App {
    fn default() -> Self {
        App {
            add_mode: false,
            default_colour: Color::Rgb(255, 192, 203),
            selected_index: 0,
            words: String::new(),
            tasks: Vec::new(),
            completed_tasks: Vec::new()
        }
    }
}

impl App {
    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.size());
        f.render_widget(Block::default().title("Todo!!!\n".to_string()), chunks[0]);

        let tasks: Vec<ListItem> = self.tasks.iter().map(|x| {
            let content = Spans::from(Span::raw(if x.progress { format!("[-] {}", x.content) } else { format!("[ ] {}", x.content) }));
            ListItem::new(content)
        }).collect();
        
        let completed_tasks: Vec<ListItem> = self.completed_tasks.iter().map(|x| {
            let content = Spans::from(Span::raw(format!("{}", x.content)));
            ListItem::new(content)
        }).collect();

        {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(chunks[1]);
            
            let current = List::new(tasks)
                .block(Block::default().title("Current List").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(self.default_colour)))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow));
            
            let mut state = ListState::default();
            state.select(Some(self.selected_index));

            f.render_stateful_widget(current, chunks[0], &mut state);
           

            let recently_competed = List::new(completed_tasks)
                .block(Block::default().title("Completed").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(self.default_colour)))
                .style(Style::default().fg(Color::White));
            
            let mut completed_state = ListState::default();
            if self.completed_tasks.len() != 0 {
                completed_state.select(Some(self.completed_tasks.len() - 1));
            }
            
            f.render_stateful_widget(recently_competed, chunks[1], &mut completed_state);
        }


        if self.add_mode {
            let text = Text::from(Spans::from(self.words.as_ref()));
            let help_message = Paragraph::new(text);
            let help_message = help_message.block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("Add a task"));
            let area = centered_rect(70, 20, f.size());
            f.render_widget(Clear, area);
            f.render_widget(help_message, area);
            f.set_cursor(area.x + 1 + self.words.len() as u16, area.y + 1)
        }
    }

    pub fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            terminal.draw(|mut f| self.ui(&mut f))?;

            // This function blocks
            if let Event::Key(key) = event::read()? {
                if self.add_mode {
                    match key.code {
                        KeyCode::Char(c) => self.words.push(c),
                        KeyCode::Backspace => {
                            self.words.pop();
                        },
                        KeyCode::Enter => {
                            self.tasks.push(Task::new(self.words.drain(..).collect()));
                            self.add_mode = !self.add_mode
                        },
                        KeyCode::Esc => self.add_mode = !self.add_mode,
                        _ => {  }
                    }
                    continue;
                }
                match key.code {
                    KeyCode::Char('a') => self.add_mode = !self.add_mode,
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('j') => {
                        if self.tasks.len() == 0 { continue; }
                        if self.selected_index == self.tasks.len() - 1 {
                            self.selected_index = 0;
                        } else {
                            self.selected_index += 1;
                        }
                    }
                    KeyCode::Char('d') => {
                        if self.tasks.len() == 0 { continue; }
                        self.tasks.remove(self.selected_index);
                        if self.selected_index == self.tasks.len() && self.tasks.len() != 0 {
                            self.selected_index -= 1;
                        }
                    }
                    KeyCode::Char('p') => {
                        if self.tasks.len() == 0 { continue; }
                        self.tasks[self.selected_index].progress = !self.tasks[self.selected_index].progress;
                    }
                    KeyCode::Char(' ') => {
                        let local = Local::now();
                        let time = local.time().format("%-I:%M:%S %p").to_string();
                        if self.tasks.len() == 0 { continue; }
                        let mut task = self.tasks.remove(self.selected_index);
                        task.content = format!("{} {}", time, task.content);
                        self.completed_tasks.push(task);
                        if self.selected_index == self.tasks.len() && self.tasks.len() != 0 {
                            self.selected_index -= 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        if self.tasks.len() == 0 { continue; }
                        if self.selected_index == 0 {
                            self.selected_index = self.tasks.len() - 1;
                        } else {
                            self.selected_index -= 1;
                        }
                    }
                    _ => {  }
                }
            }
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
