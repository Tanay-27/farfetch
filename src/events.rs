use crate::network::types::HttpResponse;

pub enum AppEvent {
    HttpResponse(HttpResponse),
    FileChanged(String),
    Error(String),
}
