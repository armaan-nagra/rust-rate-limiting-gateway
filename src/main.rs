use axum::{
    routing::{any, post},
    extract::{State, Query},
    response::IntoResponse, http::{Request, HeaderMap, Method, Uri}, middleware::Next,
    Router, Json, body::Body
};
use serde::Serialize;
use uuid::Uuid;
use redis::{Client, Commands};
use axum::http::StatusCode;
use std::{sync::Arc, collections::HashMap};
use axum::middleware;
use reqwest::Client as HttpClient;
type AppState = Arc<(Arc<Client>, HttpClient, ProxyConfig)>;

#[derive(Clone)]
struct ProxyConfig {
    target_base_url: String,
    api_key: Option<String>,
    rate_limit: u32,
}

async fn rate_limiter(
    State(app_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .replace("Bearer ", "");

    if token.is_empty() {
        return (StatusCode::UNAUTHORIZED, "Missing token").into_response();
    }

    let (redis_client, _, config) = app_state.as_ref();
    let mut conn = redis_client.get_connection().unwrap();
    let key = format!("ratelimit:{}", token);
    let count: i32 = conn.incr(&key, 1).unwrap();
    let _: bool = conn.expire(&key, 60).unwrap();

    if count > config.rate_limit as i32 {
        return (StatusCode::TOO_MANY_REQUESTS, format!("Rate limit exceeded, rate is {} requests per minute", config.rate_limit)).into_response();
    }

    next.run(req).await
}

#[tokio::main]
async fn main() {
    // Logger
    tracing_subscriber::fmt::init();

    // Load configuration from environment variables
    let target_base_url = std::env::var("TARGET_API_URL")
        .unwrap_or_else(|_| "https://api.example.com".to_string());
    let api_key = std::env::var("TARGET_API_KEY").ok();
    let rate_limit = std::env::var("RATE_LIMIT")
        .unwrap_or_else(|_| "60".to_string())
        .parse::<u32>()
        .unwrap_or(60);

    println!("🚀 Starting API Rate Limiter Proxy");
    println!("📡 Target API: {}", target_base_url);
    println!("🚦 Rate Limit: {} requests per minute", rate_limit);
    println!("🔑 API Key: {}", if api_key.is_some() { "Configured" } else { "Not configured" });

    let redis_client = Arc::new(Client::open("redis://127.0.0.1").unwrap());
    let http_client = HttpClient::new();
    let config = ProxyConfig {
        target_base_url,
        api_key,
        rate_limit,
    };
    
    let app_state = Arc::new((redis_client, http_client, config));
    
    let protected_routes = Router::new()
        .route("/api/*path", any(proxy_request))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&app_state),
            rate_limiter,
        ));

    let app = Router::new()
        .route("/register", post(register))
        .merge(protected_routes)
        .with_state(Arc::clone(&app_state));

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
        
    println!("Listening on http://127.0.0.1:3000");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}

#[derive(Serialize)]
struct TokenResponse {
    token: String,
}

async fn register(State(app_state): State<AppState>) -> Json<TokenResponse> {
    let token = Uuid::new_v4().to_string();
    let (redis_client, _, _) = app_state.as_ref();
    let mut conn = redis_client.get_connection().unwrap();
    let _: () = conn.set_ex(token.clone(), "valid", 3600).unwrap(); // store for 1 hour
    Json(TokenResponse { token })
}

async fn proxy_request(
    State(app_state): State<AppState>,
    method: Method,
    uri: Uri,
    Query(query_params): Query<HashMap<String, String>>,
    headers: HeaderMap,
    body: Body,
) -> Result<axum::response::Response, StatusCode> {
    let (_, http_client, config) = app_state.as_ref();
    
    // Build the target URL
    let path = uri.path();
    let query = if query_params.is_empty() {
        String::new()
    } else {
        format!("?{}", serde_urlencoded::to_string(&query_params).unwrap())
    };
    let target_url = format!("{}{}{}", config.target_base_url, path, query);
    
    // Convert axum body to bytes
    let body_bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    // Build the proxied request
    let mut request_builder = http_client.request(method, &target_url);
    
    // Forward most headers (exclude host, authorization from original request)
    for (name, value) in &headers {
        if name != "host" && name != "authorization" {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name, value_str);
            }
        }
    }
    
    // Add API key if configured
    if let Some(api_key) = &config.api_key {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
    }
    
    // Add body if present
    if !body_bytes.is_empty() {
        request_builder = request_builder.body(body_bytes);
    }
    
    // Make the request
    let response = match request_builder.send().await {
        Ok(resp) => resp,
        Err(_) => return Err(StatusCode::BAD_GATEWAY),
    };
    
    // Build the response
    let status = response.status();
    let response_headers = response.headers().clone();
    let response_body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(_) => return Err(StatusCode::BAD_GATEWAY),
    };
    
    // Create axum response
    let mut axum_response = axum::response::Response::builder()
        .status(status);
    
    // Copy response headers
    for (name, value) in response_headers {
        if let Some(name) = name {
            axum_response = axum_response.header(name, value);
        }
    }
    
    Ok(axum_response.body(Body::from(response_body)).unwrap())
}