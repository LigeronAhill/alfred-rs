mod public;
mod users;
use std::sync::Arc;

use axum::Router;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info_span};

use crate::AppState;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub(super) fn init(state: Arc<AppState>, origin: &Option<String>) -> Router {
    // Middleware layers configuration:
    // 1. CatchPanicLayer: Prevents server crashes by catching panics
    let catch_panic_layer = CatchPanicLayer::new();

    // 2. Request ID middleware stack:
    //    - Sets unique request IDs using UUIDs
    //    - Creates tracing spans with request IDs for observability
    //    - Propagates request IDs through the call chain
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

    // 3. Timeout middleware: Returns 408 status after 10 seconds
    let timeout_layer = TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        std::time::Duration::from_secs(10),
    );

    // 4. CORS configuration: Configures cross-origin requests
    //    If origin is provided and valid, uses specific origin, otherwise allows any origin
    let cors_layer = origin
        .as_ref()
        .and_then(|o| o.parse::<axum::http::HeaderValue>().ok())
        .map(|hv| {
            CorsLayer::new().allow_origin(hv).allow_methods(Any)
            // .allow_credentials(true)
        })
        .unwrap_or(
            CorsLayer::new().allow_origin(Any).allow_methods(Any), // .allow_credentials(true),
        );

    let compression_layer = CompressionLayer::new();

    // Router configuration:
    // - Public API routes under /api/v1
    // - User-specific routes under /api/v1/users
    // - All middleware layers applied in order
    // - Fallback handler for unmatched routes
    let app = Router::new()
        .merge(public::routes(state.clone()))
        .merge(users::routes(state.clone()));
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
