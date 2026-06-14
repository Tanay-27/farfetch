use std::time::Instant;

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use crate::network::types::{HttpMethod, HttpRequest, HttpResponse};

pub fn build_client(danger_accept_invalid_certs: bool) -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .danger_accept_invalid_certs(danger_accept_invalid_certs)
        .build()?)
}

pub async fn execute_request(client: &reqwest::Client, req: HttpRequest) -> Result<HttpResponse> {
    let method = match req.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    let mut header_map = HeaderMap::new();
    for (k, v) in &req.headers {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            header_map.insert(name, val);
        }
    }

    let mut builder = client.request(method, &req.url).headers(header_map);
    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    let start = Instant::now();
    let resp = builder.send().await?;
    let elapsed_ms = start.elapsed().as_millis() as u64;

    let status = resp.status().as_u16();
    let status_text = resp
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body_raw = resp.text().await?;
    let size_bytes = body_raw.len();

    // Pretty-print JSON if possible
    let body = if content_type.contains("application/json") || content_type.contains("/json") {
        serde_json::from_str::<serde_json::Value>(&body_raw)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or(body_raw)
    } else {
        body_raw
    };

    Ok(HttpResponse {
        status,
        status_text,
        body,
        elapsed_ms,
        size_bytes,
        content_type,
    })
}
