use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

mod app;
mod clipboard;
mod config;
mod curl;
mod editor;
mod events;
mod fuzzy;
mod git;
mod network;
mod state;
mod storage;
mod ui;

use app::{App, InputMode, Overlay};
use events::AppEvent;
use state::focus::FocusedPane;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // ── Bootstrap config & git ────────────────────────────────────────────────
    let (cfg, farfetch_dir) = config::loader::load_or_create_config();
    let environments = config::loader::load_environments(&farfetch_dir);
    let branch = git::read_current_branch();
    let active_env = config::environment::resolve_environment(
        branch.as_deref().unwrap_or(""),
        &cfg.git_branch_mapping,
    );
    let client =
        network::client::build_client(cfg.danger_accept_invalid_certs).unwrap_or_default();

    // Load saved history + collections
    let history = storage::history::load(&farfetch_dir.join("history.json"));
    let collections = storage::collections::load(&farfetch_dir.join("collections.json"));

    // ── Terminal setup ────────────────────────────────────────────────────────
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ── Channels ──────────────────────────────────────────────────────────────
    let (app_tx, mut app_rx) = mpsc::channel::<AppEvent>(32);
    let (ev_tx, mut ev_rx) = mpsc::channel::<Event>(64);

    // Blocking thread that reads crossterm events
    let ev_tx2 = ev_tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    if ev_tx2.blocking_send(ev).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // ── Build app ─────────────────────────────────────────────────────────────
    let mut app = App::new(app_tx, client);
    app.git_branch = branch;
    app.active_env = active_env;
    app.config = cfg;
    app.environments = environments;
    app.farfetch_dir = farfetch_dir;
    app.history = history;
    app.collections = collections;
    app.rebuild_sidebar();

    // ── Run ───────────────────────────────────────────────────────────────────
    let result = run_app(&mut terminal, &mut app, &mut app_rx, &mut ev_rx).await;

    // ── Restore terminal ──────────────────────────────────────────────────────
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(e) = &result {
        eprintln!("Error: {e}");
    }
    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    app_rx: &mut mpsc::Receiver<AppEvent>,
    ev_rx: &mut mpsc::Receiver<Event>,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        tokio::select! {
            Some(ev) = ev_rx.recv() => {
                match ev {
                    Event::Key(key) => handle_key(app, key),
                    Event::Mouse(mouse) => handle_mouse(app, mouse, terminal.size()?),
                    _ => {}
                }
            }
            Some(app_ev) = app_rx.recv() => {
                app.handle_event(app_ev);
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent, term_size: ratatui::layout::Rect) {
    if mouse.kind != MouseEventKind::Down(crossterm::event::MouseButton::Left) {
        return;
    }
    // Sidebar is 24 cols. SEND button is the last 10 cols of the main area,
    // which starts at col 24. Vertically it's the request bar: rows 3..6.
    let btn_col_start = term_size.width.saturating_sub(10);
    let btn_row_start = 3u16;
    let btn_row_end = 6u16;
    if mouse.column >= btn_col_start
        && mouse.row >= btn_row_start
        && mouse.row < btn_row_end
    {
        app.fire_request();
    }
}

fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    // Overlays get first crack at every key
    if !matches!(app.overlay, Overlay::None) {
        app.handle_overlay_key(key);
        return;
    }

    // Global shortcuts — work in any input mode
    match key.code {
        KeyCode::F(1) | KeyCode::Char('?') => {
            app.overlay = Overlay::Help;
            return;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
            return;
        }
        _ => {}
    }

    // Header pane: intercept keys when editing or for add/delete/edit triggers
    if app.focused_pane == FocusedPane::Params {
        let in_edit = !matches!(app.header_edit, app::HeaderEditMode::None);
        let is_trigger = key.modifiers.is_empty()
            && matches!(key.code, KeyCode::Char('n') | KeyCode::Char('d') | KeyCode::Enter);
        if in_edit || is_trigger {
            app.handle_header_key(key);
            return;
        }
    }

    match app.input_mode {
        InputMode::Normal => match key.code {
            // ── Quit ────────────────────────────────────────────────────────
            KeyCode::Char('q') if key.modifiers.is_empty() => app.should_quit = true,

            // ── Pane navigation ─────────────────────────────────────────────
            KeyCode::Tab => app.next_pane(),
            KeyCode::BackTab => app.prev_pane(),

            // ── Scroll / method cycle ───────────────────────────────────────
            KeyCode::Char('j') if key.modifiers.is_empty() => app.scroll_down(),
            KeyCode::Char('k') if key.modifiers.is_empty() => app.scroll_up(),
            KeyCode::Char('d')
                if key.modifiers.is_empty()
                    && app.focused_pane == FocusedPane::Sidebar =>
            {
                app.sidebar_delete_selected()
            }
            KeyCode::Left if app.focused_pane == FocusedPane::Url => app.cycle_method_back(),
            KeyCode::Right if app.focused_pane == FocusedPane::Url => {
                app.cycle_method_forward()
            }

            // ── Send request ────────────────────────────────────────────────
            // Ctrl+Enter works in terminals that support it (kitty, Zed enhanced mode).
            // F5 is the universal fallback — works everywhere.
            // Enter on the SEND button fires the request.
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.fire_request()
            }
            KeyCode::F(5) => app.fire_request(),
            KeyCode::Enter if app.focused_pane == FocusedPane::SendButton => {
                app.fire_request()
            }

            // ── Enter editing ───────────────────────────────────────────────
            KeyCode::Enter => app.enter_editing(),

            // ── Clipboard / yank ────────────────────────────────────────────
            KeyCode::Char('Y') | KeyCode::Char('y')
                if key.modifiers.is_empty()
                    && app.focused_pane == FocusedPane::Response =>
            {
                app.yank_response()
            }
            KeyCode::Char('C') | KeyCode::Char('c')
                if key.modifiers.is_empty() =>
            {
                app.copy_as_curl()
            }

            // ── External editor ─────────────────────────────────────────────
            KeyCode::Char('e') | KeyCode::Char('E')
                if key.modifiers.is_empty()
                    && app.focused_pane == FocusedPane::Body =>
            {
                app.open_editor()
            }

            // ── Collections & history ───────────────────────────────────────
            // F4 = save (primary), Ctrl+S = alias
            KeyCode::F(4) => app.open_save_overlay(),
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_save_overlay()
            }
            // F3 = history search (primary), Ctrl+R = alias
            KeyCode::F(3) => app.open_fuzzy_overlay(),
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_fuzzy_overlay()
            }

            // ── Env manager ─────────────────────────────────────────────────
            // F2 = env manager (primary), Ctrl+E = alias
            KeyCode::F(2) => app.open_env_manager(),
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_env_manager()
            }


            _ => {}
        },

        InputMode::Editing => match key.code {
            KeyCode::Esc => app.input_mode = InputMode::Normal,
            KeyCode::Tab => {
                app.input_mode = InputMode::Normal;
                app.next_pane();
            }
            KeyCode::Backspace => app.backspace_char(),
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.input_mode = InputMode::Normal;
                app.fire_request();
            }
            // URL bar: Enter submits (like a browser address bar)
            KeyCode::Enter if app.focused_pane == FocusedPane::Url => {
                app.input_mode = InputMode::Normal;
                app.fire_request();
            }
            KeyCode::Enter => {
                if app.focused_pane == FocusedPane::Body {
                    app.insert_char('\n');
                } else {
                    app.input_mode = InputMode::Normal;
                }
            }
            // F6 = paste/cURL import (primary), Ctrl+V = alias
            KeyCode::F(6) => app.paste_from_clipboard(),
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.paste_from_clipboard();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.insert_char(c);
            }
            _ => {}
        },
    }
}
