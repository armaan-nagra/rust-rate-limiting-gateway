use axum::{
    routing::{get, post},
    extract::State,
    Router, Json
};
use serde::Serialize;
use uuid::Uuid;


#[tokio::main]
async fn main() {
    // Logger
    tracing_subscriber::fmt::init();

    let redit_client = Client::open("redis://127.0.0.1").unwrap();

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

async fn register(State(client): State<RedisState>) -> Json<TokenResponse> {
    let token = Uuid::new_v4().to_string();
    let mut conn = client.get_async_connection().await.unwrap();
    let _: () = conn.set_ex(token.clone(), "valid", 3600).await.unwrap(); // store for 1 hour
    Json(TokenResponse { token })
}

async fn protected_data() -> &'static str {
    "Protected data here"
}