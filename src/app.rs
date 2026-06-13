use std::path::PathBuf;

use tokio::sync::mpsc;

use crate::{
    config::types::{Config, Environments},
    events::AppEvent,
    fuzzy::FuzzyOverlay,
    network::types::HttpRequest,
    state::{focus::FocusedPane, request_state::RequestState, response_state::ResponseState},
    storage::types::{Collection, SavedRequest},
};

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Flat sidebar row — collection headers are visual dividers, not selectable.
#[derive(Debug, Clone)]
pub enum SidebarItem {
    CollectionHeader { name: String },
    Request { name: String, col_idx: usize, req_idx: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveStep {
    PickCollection,
    NameRequest,
}

// ── Header editing ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub enum HeaderEditMode {
    #[default]
    None,
    NewKey { input: String },
    NewValue { key: String, input: String },
    EditValue { idx: usize, input: String },
}

// ── Env manager ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum EnvPane {
    EnvList,
    VarList,
}

#[derive(Debug, Clone)]
pub enum EnvEditMode {
    None,
    NewEnvName { input: String },
    NewVarKey { input: String },
    NewVarValue { key: String, input: String },
    EditVarValue { var_key: String, input: String },
}

#[derive(Debug, Clone)]
pub struct EnvManagerState {
    pub focus: EnvPane,
    pub env_cursor: usize,
    pub var_cursor: usize,
    pub edit: EnvEditMode,
}

impl EnvManagerState {
    fn new(active_env: &str, env_names: &[String]) -> Self {
        let cursor = env_names.iter().position(|n| n == active_env).unwrap_or(0);
        Self {
            focus: EnvPane::EnvList,
            env_cursor: cursor,
            var_cursor: 0,
            edit: EnvEditMode::None,
        }
    }
}

// ── Overlay ───────────────────────────────────────────────────────────────────

pub enum Overlay {
    None,
    Help,
    SaveRequest {
        step: SaveStep,
        col_query: String,
        col_selected: Option<usize>,
        col_name: String,
        req_name: String,
    },
    FuzzySearch(FuzzyOverlay),
    EnvManager(EnvManagerState),
}

impl Default for Overlay {
    fn default() -> Self {
        Overlay::None
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct App {
    pub input_mode: InputMode,
    pub focused_pane: FocusedPane,
    pub request: RequestState,
    pub response: ResponseState,
    pub loading: bool,
    pub waiting_for_editor: bool,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub git_branch: Option<String>,
    pub active_env: String,
    pub overlay: Overlay,
    pub config: Config,
    pub environments: Environments,
    pub farfetch_dir: PathBuf,
    pub collections: Vec<Collection>,
    pub history: Vec<SavedRequest>,
    pub sidebar_items: Vec<SidebarItem>,
    pub sidebar_cursor: usize,
    pub header_edit: HeaderEditMode,
    pub client: reqwest::Client,
    pub tx: mpsc::Sender<AppEvent>,
}

impl App {
    pub fn new(tx: mpsc::Sender<AppEvent>, client: reqwest::Client) -> Self {
        let mut request = RequestState::default();
        request
            .headers
            .push(("Content-Type".to_string(), "application/json".to_string()));
        Self {
            input_mode: InputMode::Normal,
            focused_pane: FocusedPane::default(),
            request,
            response: ResponseState::default(),
            loading: false,
            waiting_for_editor: false,
            should_quit: false,
            status_message: None,
            git_branch: None,
            active_env: "local".to_string(),
            overlay: Overlay::None,
            config: Config::default(),
            environments: Environments::default(),
            farfetch_dir: PathBuf::from(".farfetch"),
            collections: Vec::new(),
            history: Vec::new(),
            sidebar_items: Vec::new(),
            sidebar_cursor: 0,
            header_edit: HeaderEditMode::None,
            client,
            tx,
        }
    }

    // ── Env helpers ───────────────────────────────────────────────────────────

    pub fn env_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.environments.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn env_vars_sorted(&self, env_name: &str) -> Vec<(String, String)> {
        let Some(vars) = self.environments.get(env_name) else {
            return vec![];
        };
        let mut pairs: Vec<(String, String)> =
            vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));
        pairs
    }

