mod auth;
mod config;
mod middleware;
mod proxy;

use axum::{
    middleware as axum_middleware,
    routing::{any, post},
    Router,
};
use redis::Client as RedisClient;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

use crate::{
    auth::register,
    config::ProxyConfig,
    middleware::{rate_limiter, AppState},
    proxy::proxy_request,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = ProxyConfig::from_env()?;
    config.display_summary();

    let redis_client = Arc::new(RedisClient::open("redis://127.0.0.1")?);
    let http_client = reqwest::Client::new();

    let app_state = Arc::new((redis_client, http_client, config));

    let app = build_router(app_state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(app_state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/register", post(register));

    let protected_routes = Router::new()
        .route("/api/*path", any(proxy_request))
        .layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            rate_limiter,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(app_state)
}