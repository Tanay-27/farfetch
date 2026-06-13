use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::config::environment::matched_pattern;
use super::styles;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let branch = app.git_branch.as_deref().unwrap_or("—");
    let env    = &app.active_env;

    // Left: active env + current git branch
    let left = format!(" {} ·  git: {}", env, branch);

    // Right: the mapping rule that produced the active env, e.g. "feature/* → local"
    let right = if let Some(pattern) = matched_pattern(branch, &app.config.git_branch_mapping) {
        format!("{}  →  {}  [?] help ", pattern, env)
    } else {
        format!("(no rule matched)  →  {}  [?] help ", env)
    };

    let inner_width = area.width.saturating_sub(2) as usize;
    let padding = inner_width
        .saturating_sub(left.len() + right.len())
        .saturating_sub(1); // account for leading space in left

    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(padding)),
        Span::styled(right, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::inactive_border())
        .title(Span::styled(
            " farfetch ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    frame.render_widget(Paragraph::new(line).block(block), area);
}
