use axum::{
    routing::{get, post},
    extract::State,
    response::IntoResponse, http::Request, middleware::Next,
    Router, Json, body::Body
};
use serde::Serialize;
use uuid::Uuid;
use redis::{Client, Commands};
use axum::http::StatusCode;
use std::sync::Arc;
use axum::middleware;
type RedisState = Arc<Client>;

async fn rate_limiter(
    State(client): State<RedisState>,
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

    let mut conn = client.get_connection().unwrap();
    let key = format!("ratelimit:{}", token);
    let count: i32 = conn.incr(&key, 1).unwrap();
    let _: bool = conn.expire(&key, 60).unwrap();

    if count > 60 {
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded, rate is 60 requests per minute").into_response();
    }

    next.run(req).await
}

#[tokio::main]
async fn main() {
    // Logger
    tracing_subscriber::fmt::init();

    let redis_client = Arc::new(Client::open("redis://127.0.0.1").unwrap());
    
    let protected_routes = Router::new()
        .route("/api/data", get(protected_data))
        .layer(middleware::from_fn_with_state(
            Arc::clone(&redis_client),
            rate_limiter,
        ));

    let app = Router::new()
        .route("/register", post(register))
        .merge(protected_routes)
        .with_state(Arc::clone(&redis_client));

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

async fn register(State(client): State<RedisState>) -> Json<TokenResponse> {
    let token = Uuid::new_v4().to_string();
    let mut conn = client.get_connection().unwrap();
    let _: () = conn.set_ex(token.clone(), "valid", 3600).unwrap(); // store for 1 hour
    Json(TokenResponse { token })
}

async fn protected_data() -> &'static str {
    "Protected data here"
}