use axum::{
    routing::{get, post},
    Router, Json
};
use serde::Serialize;
use uuid::Uuid;


#[tokio::main]
async fn main() {
    // Logger
    tracing_subscriber::fmt::init();

    // Build routes
    let app = Router::new()
        .route("/register", post(register))
        .route("/api/data", get(protected_data));

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

async fn register() -> Json<TokenResponse> {
    let token = Uuid::new_v4().to_string();
    // For now just return it. We'll store it in Redis later.
    Json(TokenResponse { token })
}

async fn protected_data() -> &'static str {
    "Protected data here"
}