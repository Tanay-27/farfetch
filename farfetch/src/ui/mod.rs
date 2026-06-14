use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, Overlay};

pub mod body;
pub mod footer;
pub mod header;
pub mod highlight;
pub mod overlay;
pub mod params;
pub mod request;
pub mod response;
pub mod sidebar;
pub mod styles;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.size();

    // Horizontal split: sidebar | main
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(24), Constraint::Min(0)])
        .split(area);

    let main = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // header
            Constraint::Length(3),      // request bar + send button
            Constraint::Percentage(30), // params | body
            Constraint::Length(3),      // response metrics
            Constraint::Min(0),         // response body
            Constraint::Length(1),      // footer command bar
        ])
        .split(columns[1]);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(main[2]);

    sidebar::render(frame, app, columns[0]);
    header::render(frame, app, main[0]);
    request::render(frame, app, main[1]);
    params::render(frame, app, middle[0]);
    body::render(frame, app, middle[1]);
    response::render_metrics(frame, app, main[3]);
    response::render_body(frame, app, main[4]);
    footer::render(frame, app, main[5]);

    // Overlays on top
    match &app.overlay {
        Overlay::None => {}
        Overlay::Help => overlay::render_help(frame, area),
        Overlay::FuzzySearch(fuzzy) => overlay::render_fuzzy(frame, fuzzy, area),
        Overlay::SaveRequest {
            step, col_query, col_selected, col_name, req_name,
        } => overlay::render_save_request(
            frame, app, step, col_query, col_selected, col_name, req_name, area,
        ),
        Overlay::EnvManager(state) => overlay::render_env_manager(frame, app, state, area),
    }
}
