mod public;
mod users;
use std::sync::Arc;

use axum::{Router, middleware};
use http::{Method, header};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info_span};

use crate::AppState;

const REQUEST_ID_HEADER: &str = "alfred-request-id";

pub(super) fn init(state: Arc<AppState>, origin: &str) -> Router {
    let catch_panic_layer = CatchPanicLayer::new();

    let x_request_id = axum::http::HeaderName::from_static(REQUEST_ID_HEADER);

    let request_id_middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(
            x_request_id.clone(),
            MakeRequestUuid,
        ))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                let request_id = request.headers().get(REQUEST_ID_HEADER);

                match request_id {
                    Some(request_id) => info_span!(
                        "http_request",
                        request_id = ?request_id,
                    ),
                    None => {
                        error!("could not extract request_id");
                        info_span!("http_request")
                    }
                }
            }),
        )
        .layer(PropagateRequestIdLayer::new(x_request_id));

    let timeout_layer = TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        std::time::Duration::from_secs(10),
    );

    let cors_layer = CorsLayer::new()
        .allow_origin([origin.parse().unwrap()])
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::ACCEPT, header::AUTHORIZATION])
        .max_age(std::time::Duration::from_secs(60 * 60))
        .allow_credentials(true);

    let compression_layer = CompressionLayer::new();

    let users_routes = Router::new().nest("/users", users::routes(state.clone()));

    let protected_routes = Router::new()
        .merge(users_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            super::middleware::auth,
        ));

    let app = Router::new()
        .merge(public::routes(state.clone()))
        .merge(protected_routes);
    Router::new()
        .nest("/api/v1", app)
        .layer(catch_panic_layer)
        .layer(request_id_middleware)
        .layer(timeout_layer)
        .layer(cors_layer)
        .layer(compression_layer)
        .fallback(fallback_handler)
}

// Fallback handler: Returns 404 status with informative message
async fn fallback_handler(uri: axum::http::Uri) -> (axum::http::StatusCode, String) {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route for {uri}"),
    )
}
