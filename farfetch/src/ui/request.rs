use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode};
use crate::state::focus::FocusedPane;
use super::styles;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    // Split row: URL input | SEND button
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(10)])
        .split(area);

    render_url(frame, app, chunks[0]);
    render_send_button(frame, app, chunks[1]);
}

fn render_url(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Url;
    let is_editing = is_focused && app.input_mode == InputMode::Editing;

    let border_style = if is_editing {
        styles::editing_border()
    } else if is_focused {
        styles::active_border()
    } else {
        styles::inactive_border()
    };

    let title = if is_editing {
        " Request · [Enter] send  [Esc] cancel  [←/→] method "
    } else if is_focused {
        " Request · [Enter] edit URL  [F5] send  [←/→] method "
    } else {
        " Request "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let url_display = if is_editing {
        format!("{}_", app.request.url)
    } else {
        app.request.url.clone()
    };

    let line = Line::from(vec![
        Span::styled(
            format!("[{}]", app.request.method.as_str()),
            styles::method_style(&app.request.method),
        ),
        Span::raw(" "),
        Span::raw(url_display),
    ]);

    frame.render_widget(Paragraph::new(line).block(block), area);
}

fn render_send_button(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::SendButton;

    let (border_style, label_style) = if is_focused {
        (
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (
            Style::default().fg(Color::Green),
            Style::default().fg(Color::Green),
        )
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let label = if app.loading {
        " ··· "
    } else {
        " SEND "
    };

    let para = Paragraph::new(Line::from(Span::styled(label, label_style)))
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(para, area);
}
