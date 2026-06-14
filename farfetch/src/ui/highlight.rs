use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

pub fn colorize_json(json: &str) -> Vec<Line<'static>> {
    json.lines().map(colorize_line).collect()
}

fn colorize_line(line: &str) -> Line<'static> {
    let trimmed = line.trim_start();
    let indent = " ".repeat(line.len() - trimmed.len());

    // Pure structural lines
    if matches!(trimmed.trim_end_matches(','), "{" | "}" | "[" | "]" | "{}" | "[]") {
        return Line::from(Span::styled(
            line.to_owned(),
            Style::default().fg(Color::DarkGray),
        ));
    }

    // Key-value line starting with a quoted key
    if trimmed.starts_with('"') {
        if let Some(colon) = find_key_colon(trimmed) {
            let key_part = trimmed[..=colon].to_owned(); // "key":
            let rest = trimmed[colon + 1..].trim_start();
            let trailing = if rest.ends_with(',') { "," } else { "" };
            let value = rest.trim_end_matches(',');

            return Line::from(vec![
                Span::raw(indent),
                Span::styled(key_part, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                value_span(value, trailing),
            ]);
        }
    }

    // Bare values (array elements, etc.)
    if !trimmed.is_empty() {
        let trailing = if trimmed.ends_with(',') { "," } else { "" };
        let value = trimmed.trim_end_matches(',');
        return Line::from(vec![Span::raw(indent), value_span(value, trailing)]);
    }

    Line::from(line.to_owned())
}

fn value_span(value: &str, trailing: &str) -> Span<'static> {
    let color = if value.starts_with('"') {
        Color::Yellow
    } else if matches!(value, "true" | "false" | "null") {
        Color::Magenta
    } else if value.parse::<f64>().is_ok() {
        Color::Green
    } else if value.starts_with('{') || value.starts_with('[') {
        Color::DarkGray
    } else {
        Color::Reset
    };
    Span::styled(format!("{value}{trailing}"), Style::default().fg(color))
}

fn find_key_colon(s: &str) -> Option<usize> {
    let mut in_str = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_str => escaped = true,
            '"' => in_str = !in_str,
            ':' if !in_str => return Some(i),
            _ => {}
        }
    }
    None
}
