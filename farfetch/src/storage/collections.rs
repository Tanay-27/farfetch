use std::path::Path;

use super::types::Collection;

pub fn load(path: &Path) -> Vec<Collection> {
    if !path.exists() {
        return vec![];
    }
    let Ok(s) = std::fs::read_to_string(path) else {
        return vec![];
    };

    serde_json::from_str::<Vec<Collection>>(&s).unwrap_or_default()
}

pub fn save(path: &Path, collections: &[Collection]) -> anyhow::Result<()> {
    std::fs::write(path, serde_json::to_string_pretty(collections)?)?;
    Ok(())
}
