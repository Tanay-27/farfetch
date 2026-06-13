use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::state::focus::FocusedPane;
use super::{highlight, styles};

pub fn render_metrics(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(styles::inactive_border())
        .title(" Response ");

    let line = if app.loading {
        Line::from(Span::raw("  Sending..."))
    } else if let Some(status) = app.response.status {
        let size_display = if app.response.size_bytes >= 1024 {
            format!("{:.1} KB", app.response.size_bytes as f64 / 1024.0)
        } else {
            format!("{} B", app.response.size_bytes)
        };
        let mut spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("{} {}", status, app.response.status_text),
                styles::status_style(status),
            ),
            Span::raw(format!(
                "  ·  {} ms  ·  {}",
                app.response.elapsed_ms, size_display
            )),
        ];
        // Show status message on the right if present
        if let Some(msg) = &app.status_message {
            spans.push(Span::raw("    "));
            spans.push(Span::styled(
                format!("✓ {}", msg),
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ));
        }
        Line::from(spans)
    } else if let Some(msg) = &app.status_message {
        Line::from(Span::styled(
            format!("  {}", msg),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ))
    } else {
        Line::from(Span::raw("  No response yet — press F5 to send"))
    };

    frame.render_widget(Paragraph::new(line).block(block), area);
}

pub fn render_body(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Response;

    let border_style = if is_focused {
        styles::active_border()
    } else {
        styles::inactive_border()
    };

    // Show scroll position when there's content to scroll
    let total_lines = app.response.lines.len();
    let title = if is_focused && total_lines > 0 {
        let offset = app.response.scroll_offset + 1;
        format!(
            " Response Body · {}/{} · [j/k] scroll  [Y] yank ",
            offset, total_lines
        )
    } else if is_focused {
        " Response Body · [Y] yank ".to_string()
    } else {
        " Response Body ".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let lines: Vec<Line<'static>> = if app.loading {
        vec![Line::from("  Sending request...")]
    } else if app.response.body.is_empty() && app.response.status.is_none() {
        vec![Line::from("  Press F5 to send a request")]
    } else if app.response.content_type.contains("json") {
        highlight::colorize_json(&app.response.body)
    } else {
        app.response
            .lines
            .iter()
            .map(|l| Line::from(l.clone()))
            .collect()
    };

    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((app.response.scroll_offset as u16, 0)),
        area,
    );
}
