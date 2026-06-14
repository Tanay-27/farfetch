use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{App, EnvEditMode, EnvManagerState, EnvPane, SaveStep};
use crate::fuzzy::FuzzyOverlay;

pub fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(62, 80, area);
    frame.render_widget(Clear, popup);

    // (section_header, key, desc) — empty key = section header row
    let rows: &[(&str, &str, &str)] = &[
        ("NAVIGATION", "", ""),
        ("", "Tab / Shift+Tab",  "Cycle panes"),
        ("", "j / k",            "Scroll / navigate list"),
        ("", "← / →",            "Cycle HTTP method"),
        ("", "Enter",            "Edit field / load sidebar request"),
        ("", "Esc",              "Exit editing mode"),
        ("REQUEST", "", ""),
        ("", "F5",               "Send request"),
        ("", "F4",               "Save to collection"),
        ("", "F6",               "Paste / import cURL"),
        ("", "C",                "Copy request as cURL"),
        ("", "E",                "Open body in editor  (Body pane)"),
        ("RESPONSE", "", ""),
        ("", "Y",                "Yank body to clipboard"),
        ("OVERLAYS", "", ""),
        ("", "F2",               "Environment manager"),
        ("", "F3",               "History search"),
        ("", "F1 / ?",           "This help"),
        ("OTHER", "", ""),
        ("", "q / Ctrl+C",       "Quit"),
    ];

    let items: Vec<Line> = rows
        .iter()
        .map(|(section, key, desc)| {
            if !section.is_empty() {
                // Section header
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        *section,
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        format!("    {:<20}", key),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(*desc, Style::default().fg(Color::Gray)),
                ])
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " farfetch — Keybindings · any key to close ",
            Style::default().fg(Color::Cyan),
        ));

    frame.render_widget(Paragraph::new(items).block(block), popup);
}

pub fn render_fuzzy(frame: &mut Frame, fuzzy: &FuzzyOverlay, area: Rect) {
    let popup = centered_rect(72, 65, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(Span::styled(
            " History Search · [j/k] navigate  [Enter] load  [Esc] close ",
            Style::default().fg(Color::Yellow),
        ));

    let inner = block.inner(popup);
    frame.render_widget(block, popup);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    frame.render_widget(
        Paragraph::new(format!(" > {}▌", fuzzy.query))
            .style(Style::default().fg(Color::White)),
        chunks[0],
    );

    let items: Vec<ListItem> = fuzzy
        .filtered
        .iter()
        .map(|&i| {
            let e = &fuzzy.all_entries[i];
            let label = if e.name.is_empty() {
                format!(" {}  {}", e.method.as_str(), e.url)
            } else {
                format!(" {}  {}  — {}", e.method.as_str(), e.url, e.name)
            };
            ListItem::new(label)
        })
        .collect();

    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("  (no results)").style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
        return;
    }

    let mut list_state = ListState::default();
    list_state.select(Some(fuzzy.selected));

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, chunks[1], &mut list_state);
}

#[allow(clippy::too_many_arguments)]
pub fn render_save_request(
    frame: &mut Frame,
    app: &App,
    step: &SaveStep,
    col_query: &str,
    col_selected: &Option<usize>,
    col_name: &str,
    req_name: &str,
    area: Rect,
) {
    let popup = centered_rect(52, 60, area);
    frame.render_widget(Clear, popup);

    match step {
        SaveStep::PickCollection => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(Span::styled(
                    " Save Request · Step 1: Pick or create collection ",
                    Style::default().fg(Color::Green),
                ));

            let inner = block.inner(popup);
            frame.render_widget(block, popup);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(0)])
                .split(inner);

            // Input row
            frame.render_widget(
                Paragraph::new(format!(" Collection: {}▌", col_query))
                    .style(Style::default().fg(Color::White)),
                chunks[0],
            );

            // Filtered collection list
            let matches = app.matching_collections(col_query);
            if matches.is_empty() && !col_query.is_empty() {
                frame.render_widget(
                    Paragraph::new(Span::styled(
                        format!("  + Create \"{}\"", col_query),
                        Style::default().fg(Color::Yellow),
                    )),
                    chunks[1],
                );
            } else {
                let items: Vec<ListItem> = matches
                    .iter()
                    .map(|&i| {
                        ListItem::new(Line::from(Span::raw(format!(
                            "  {}",
                            app.collections[i].name
                        ))))
                    })
                    .collect();

                let mut list_state = ListState::default();
                if let Some(sel) = col_selected {
                    list_state.select(matches.iter().position(|&i| i == *sel));
                }

                let list = List::new(items)
                    .highlight_style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol("› ");

                frame.render_stateful_widget(list, chunks[1], &mut list_state);
            }
        }

        SaveStep::NameRequest => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(Span::styled(
                    " Save Request · Step 2: Name this request ",
                    Style::default().fg(Color::Green),
                ));

            let inner = block.inner(popup);
            frame.render_widget(block, popup);

            let lines = vec![
                Line::from(Span::styled(
                    format!(" Collection: {}", col_name),
                    Style::default().fg(Color::Cyan),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    format!(" Name: {}▌", req_name),
                    Style::default().fg(Color::White),
                )),
                Line::from(Span::raw("")),
                Line::from(Span::styled(
                    " [Enter] save   [Esc] back",
                    Style::default().fg(Color::DarkGray),
                )),
            ];

            frame.render_widget(Paragraph::new(lines), inner);
        }
    }
}