    fn save_environments(&self) -> anyhow::Result<()> {
        let path = self.farfetch_dir.join("environments.json");
        let s = serde_json::to_string_pretty(&self.environments)?;
        std::fs::write(path, s)?;
        Ok(())
    }

    fn save_environments_or_warn(&mut self) {
        if let Err(e) = self.save_environments() {
            self.status_message = Some(format!("Save failed: {e}"));
        }
    }

    pub fn open_env_manager(&mut self) {
        let names = self.env_names();
        self.overlay = Overlay::EnvManager(EnvManagerState::new(&self.active_env, &names));
    }

    // ── Sidebar ───────────────────────────────────────────────────────────────

    pub fn rebuild_sidebar(&mut self) {
        self.sidebar_items.clear();
        for (col_idx, col) in self.collections.iter().enumerate() {
            self.sidebar_items
                .push(SidebarItem::CollectionHeader { name: col.name.clone() });
            for (req_idx, req) in col.requests.iter().enumerate() {
                self.sidebar_items.push(SidebarItem::Request {
                    name: req.name.clone(),
                    col_idx,
                    req_idx,
                });
            }
        }
        let max = self.selectable_sidebar_count().saturating_sub(1);
        if self.sidebar_cursor > max {
            self.sidebar_cursor = 0;
        }
    }

    fn selectable_sidebar_count(&self) -> usize {
        self.sidebar_items
            .iter()
            .filter(|i| matches!(i, SidebarItem::Request { .. }))
            .count()
    }

    pub fn sidebar_request_index_to_flat(&self, req_cursor: usize) -> usize {
        let mut count = 0;
        for (i, item) in self.sidebar_items.iter().enumerate() {
            if matches!(item, SidebarItem::Request { .. }) {
                if count == req_cursor {
                    return i;
                }
                count += 1;
            }
        }
        0
    }

    pub fn sidebar_delete_selected(&mut self) {
        let flat = self.sidebar_request_index_to_flat(self.sidebar_cursor);
        if let Some(SidebarItem::Request { col_idx, req_idx, .. }) = self.sidebar_items.get(flat) {
            let (col_idx, req_idx) = (*col_idx, *req_idx);
            let col = &mut self.collections[col_idx];
            let name = col.requests[req_idx].name.clone();
            col.requests.remove(req_idx);
            // Remove collection if now empty
            if col.requests.is_empty() {
                self.collections.remove(col_idx);
            }
            let path = self.farfetch_dir.join("collections.json");
            if let Err(e) = crate::storage::collections::save(&path, &self.collections) {
                self.status_message = Some(format!("Save failed: {e}"));
            } else {
                self.status_message = Some(format!("Deleted «{}»", name));
            }
            self.rebuild_sidebar();
        }
    }

    pub fn sidebar_load_selected(&mut self) {
        let flat = self.sidebar_request_index_to_flat(self.sidebar_cursor);
        if let Some(SidebarItem::Request { col_idx, req_idx, .. }) = self.sidebar_items.get(flat) {
            let (col_idx, req_idx) = (*col_idx, *req_idx);
            if let Some(req) = self
                .collections
                .get(col_idx)
                .and_then(|c| c.requests.get(req_idx))
            {
                self.request.method = req.method.clone();
                self.request.url = req.url.clone();
                self.request.headers = req.headers.clone();
                self.request.body = req.body.clone();
                self.focused_pane = FocusedPane::Url;
                self.status_message = Some(format!("Loaded «{}»", req.name));
            }
        }
    }

    // ── Pane navigation ───────────────────────────────────────────────────────

