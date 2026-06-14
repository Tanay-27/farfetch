use std::path::{Path, PathBuf};

use super::types::{Config, Environments};

fn find_farfetch_dir() -> PathBuf {
    let Ok(mut dir) = std::env::current_dir() else {
        return PathBuf::from(".farfetch");
    };
    loop {
        let candidate = dir.join(".farfetch");
        if candidate.exists() {
            return candidate;
        }
        if !dir.pop() {
            break;
        }
    }
    std::env::current_dir()
        .unwrap_or_default()
        .join(".farfetch")
}

pub fn load_or_create_config() -> (Config, PathBuf) {
    let dir = find_farfetch_dir();
    let _ = std::fs::create_dir_all(&dir);

    let config_path = dir.join("config.json");
    let config = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        let default = Config::default();
        let _ = serde_json::to_string_pretty(&default)
            .map(|s| std::fs::write(&config_path, s));
        default
    };

    (config, dir)
}

pub fn load_environments(farfetch_dir: &Path) -> Environments {
    let path = farfetch_dir.join("environments.json");
    if !path.exists() {
        let _ = std::fs::write(&path, "{}");
        return Environments::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
