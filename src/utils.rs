use tui::layout::{Constraint, Direction, Layout, Rect};

// Only available for percentages, ratios and length
pub fn centered_rect(constraint_x: Constraint, constraint_y: Constraint, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(generate_constraints(constraint_y, r.height).as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(generate_constraints(constraint_x, r.width).as_ref())
        .split(popup_layout[1])[1]
}

fn generate_constraints(constraint: Constraint, rect_bound: u16) -> [Constraint; 3] {
    match constraint {
        Constraint::Percentage(percent) => [
            Constraint::Percentage((100 - percent) / 2),
            Constraint::Percentage(percent),
            Constraint::Percentage((100 - percent) / 2),
        ],
        Constraint::Ratio(num, den) => [
            Constraint::Ratio((den - num) / 2, den),
            Constraint::Ratio(num, den),
            Constraint::Ratio((den - num) / 2, den),
        ],
        Constraint::Length(length) => [
            Constraint::Length((rect_bound - length) / 2),
            Constraint::Length(length),
            Constraint::Length((rect_bound - length) / 2),
        ],
        _ => [constraint, constraint, constraint],
    }
}
