use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, HeaderEditMode};
use crate::state::focus::FocusedPane;
use super::styles;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Params;

    let border_style = if is_focused {
        styles::active_border()
    } else {
        styles::inactive_border()
    };

    let title = if is_focused {
        " Headers · [n] add  [d] del  [Enter] edit "
    } else {
        " Headers "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Show inline editor when actively adding/editing
    if is_focused {
        match &app.header_edit {
            HeaderEditMode::NewKey { input } => {
                render_inline_edit(frame, inner, "New header", "Key", input, None);
                return;
            }
            HeaderEditMode::NewValue { key, input } => {
                render_inline_edit(frame, inner, "New header", "Value", input, Some(key.as_str()));
                return;
            }
            HeaderEditMode::EditValue { idx, input } => {
                let key = app
                    .request
                    .headers
                    .get(*idx)
                    .map(|(k, _)| k.as_str())
                    .unwrap_or("");
                render_inline_edit(frame, inner, "Edit header", "Value", input, Some(key));
                return;
            }
            HeaderEditMode::None => {}
        }
    }

    // Normal list view
    let items: Vec<ListItem> = if app.request.headers.is_empty() {
        vec![ListItem::new(Span::styled(
            "  (no headers)  [n] to add",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        let key_width = app
            .request
            .headers
            .iter()
            .map(|(k, _)| k.len())
            .max()
            .unwrap_or(4)
            .max(4);

        app.request
            .headers
            .iter()
            .map(|(k, v)| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {:<width$}", k, width = key_width),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled("  ", Style::default()),
                    Span::styled(v.clone(), Style::default().fg(Color::White)),
                ]))
            })
            .collect()
    };

    let mut list_state = ListState::default();
    if is_focused && !app.request.headers.is_empty() {
        list_state.select(Some(app.request.selected_header));
    }

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_inline_edit(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    active_field: &str,
    input: &str,
    locked_key: Option<&str>,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    frame.render_widget(
        Paragraph::new(Span::styled(title, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        chunks[0],
    );

    if let Some(key) = locked_key {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Key:   ", Style::default().fg(Color::DarkGray)),
                Span::styled(key.to_string(), Style::default().fg(Color::Cyan)),
            ])),
            chunks[1],
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("Value: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}▌", input), Style::default().fg(Color::White)),
            ])),
            chunks[2],
        );
    } else {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{}:   ", active_field),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(format!("{}▌", input), Style::default().fg(Color::White)),
            ])),
            chunks[1],
        );
    }

    frame.render_widget(
        Paragraph::new(Span::styled(
            "[Enter] confirm  [Esc] cancel",
            Style::default().fg(Color::DarkGray),
        )),
        chunks[4],
    );
}
