use crate::network::types::HttpMethod;

#[derive(Debug, Clone, Default)]
pub struct RequestState {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub selected_header: usize,
}