pub fn render_env_manager(frame: &mut Frame, app: &App, state: &EnvManagerState, area: Rect) {
    let popup = centered_rect(80, 70, area);
    frame.render_widget(Clear, popup);

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(Span::styled(
            " Environment Manager · [Tab] switch pane  [Esc] close ",
            Style::default().fg(Color::Cyan),
        ));
    let inner = outer.inner(popup);
    frame.render_widget(outer, popup);

    // Split: left env list (30%) | right var list (70%)
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(inner);

    render_env_list(frame, app, state, panes[0]);
    render_var_list(frame, app, state, panes[1]);
}

fn render_env_list(frame: &mut Frame, app: &App, state: &EnvManagerState, area: Rect) {
    let is_focused = state.focus == EnvPane::EnvList;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let hint = if is_focused { " [n] new  [d] del  [Enter] activate " } else { " Envs " };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(hint, border_style));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let names = app.env_names();

    // Inline edit: new env name input
    if let EnvEditMode::NewEnvName { input } = &state.edit {
        if is_focused {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(0)])
                .split(inner);
            frame.render_widget(
                Paragraph::new(format!(" New env: {}▌", input))
                    .style(Style::default().fg(Color::Yellow)),
                rows[0],
            );
            return;
        }
    }

    if names.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(" (none)  [n] to add", Style::default().fg(Color::DarkGray))),
            inner,
        );
        return;
    }

    let items: Vec<ListItem> = names
        .iter()
        .map(|name| {
            let active_marker = if name == &app.active_env { "● " } else { "  " };
            let style = if name == &app.active_env {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!("{}{}", active_marker, name), style)))
        })
        .collect();

    let mut list_state = ListState::default();
    if is_focused {
        list_state.select(Some(state.env_cursor));
    }

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn render_var_list(frame: &mut Frame, app: &App, state: &EnvManagerState, area: Rect) {
    let is_focused = state.focus == EnvPane::VarList;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let names = app.env_names();
    let env_name = names.get(state.env_cursor).map(|s| s.as_str()).unwrap_or("—");
    let title = format!(" Variables · {} ", env_name);
    let hint = if is_focused {
        format!("{}[n] new  [d] del  [Enter] edit", title)
    } else {
        title
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(hint, border_style));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vars = app.env_vars_sorted(env_name);

    // Inline edit overlays
    match &state.edit {
        EnvEditMode::NewVarKey { input } if is_focused => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(" New variable", Style::default().fg(Color::Yellow))),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled(
                        format!(" Key:   {}▌", input),
                        Style::default().fg(Color::White),
                    )),
                ]),
                inner,
            );
            return;
        }
        EnvEditMode::NewVarValue { key, input } if is_focused => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(" New variable", Style::default().fg(Color::Yellow))),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled(
                        format!(" Key:   {}", key),
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        format!(" Value: {}▌", input),
                        Style::default().fg(Color::White),
                    )),
                ]),
                inner,
            );
            return;
        }
        EnvEditMode::EditVarValue { var_key, input } if is_focused => {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(" Edit variable", Style::default().fg(Color::Yellow))),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled(
                        format!(" Key:   {}", var_key),
                        Style::default().fg(Color::Cyan),
                    )),
                    Line::from(Span::styled(
                        format!(" Value: {}▌", input),
                        Style::default().fg(Color::White),
                    )),
                    Line::from(Span::raw("")),
                    Line::from(Span::styled(
                        " [Enter] save   [Esc] cancel",
                        Style::default().fg(Color::DarkGray),
                    )),
                ]),
                inner,
            );
            return;
        }
        _ => {}
    }

    if vars.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                " (no variables)  [n] to add",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    // Compute column widths
    let key_width = vars.iter().map(|(k, _)| k.len()).max().unwrap_or(8).max(8);
    let items: Vec<ListItem> = vars
        .iter()
        .map(|(k, v)| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<width$}", k, width = key_width),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled("  ", Style::default()),
                Span::styled(v.clone(), Style::default().fg(Color::White)),
            ]))
        })
        .collect();

    let mut list_state = ListState::default();
    if is_focused {
        list_state.select(Some(state.var_cursor));
    }

    let list = List::new(items)
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("► ");

    frame.render_stateful_widget(list, inner, &mut list_state);
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let margin_y = (100 - pct_y) / 2;
    let margin_x = (100 - pct_x) / 2;

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(margin_y),
            Constraint::Percentage(pct_y),
            Constraint::Percentage(margin_y),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(margin_x),
            Constraint::Percentage(pct_x),
            Constraint::Percentage(margin_x),
        ])
        .split(vert[1])[1]
}
