use crate::storage::types::SavedRequest;

pub struct FuzzyOverlay {
    pub query: String,
    pub all_entries: Vec<SavedRequest>,
    pub filtered: Vec<usize>,
    pub selected: usize,
}

impl FuzzyOverlay {
    pub fn new(entries: Vec<SavedRequest>) -> Self {
        let count = entries.len();
        // Show newest first
        let filtered: Vec<usize> = (0..count).rev().collect();
        Self {
            query: String::new(),
            all_entries: entries,
            filtered,
            selected: 0,
        }
    }

    pub fn update_query(&mut self, query: &str) {
        self.query = query.to_string();
        let q = query.to_lowercase();
        self.filtered = self
            .all_entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.url.to_lowercase().contains(&q)
                    || e.name.to_lowercase().contains(&q)
                    || e.method.as_str().to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .rev()
            .collect();
        self.selected = 0;
    }

    pub fn selected_entry(&self) -> Option<&SavedRequest> {
        self.filtered.get(self.selected).map(|&i| &self.all_entries[i])
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }
}
