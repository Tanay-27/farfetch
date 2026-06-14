use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::app::{App, SidebarItem};
use crate::state::focus::FocusedPane;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Sidebar;

    let border_style = if is_focused {
        super::styles::active_border()
    } else {
        super::styles::inactive_border()
    };

    let title = if is_focused {
        " Collections · [j/k] scroll  [Enter] load "
    } else {
        " Collections "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    if app.sidebar_items.is_empty() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(Span::styled(
                "  No collections\n  Ctrl+S to save",
                Style::default().fg(Color::DarkGray),
            )))
            .block(block),
            area,
        );
        return;
    }

    // Build list items; track the flat index of the selected request row
    let selected_flat = app.sidebar_request_index_to_flat(app.sidebar_cursor);
    let mut list_state = ListState::default();
    list_state.select(Some(selected_flat));

    let items: Vec<ListItem> = app
        .sidebar_items
        .iter()
        .map(|item| match item {
            SidebarItem::CollectionHeader { name, .. } => {
                ListItem::new(Line::from(Span::styled(
                    format!(" {}", name.to_uppercase()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )))
            }
            SidebarItem::Request { name, col_idx: _, req_idx: _ } => {
                ListItem::new(Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(name.clone(), Style::default().fg(Color::White)),
                ]))
            }
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    frame.render_stateful_widget(list, area, &mut list_state);
}
