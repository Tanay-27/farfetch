use ratatui::text::Line;

#[derive(Debug, Clone, Default)]
pub struct ResponseState {
    pub status: Option<u16>,
    pub status_text: String,
    pub body: String,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
    pub content_type: String,
    pub scroll_offset: usize,
    pub lines: Vec<String>,
    /// Pre-rendered highlighted lines for JSON bodies. Computed once on response
    /// arrival to avoid re-parsing the full body on every render frame.
    pub highlighted_lines: Vec<Line<'static>>,
}

impl ResponseState {
    pub fn set_body(&mut self, body: String) {
        self.lines = body.lines().map(|l| l.to_string()).collect();
        self.body = body;
        self.scroll_offset = 0;
        self.highlighted_lines.clear();
    }
}
