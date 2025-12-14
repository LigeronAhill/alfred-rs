use crate::{
    AppState,
    server::{ErrorResponse, TOKEN, TokenClaims},
};

use std::sync::Arc;

use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};

use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{DecodingKey, Validation, decode};

pub async fn auth(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    for (key, value) in req.headers() {
        tracing::debug!("Header '{key}': '{value:?}'");
    }
    let token = cookie_jar
        .get(TOKEN)
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value.replace("Bearer ", "").to_owned())
                    } else {
                        None
                    }
                })
        });
    let token = token.ok_or((
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            status: "fail",
            message: "No token provided".into(),
        }),
    ))?;

    let claims = decode::<TokenClaims>(
        &token,
        &DecodingKey::from_secret(state.jwt_settings.secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                status: "fail",
                message: "Invalid token".into(),
            }),
        )
    })?
    .claims;
    let user_id = claims.sub;
    let user = state.users_service.get_by_id(&user_id).await.map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                status: "fail",
                message: "Invalid token".into(),
            }),
        )
    })?;
    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
