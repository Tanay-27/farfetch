use crate::network::types::HttpMethod;
use crate::state::request_state::RequestState;

pub fn parse_curl(input: &str) -> Option<RequestState> {
    let input = input.trim();
    if !input.starts_with("curl") {
        return None;
    }

    let tokens = tokenize(input);
    let mut it = tokens.iter().peekable();
    it.next(); // skip "curl"

    let mut method: Option<HttpMethod> = None;
    let mut url: Option<String> = None;
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut body: Option<String> = None;

    while let Some(tok) = it.next() {
        match tok.as_str() {
            "-X" | "--request" => {
                if let Some(m) = it.next() {
                    method = parse_method(m);
                }
            }
            "-H" | "--header" => {
                if let Some(h) = it.next() {
                    if let Some((k, v)) = h.split_once(':') {
                        headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                if let Some(b) = it.next() {
                    body = Some(b.clone());
                }
            }
            "--json" => {
                if let Some(b) = it.next() {
                    body = Some(b.clone());
                    if !headers.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) {
                        headers.push(("Content-Type".to_string(), "application/json".to_string()));
                    }
                }
            }
            // Flags we silently skip
            "-L" | "--location" | "-s" | "--silent" | "-v" | "--verbose"
            | "--compressed" | "-k" | "--insecure" | "-i" | "--include"
            | "-f" | "--fail" | "-S" | "--show-error" => {}
            tok if !tok.starts_with('-') && url.is_none() => {
                url = Some(tok.to_string());
            }
            _ => {}
        }
    }

    let url = url?;
    let method = method.unwrap_or_else(|| {
        if body.is_some() {
            HttpMethod::Post
        } else {
            HttpMethod::Get
        }
    });

    Some(RequestState {
        method,
        url,
        headers,
        body: body.unwrap_or_default(),
        selected_header: 0,
    })
}

fn parse_method(s: &str) -> Option<HttpMethod> {
    match s.to_uppercase().as_str() {
        "GET" => Some(HttpMethod::Get),
        "POST" => Some(HttpMethod::Post),
        "PUT" => Some(HttpMethod::Put),
        "PATCH" => Some(HttpMethod::Patch),
        "DELETE" => Some(HttpMethod::Delete),
        "HEAD" => Some(HttpMethod::Head),
        "OPTIONS" => Some(HttpMethod::Options),
        _ => None,
    }
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '\\' if in_double => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\\' if !in_single && !in_double => {
                // Line continuation — skip trailing newline
                chars.next();
            }
            ' ' | '\t' | '\n' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}
