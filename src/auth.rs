//! Authentication and token management

use axum::{extract::State, Json};
use redis::Commands;
use serde::Serialize;
use uuid::Uuid;

use crate::middleware::AppState;

#[derive(Serialize)]
pub struct TokenResponse {
    pub token: String,
}

/// Generate a new authentication token for rate limiting
pub async fn register(State(app_state): State<AppState>) -> Json<TokenResponse> {
    let token = generate_token();
    
    let (redis_client, _, _) = app_state.as_ref();
    
    // Store token in Redis with 1 hour expiration
    if let Ok(mut conn) = redis_client.get_connection() {
        let _: Result<(), redis::RedisError> = conn.set_ex(&token, "valid", 3600);
    }
    
    tracing::info!("Generated new token: {}", &token[..8]);
    
    Json(TokenResponse { token })
}

/// Generate a cryptographically secure token
fn generate_token() -> String {
    Uuid::new_v4().to_string()
} 