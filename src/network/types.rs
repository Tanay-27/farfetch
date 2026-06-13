use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Head => "HEAD",
            Self::Options => "OPTIONS",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Get => Self::Post,
            Self::Post => Self::Put,
            Self::Put => Self::Patch,
            Self::Patch => Self::Delete,
            Self::Delete => Self::Head,
            Self::Head => Self::Options,
            Self::Options => Self::Get,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Get => Self::Options,
            Self::Options => Self::Head,
            Self::Head => Self::Delete,
            Self::Delete => Self::Patch,
            Self::Patch => Self::Put,
            Self::Put => Self::Post,
            Self::Post => Self::Get,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub body: String,
    pub elapsed_ms: u64,
    pub size_bytes: usize,
    pub content_type: String,
}
