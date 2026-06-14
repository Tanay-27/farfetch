use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, InputMode};
use crate::state::focus::FocusedPane;
use super::styles;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focused_pane == FocusedPane::Body;
    let is_editing = is_focused && app.input_mode == InputMode::Editing;

    let border_style = if is_editing {
        styles::editing_border()
    } else if is_focused {
        styles::active_border()
    } else {
        styles::inactive_border()
    };

    let title = if is_editing {
        " Body (JSON) · [Esc] stop editing "
    } else if is_focused {
        " Body (JSON) · [Enter] edit  [E] external editor "
    } else {
        " Body (JSON) "
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title);

    let display = if is_editing {
        format!("{}_", app.request.body)
    } else {
        app.request.body.clone()
    };

    frame.render_widget(Paragraph::new(display).block(block), area);
}