    pub fn next_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Sidebar   => FocusedPane::Url,
            FocusedPane::Url       => FocusedPane::SendButton,
            FocusedPane::SendButton => FocusedPane::Params,
            FocusedPane::Params    => FocusedPane::Body,
            FocusedPane::Body      => FocusedPane::Response,
            FocusedPane::Response  => FocusedPane::Sidebar,
        };
        self.input_mode = InputMode::Normal;
    }

    pub fn prev_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            FocusedPane::Sidebar   => FocusedPane::Response,
            FocusedPane::Url       => FocusedPane::Sidebar,
            FocusedPane::SendButton => FocusedPane::Url,
            FocusedPane::Params    => FocusedPane::SendButton,
            FocusedPane::Body      => FocusedPane::Params,
            FocusedPane::Response  => FocusedPane::Body,
        };
        self.input_mode = InputMode::Normal;
    }

    // ── Scrolling ─────────────────────────────────────────────────────────────

    pub fn scroll_down(&mut self) {
        match self.focused_pane {
            FocusedPane::Sidebar => {
                let max = self.selectable_sidebar_count().saturating_sub(1);
                if self.sidebar_cursor < max {
                    self.sidebar_cursor += 1;
                }
            }
            FocusedPane::Response => {
                let max = self.response.lines.len().saturating_sub(1);
                if self.response.scroll_offset < max {
                    self.response.scroll_offset += 1;
                }
            }
            FocusedPane::Params => {
                let max = self.request.headers.len().saturating_sub(1);
                if self.request.selected_header < max {
                    self.request.selected_header += 1;
                }
            }
            _ => {}
        }
    }

    pub fn scroll_up(&mut self) {
        match self.focused_pane {
            FocusedPane::Sidebar => {
                self.sidebar_cursor = self.sidebar_cursor.saturating_sub(1);
            }
            FocusedPane::Response => {
                self.response.scroll_offset = self.response.scroll_offset.saturating_sub(1);
            }
            FocusedPane::Params => {
                self.request.selected_header = self.request.selected_header.saturating_sub(1);
            }
            _ => {}
        }
    }

    pub fn cycle_method_forward(&mut self) { self.request.method = self.request.method.next(); }
    pub fn cycle_method_back(&mut self)    { self.request.method = self.request.method.prev(); }

    // ── Header editing ────────────────────────────────────────────────────────

    pub fn handle_header_key(&mut self, key: crossterm::event::KeyEvent) {
        use crossterm::event::{KeyCode, KeyModifiers};
        use std::mem;

        let mut edit = mem::take(&mut self.header_edit);
        match &mut edit {
            HeaderEditMode::None => match key.code {
                KeyCode::Char('n') if key.modifiers.is_empty() => {
                    edit = HeaderEditMode::NewKey { input: String::new() };
                }
                KeyCode::Char('d') if key.modifiers.is_empty() => {
                    if !self.request.headers.is_empty() {
                        self.request.headers.remove(self.request.selected_header);
                        let max = self.request.headers.len().saturating_sub(1);
                        if self.request.selected_header > max {
                            self.request.selected_header = max;
                        }
                    }
                }
                KeyCode::Enter => {
                    if let Some((_, v)) = self.request.headers.get(self.request.selected_header) {
                        edit = HeaderEditMode::EditValue {
                            idx: self.request.selected_header,
                            input: v.clone(),
                        };
                    }
                }
                _ => {}
            },

            HeaderEditMode::NewKey { input } => match key.code {
                KeyCode::Esc => edit = HeaderEditMode::None,
                KeyCode::Enter => {
                    let k = input.trim().to_string();
                    if !k.is_empty() {
                        edit = HeaderEditMode::NewValue { key: k, input: String::new() };
                    }
                }
                KeyCode::Backspace => { input.pop(); }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(c);
                }
                _ => {}
            },

            HeaderEditMode::NewValue { key: hkey, input } => {
                let hkey = hkey.clone();
                match key.code {
                    KeyCode::Esc => edit = HeaderEditMode::None,
                    KeyCode::Enter => {
                        self.request.headers.push((hkey, input.clone()));
                        self.request.selected_header = self.request.headers.len() - 1;
                        edit = HeaderEditMode::None;
                    }
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }

            HeaderEditMode::EditValue { idx, input } => {
                let idx = *idx;
                match key.code {
                    KeyCode::Esc => edit = HeaderEditMode::None,
                    KeyCode::Enter => {
                        if let Some(header) = self.request.headers.get_mut(idx) {
                            header.1 = input.clone();
                        }
                        edit = HeaderEditMode::None;
                    }
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }
        }
        self.header_edit = edit;
    }

    // ── Text editing ──────────────────────────────────────────────────────────

    pub fn enter_editing(&mut self) {
        match self.focused_pane {
            FocusedPane::Url | FocusedPane::Body => self.input_mode = InputMode::Editing,
            FocusedPane::Sidebar => self.sidebar_load_selected(),
            _ => {}
        }
    }

    pub fn insert_char(&mut self, c: char) {
        match self.focused_pane {
            FocusedPane::Url  => self.request.url.push(c),
            FocusedPane::Body => self.request.body.push(c),
            _ => {}
        }
    }

    pub fn backspace_char(&mut self) {
        match self.focused_pane {
            FocusedPane::Url  => { self.request.url.pop(); }
            FocusedPane::Body => { self.request.body.pop(); }
            _ => {}
        }
    }

    // ── HTTP ──────────────────────────────────────────────────────────────────

    pub fn fire_request(&mut self) {
        if self.loading || self.request.url.is_empty() { return; }

        self.input_mode = InputMode::Normal;

        let env_vars = self.environments.get(&self.active_env).cloned().unwrap_or_default();
        let url      = crate::config::environment::resolve_vars(&self.request.url, &env_vars);
        let body_str = crate::config::environment::resolve_vars(&self.request.body, &env_vars);
        let headers: Vec<(String, String)> = self
            .request
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), crate::config::environment::resolve_vars(v, &env_vars)))
            .collect();

        let req = HttpRequest {
            method: self.request.method.clone(),
            url,
            headers,
            body: if body_str.is_empty() { None } else { Some(body_str) },
        };

        // Snapshot the raw request at fire time so history records what was sent,
        // not whatever the user edits while the response is in flight.
        let snapshot = SavedRequest {
            name:    String::new(),
            method:  self.request.method.clone(),
            url:     self.request.url.clone(),
            headers: self.request.headers.clone(),
            body:    self.request.body.clone(),
        };

        self.loading = true;
        self.focused_pane = FocusedPane::Response;
        self.status_message = None;

        let client = self.client.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match crate::network::client::execute_request(&client, req).await {
                Ok(resp) => { let _ = tx.send(AppEvent::HttpResponse(resp, snapshot)).await; }
                Err(e)   => { let _ = tx.send(AppEvent::Error(e.to_string())).await; }
            }
        });
    }

    // ── Clipboard ─────────────────────────────────────────────────────────────

    pub fn yank_response(&mut self) {
        if self.response.body.is_empty() { return; }
        match crate::clipboard::copy_to_clipboard(&self.response.body) {
            Ok(()) => self.status_message = Some("Yanked response to clipboard".to_string()),
            Err(e) => self.status_message = Some(format!("Clipboard error: {e}")),
        }
    }

    pub fn paste_from_clipboard(&mut self) {
        if let Some(text) = crate::clipboard::get_clipboard() {
            let trimmed = text.trim().to_string();
            if let Some(parsed) = crate::curl::parse_curl(&trimmed) {
                self.request = parsed;
                self.status_message = Some("cURL parsed — fields populated".to_string());
            } else {
                match self.focused_pane {
                    FocusedPane::Url  => self.request.url.push_str(&trimmed),
                    FocusedPane::Body => self.request.body.push_str(&text),
                    _ => {}
                }
            }
        }
    }

    pub fn copy_as_curl(&mut self) {
        let env_vars = self.environments.get(&self.active_env).cloned().unwrap_or_default();
        let url      = crate::config::environment::resolve_vars(&self.request.url, &env_vars);
        let body_str = crate::config::environment::resolve_vars(&self.request.body, &env_vars);

        let mut parts = vec!["curl".to_string()];
        if self.request.method.as_str() != "GET" {
            parts.push(format!("-X {}", self.request.method.as_str()));
        }
        for (k, v) in &self.request.headers {
            let vr = crate::config::environment::resolve_vars(v, &env_vars);
            parts.push(format!("-H {}", shell_escape(&format!("{k}: {vr}"))));
        }
        if !body_str.is_empty() {
            parts.push(format!("-d {}", shell_escape(&body_str)));
        }
        parts.push(shell_escape(&url));

        let curl = parts.join(" \\\n  ");
        match crate::clipboard::copy_to_clipboard(&curl) {
            Ok(()) => self.status_message = Some("cURL copied to clipboard".to_string()),
            Err(e) => self.status_message = Some(format!("Clipboard error: {e}")),
        }
    }

    // ── Editor ────────────────────────────────────────────────────────────────

    pub fn open_editor(&mut self) {
        if self.waiting_for_editor { return; }
        self.waiting_for_editor = true;
        let editor = self.config.default_editor.clone();
        let tx     = self.tx.clone();
        let body   = self.request.body.clone();
        if let Err(e) = crate::editor::open_in_editor(&body, &editor, tx) {
            self.status_message = Some(format!("Editor error: {e}"));
            self.waiting_for_editor = false;
        }
    }

    // ── Save overlay ──────────────────────────────────────────────────────────

    pub fn open_save_overlay(&mut self) {
        self.overlay = Overlay::SaveRequest {
            step: SaveStep::PickCollection,
            col_query: String::new(),
            col_selected: None,
            col_name: String::new(),
            req_name: String::new(),
        };
    }

    pub fn matching_collections(&self, query: &str) -> Vec<usize> {
        let q = query.to_lowercase();
        self.collections
            .iter()
            .enumerate()
            .filter(|(_, c)| q.is_empty() || c.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    fn do_save_request(&mut self, col_name: String, req_name: String) {
        let col_idx = if let Some(i) =
            self.collections.iter().position(|c| c.name.eq_ignore_ascii_case(&col_name))
        {
            i
        } else {
            self.collections.push(Collection { name: col_name.clone(), requests: Vec::new() });
            self.collections.len() - 1
        };

        let col = &mut self.collections[col_idx];
        if let Some(e) = col.requests.iter_mut().find(|r| r.name == req_name) {
            e.method  = self.request.method.clone();
            e.url     = self.request.url.clone();
            e.headers = self.request.headers.clone();
            e.body    = self.request.body.clone();
        } else {
            col.requests.push(SavedRequest {
                name:    req_name.clone(),
                method:  self.request.method.clone(),
                url:     self.request.url.clone(),
                headers: self.request.headers.clone(),
                body:    self.request.body.clone(),
            });
        }

        let path = self.farfetch_dir.join("collections.json");
        if let Err(e) = crate::storage::collections::save(&path, &self.collections) {
            self.status_message = Some(format!("Save failed: {e}"));
        } else {
            self.status_message = Some(format!("Saved «{req_name}» → {col_name}"));
        }
        self.rebuild_sidebar();
    }

    // ── Fuzzy search ──────────────────────────────────────────────────────────

    pub fn open_fuzzy_overlay(&mut self) {
        let mut entries: Vec<SavedRequest> = self.history.clone();
        for col in &self.collections {
            entries.extend(col.requests.clone());
        }
        self.overlay = Overlay::FuzzySearch(FuzzyOverlay::new(entries));
    }

    fn append_history(&mut self, entry: SavedRequest) {
        let path = self.farfetch_dir.join("history.json");
        self.history.push(entry.clone());
        let _ = crate::storage::history::append(&path, entry);
        if self.history.len() > 500 {
            self.history.drain(0..self.history.len() - 500);
        }
    }

    // ── Overlay key dispatch ──────────────────────────────────────────────────

    pub fn handle_overlay_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};
        use std::mem;

        let mut overlay = mem::take(&mut self.overlay);
        let consumed = match &mut overlay {
            Overlay::None => false,

            Overlay::Help => { overlay = Overlay::None; true }

            Overlay::SaveRequest { step, col_query, col_selected, col_name, req_name } => {
                match step {
                    SaveStep::PickCollection => match key.code {
                        KeyCode::Esc => overlay = Overlay::None,
                        KeyCode::Enter => {
                            let name = if col_query.is_empty() {
                                if let Some(idx) = col_selected {
                                    self.collections[*idx].name.clone()
                                } else {
                                    overlay = Overlay::None;
                                    self.overlay = overlay;
                                    return true;
                                }
                            } else {
                                let matches = self.matching_collections(col_query);
                                if let Some(idx) = col_selected {
                                    self.collections[*idx].name.clone()
                                } else if matches.len() == 1 {
                                    self.collections[matches[0]].name.clone()
                                } else {
                                    col_query.clone()
                                }
                            };
                            *col_name = name;
                            *step = SaveStep::NameRequest;
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let m = self.matching_collections(col_query);
                            *col_selected = Some(match col_selected {
                                None => m.first().copied().unwrap_or(0),
                                Some(cur) => {
                                    let pos = m.iter().position(|&i| i == *cur).unwrap_or(0);
                                    m.get(pos + 1).copied().unwrap_or(*cur)
                                }
                            });
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            let m = self.matching_collections(col_query);
                            *col_selected = Some(match col_selected {
                                None => m.last().copied().unwrap_or(0),
                                Some(cur) => {
                                    let pos = m.iter().position(|&i| i == *cur).unwrap_or(0);
                                    m.get(pos.saturating_sub(1)).copied().unwrap_or(*cur)
                                }
                            });
                        }
                        KeyCode::Backspace => { col_query.pop(); *col_selected = None; }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            col_query.push(c); *col_selected = None;
                        }
                        _ => {}
                    },
                    SaveStep::NameRequest => match key.code {
                        KeyCode::Esc => *step = SaveStep::PickCollection,
                        KeyCode::Enter if !req_name.is_empty() => {
                            let (cn, rn) = (col_name.clone(), req_name.clone());
                            self.overlay = Overlay::None;
                            self.do_save_request(cn, rn);
                            return true;
                        }
                        KeyCode::Backspace => { req_name.pop(); }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            req_name.push(c);
                        }
                        _ => {}
                    },
                }
                true
            }

            Overlay::FuzzySearch(fuzzy) => {
                match key.code {
                    KeyCode::Esc => overlay = Overlay::None,
                    KeyCode::Enter => {
                        if let Some(e) = fuzzy.selected_entry() {
                            self.request.method  = e.method.clone();
                            self.request.url     = e.url.clone();
                            self.request.headers = e.headers.clone();
                            self.request.body    = e.body.clone();
                        }
                        overlay = Overlay::None;
                    }
                    KeyCode::Up | KeyCode::Char('k') => fuzzy.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => fuzzy.move_down(),
                    KeyCode::Backspace => {
                        let mut q = fuzzy.query.clone(); q.pop();
                        fuzzy.update_query(&q);
                    }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let mut q = fuzzy.query.clone(); q.push(c);
                        fuzzy.update_query(&q);
                    }
                    _ => {}
                }
                true
            }

            Overlay::EnvManager(state) => {
                // Extract state, handle key, may replace overlay with None
                let mut st = state.clone();
                let close = self.handle_env_key(&mut st, key);
                if close {
                    overlay = Overlay::None;
                } else {
                    overlay = Overlay::EnvManager(st);
                }
                true
            }
        };
        self.overlay = overlay;
        consumed
    }

    /// Returns true if the overlay should close.
    fn handle_env_key(
        &mut self,
        state: &mut EnvManagerState,
        key: crossterm::event::KeyEvent,
    ) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        let names = self.env_names();

        match &mut state.edit {
            EnvEditMode::None => match state.focus {
                EnvPane::EnvList => match key.code {
                    KeyCode::Esc => return true,
                    KeyCode::Tab => { state.focus = EnvPane::VarList; state.var_cursor = 0; }
                    KeyCode::Up | KeyCode::Char('k') => {
                        state.env_cursor = state.env_cursor.saturating_sub(1);
                        state.var_cursor = 0;
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !names.is_empty() && state.env_cursor < names.len() - 1 {
                            state.env_cursor += 1;
                            state.var_cursor = 0;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(name) = names.get(state.env_cursor) {
                            self.active_env = name.clone();
                            self.status_message =
                                Some(format!("Switched to «{}» environment", name));
                        }
                    }
                    KeyCode::Char('n') if key.modifiers.is_empty() => {
                        state.edit = EnvEditMode::NewEnvName { input: String::new() };
                    }
                    KeyCode::Char('d') if key.modifiers.is_empty() => {
                        if let Some(name) = names.get(state.env_cursor) {
                            if name == &self.active_env {
                                self.status_message =
                                    Some("Can't delete the active environment".to_string());
                            } else {
                                let name = name.clone();
                                self.environments.remove(&name);
                                self.save_environments_or_warn();
                                state.env_cursor = state.env_cursor.saturating_sub(1);
                            }
                        }
                    }
                    _ => {}
                },

                EnvPane::VarList => {
                    let env_name = names.get(state.env_cursor).cloned().unwrap_or_default();
                    let vars = self.env_vars_sorted(&env_name);
                    match key.code {
                        KeyCode::Esc => return true,
                        KeyCode::Tab => { state.focus = EnvPane::EnvList; }
                        KeyCode::Up | KeyCode::Char('k') => {
                            state.var_cursor = state.var_cursor.saturating_sub(1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if !vars.is_empty() && state.var_cursor < vars.len() - 1 {
                                state.var_cursor += 1;
                            }
                        }
                        KeyCode::Enter => {
                            if let Some((k, v)) = vars.get(state.var_cursor) {
                                state.edit = EnvEditMode::EditVarValue {
                                    var_key: k.clone(),
                                    input: v.clone(),
                                };
                            }
                        }
                        KeyCode::Char('n') if key.modifiers.is_empty() => {
                            state.edit = EnvEditMode::NewVarKey { input: String::new() };
                        }
                        KeyCode::Char('d') if key.modifiers.is_empty() => {
                            if let Some((k, _)) = vars.get(state.var_cursor) {
                                let k = k.clone();
                                if let Some(env) = self.environments.get_mut(&env_name) {
                                    env.remove(&k);
                                }
                                self.save_environments_or_warn();
                                state.var_cursor = state.var_cursor.saturating_sub(1);
                            }
                        }
                        _ => {}
                    }
                }
            },

            EnvEditMode::NewEnvName { input } => match key.code {
                KeyCode::Esc => state.edit = EnvEditMode::None,
                KeyCode::Enter => {
                    let name = input.trim().to_string();
                    if !name.is_empty() && !self.environments.contains_key(&name) {
                        self.environments.insert(name.clone(), Default::default());
                        self.save_environments_or_warn();
                        let names = self.env_names();
                        state.env_cursor = names.iter().position(|n| n == &name).unwrap_or(0);
                    }
                    state.edit = EnvEditMode::None;
                }
                KeyCode::Backspace => { input.pop(); }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(c);
                }
                _ => {}
            },

            EnvEditMode::NewVarKey { input } => match key.code {
                KeyCode::Esc => state.edit = EnvEditMode::None,
                KeyCode::Enter => {
                    let k = input.trim().to_string();
                    if !k.is_empty() {
                        state.edit = EnvEditMode::NewVarValue { key: k, input: String::new() };
                    }
                }
                KeyCode::Backspace => { input.pop(); }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    input.push(c);
                }
                _ => {}
            },

            EnvEditMode::NewVarValue { key: var_key, input } => {
                let var_key = var_key.clone();
                let env_name = names.get(state.env_cursor).cloned().unwrap_or_default();
                match key.code {
                    KeyCode::Esc => state.edit = EnvEditMode::None,
                    KeyCode::Enter => {
                        let val = input.clone();
                        self.environments.entry(env_name).or_default().insert(var_key, val);
                        self.save_environments_or_warn();
                        state.edit = EnvEditMode::None;
                    }
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }

            EnvEditMode::EditVarValue { var_key, input } => {
                let var_key = var_key.clone();
                let env_name = names.get(state.env_cursor).cloned().unwrap_or_default();
                match key.code {
                    KeyCode::Esc => state.edit = EnvEditMode::None,
                    KeyCode::Enter => {
                        let val = input.clone();
                        if let Some(env) = self.environments.get_mut(&env_name) {
                            env.insert(var_key, val);
                        }
                        self.save_environments_or_warn();
                        state.edit = EnvEditMode::None;
                    }
                    KeyCode::Backspace => { input.pop(); }
                    KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                        input.push(c);
                    }
                    _ => {}
                }
            }
        }

        false
    }

    // ── Async event handler ───────────────────────────────────────────────────

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::HttpResponse(resp, snapshot) => {
                self.loading = false;
                self.response.status       = Some(resp.status);
                self.response.status_text  = resp.status_text;
                self.response.elapsed_ms   = resp.elapsed_ms;
                self.response.size_bytes   = resp.size_bytes;
                self.response.content_type = resp.content_type;
                self.response.set_body(resp.body);
                if self.response.content_type.contains("json") {
                    self.response.highlighted_lines =
                        crate::ui::highlight::colorize_json(&self.response.body);
                }
                self.append_history(snapshot);
            }
            AppEvent::FileChanged(content) => {
                self.request.body = content.trim_end().to_string();
                self.waiting_for_editor = false;
                self.status_message = Some("Body updated from editor".to_string());
            }
            AppEvent::EditorClosed => {
                self.waiting_for_editor = false;
            }
            AppEvent::Error(msg) => {
                self.loading = false;
                self.status_message = Some(format!("Error: {msg}"));
            }
        }
    }
}
