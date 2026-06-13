use std::path::Path;

use super::types::SavedRequest;

const MAX_HISTORY: usize = 500;

pub fn load(path: &Path) -> Vec<SavedRequest> {
    if !path.exists() {
        return vec![];
    }
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn append(path: &Path, entry: SavedRequest) -> anyhow::Result<()> {
    let mut entries = load(path);
    entries.push(entry);
    if entries.len() > MAX_HISTORY {
        entries.drain(0..entries.len() - MAX_HISTORY);
    }
    std::fs::write(path, serde_json::to_string_pretty(&entries)?)?;
    Ok(())
}
