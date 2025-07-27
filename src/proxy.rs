use axum::{
    extract::{Query, State},
    http::{HeaderMap, Method, StatusCode, Uri},
    response::Response,
    body::Body,
};
use std::collections::HashMap;

use crate::middleware::AppState;

pub async fn proxy_request(
    State(app_state): State<AppState>,
    method: Method,
    uri: Uri,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, StatusCode> {
    let (_, http_client, config) = app_state.as_ref();
    
    let target_url = build_target_url(&config.target_base_url, &uri, &query_params);
    
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    
    let mut request_builder = http_client.request(method, &target_url);
    
    request_builder = forward_headers(request_builder, &headers);
    
    if let Some(api_key) = &config.api_key {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
    }
    
    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes);
    }
    
    let response = request_builder
        .send()
        .await
        .map_err(|err| {
            tracing::error!("Proxy request failed: {}", err);
            StatusCode::BAD_GATEWAY
        })?;
    
    build_axum_response(response).await
}

fn build_target_url(base_url: &str, uri: &Uri, query_params: &HashMap<String, String>) -> String {
    let path = uri.path();
    let target_path = path.strip_prefix("/api").unwrap_or(path);
    
    let query = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", serde_urlencoded::to_string(query_params).unwrap_or_default())
    };
    
    format!("{}{}{}", base_url, target_path, query)
}

fn forward_headers(
    mut request_builder: reqwest::RequestBuilder,
    headers: &HeaderMap,
) -> reqwest::RequestBuilder {
    for (name, value) in headers {
        if name != "host" && name != "authorization" {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name, value_str);
            }
        }
    }
    request_builder
}

async fn build_axum_response(response: reqwest::Response) -> Result<Response, StatusCode> {
    let status = response.status();
    let response_headers = response.headers().clone();
    
    let response_body = response
        .bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    
    let mut axum_response = Response::builder().status(status);
    
    for (name, value) in response_headers {
        if let Some(name) = name {
            axum_response = axum_response.header(name, value);
        }
    }
    
    axum_response
        .body(Body::from(response_body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
} 