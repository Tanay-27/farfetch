use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{App, InputMode};
use crate::state::focus::FocusedPane;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let hints = build_hints(app);

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::raw(" "));
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }
        spans.push(Span::styled(
            format!("[{}]", key),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(Color::DarkGray),
        ));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn build_hints(app: &App) -> Vec<(&'static str, &'static str)> {
    let mut h: Vec<(&'static str, &'static str)> = Vec::new();

    match app.focused_pane {
        FocusedPane::Sidebar => {
            h.push(("j/k", "scroll"));
            h.push(("Enter", "load"));
            h.push(("d", "delete"));
        }
        FocusedPane::Url => {
            if app.input_mode == InputMode::Editing {
                h.push(("Enter", "send"));
                h.push(("Esc", "cancel"));
                h.push(("F6", "paste/cURL"));
            } else {
                h.push(("Enter", "edit"));
                h.push(("←/→", "method"));
                h.push(("F5", "send"));
            }
        }
        FocusedPane::SendButton => {
            h.push(("Enter", "send"));
        }
        FocusedPane::Params => {
            h.push(("j/k", "navigate"));
            h.push(("n", "add header"));
            h.push(("d", "delete"));
            h.push(("Enter", "edit value"));
        }
        FocusedPane::Body => {
            if app.input_mode == InputMode::Editing {
                h.push(("Esc", "done"));
                h.push(("F6", "paste"));
            } else {
                h.push(("Enter", "edit"));
                h.push(("E", "ext editor"));
            }
        }
        FocusedPane::Response => {
            h.push(("j/k", "scroll"));
            h.push(("Y", "yank"));
        }
    }

    h.push(("C", "copy curl"));
    h.push(("F4", "save"));
    h.push(("F2", "env"));
    h.push(("F3", "history"));
    h.push(("?", "help"));
    h.push(("q", "quit"));
    h
}
