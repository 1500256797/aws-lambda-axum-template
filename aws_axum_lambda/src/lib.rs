use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use state::AppState;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
pub mod features;
pub mod state;
pub mod structs;

async fn hello() -> Json<Value> {
    info!("处理 hello 请求");
    Json(json!({
        "message": "Hello from AWS Lambda + Axum!!!!! 2024-12-11"
    }))
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

pub async fn setup_server() -> Router {
    // 设置 CORS
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);

    let app_state = AppState::new().await;

    let router = Router::new()
        .route("/", get(hello))
        .route("/hello", get(hello))
        .route("/health", get(health_check))
        .merge(features::todo::route::router())
        .with_state(app_state)
        .layer(cors) // 添加 CORS 支持
        .layer(TraceLayer::new_for_http());
    router
}
