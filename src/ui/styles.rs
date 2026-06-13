use ratatui::style::{Color, Modifier, Style};

use crate::network::types::HttpMethod;

pub fn active_border() -> Style {
    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
}

pub fn inactive_border() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn editing_border() -> Style {
    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
}

pub fn method_style(method: &HttpMethod) -> Style {
    match method {
        HttpMethod::Get => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        HttpMethod::Post => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        HttpMethod::Put => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        HttpMethod::Patch => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        HttpMethod::Delete => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        HttpMethod::Head | HttpMethod::Options => {
            Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD)
        }
    }
}

pub fn status_style(status: u16) -> Style {
    match status {
        200..=299 => Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        300..=399 => Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        400..=499 => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        500..=599 => Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
        _ => Style::default().fg(Color::Gray),
    }
}
