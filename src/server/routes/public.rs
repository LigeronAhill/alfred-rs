use std::sync::Arc;

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::json;

use crate::AppState;

pub(super) fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/signin", post(|| async {}))
        .route("/signup", post(|| async {}))
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "My health is fine, thank you!"
    }))
}
