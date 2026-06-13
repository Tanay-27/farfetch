use std::collections::HashMap;

use glob::Pattern;

pub fn resolve_environment(branch: &str, mapping: &HashMap<String, String>) -> String {
    if let Some(env) = mapping.get(branch) {
        return env.clone();
    }
    for (pattern, env) in mapping {
        if Pattern::new(pattern).map(|p| p.matches(branch)).unwrap_or(false) {
            return env.clone();
        }
    }
    "local".to_string()
}

/// Returns the pattern that matched the branch, e.g. `"feature/*"`.
/// Returns `None` if no rule matched (fell through to default).
pub fn matched_pattern<'a>(branch: &str, mapping: &'a HashMap<String, String>) -> Option<&'a str> {
    if mapping.contains_key(branch) {
        return Some(mapping.get_key_value(branch).unwrap().0.as_str());
    }
    for (pattern, _) in mapping {
        if Pattern::new(pattern).map(|p| p.matches(branch)).unwrap_or(false) {
            return Some(pattern.as_str());
        }
    }
    None
}

pub fn resolve_vars(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}
