// use crate::{task::Task, tests::assert_screen, utils::test::setup};
// use std::collections::HashMap;
// use std::io;
// use tui::style::{Color, Modifier, Style};

// #[test]
// fn test_main() -> io::Result<()> {
//     use crate::app::TaskStore;

//     let (mut app, mut stack_layout) = setup(TaskStore {
//         tasks: vec![Task::from_string(String::from("meme"))],
//         completed_tasks: vec![],
//         tags: HashMap::new(),
//         auto_sort: false,
//     });

//     let format_styles = vec![
//         Style::default().fg(Color::Green).bg(Color::Reset),
//         Style::default().fg(Color::Reset).bg(Color::Reset),
//         Style::default().fg(Color::White).bg(Color::Reset),
//         Style::default()
//             .fg(Color::LightBlue)
//             .bg(Color::Reset)
//             .add_modifier(Modifier::BOLD),
//         Style::default()
//             .fg(Color::White)
//             .bg(Color::Reset)
//             .add_modifier(Modifier::BOLD),
//     ];

//     let str = "
// {0:┌Current List────────────────────────────────────┐}{1:┌Task information────────────────────────────────┐}}
// {0:│}{3:[ ] }{4:    }{3:meme}{1:                                    }{0:│}{1:│Title     meme                                  │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│Priority  }{2:None}{1:                                  │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│Tags      None                                  │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:│}{1:                                                }{0:│}{1:│                                                │}}
// {0:└────────────────────────────────────────────────┘}{1:│                                                │}}
// {2:┌Completed tasks─────────────────────────────────┐}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:│                                                │}{1:│                                                │}}
// {2:└────────────────────────────────────────────────┘}{1:└────────────────────────────────────────────────┘}}
// {2:Press x for help. Press q to exit.}{1:                                                                  }}"
//         .trim();

//     assert_screen(&mut app, &mut stack_layout, format_styles, str);

//     Ok(())
// }
