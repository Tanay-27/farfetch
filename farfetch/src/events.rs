use crate::network::types::HttpResponse;
use crate::storage::types::SavedRequest;

pub enum AppEvent {
    HttpResponse(HttpResponse, SavedRequest),
    FileChanged(String),
    EditorClosed,
    Error(String),
}
