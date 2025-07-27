use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    body::Body,
};
use redis::{Client as RedisClient, Commands};
use std::sync::Arc;

use crate::config::ProxyConfig;

pub type AppState = Arc<(Arc<RedisClient>, reqwest::Client, ProxyConfig)>;


pub async fn rate_limiter(
    State(app_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> axum::response::Response {
    let token = extract_bearer_token(&req);
    
    if token.is_empty() {
        return (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header").into_response();
    }

    let (redis_client, _, config) = app_state.as_ref();
    
    match check_rate_limit(redis_client, &token, config.rate_limit).await {
        Ok(allowed) => {
            if allowed {
                next.run(req).await
            } else {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    format!("Rate limit exceeded: {} requests per minute", config.rate_limit)
                ).into_response()
            }
        }
        Err(err) => {
            tracing::error!("Rate limiting error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Rate limiting service unavailable").into_response()
        }
    }
}

fn extract_bearer_token(req: &Request<Body>) -> String {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .strip_prefix("Bearer ")
        .unwrap_or("")
        .to_string()
}

async fn check_rate_limit(
    redis_client: &RedisClient,
    token: &str,
    rate_limit: u32,
) -> Result<bool, RateLimitError> {
    let mut conn = redis_client
        .get_connection()
        .map_err(RateLimitError::RedisConnection)?;
    
    let key = format!("ratelimit:{}", token);
    // Atomic increment and expire
    let count: i32 = conn
        .incr(&key, 1)
        .map_err(RateLimitError::RedisOperation)?;
    
    let _: bool = conn
        .expire(&key, 60)
        .map_err(RateLimitError::RedisOperation)?;
    
    Ok(count <= rate_limit as i32)
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis connection failed: {0}")]
    RedisConnection(#[from] redis::RedisError),
    #[error("Redis operation failed: {0}")]
    RedisOperation(redis::RedisError),
} 