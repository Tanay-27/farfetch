use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub git_branch_mapping: HashMap<String, String>,
    pub default_editor: String,
    pub danger_accept_invalid_certs: bool,
}

impl Default for Config {
    fn default() -> Self {
        let mut mapping = HashMap::new();
        mapping.insert("main".to_string(), "production".to_string());
        mapping.insert("release/*".to_string(), "uat".to_string());
        mapping.insert("feature/*".to_string(), "local".to_string());
        mapping.insert("bugfix/*".to_string(), "dev".to_string());
        Self {
            git_branch_mapping: mapping,
            default_editor: "zed".to_string(),
            danger_accept_invalid_certs: false,
        }
    }
}

/// env_name → { var_name → value }
pub type Environments = HashMap<String, HashMap<String, String>>;
