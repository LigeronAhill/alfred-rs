use std::sync::Arc;

use axum::{Router, routing::*};

use crate::AppState;

pub(super) fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/{id}",
            get(|| async {}).put(|| async {}).delete(|| async {}),
        )
        .route("/", get(|| async {}))
        .with_state(state)
}
