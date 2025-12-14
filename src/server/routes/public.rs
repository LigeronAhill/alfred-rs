use std::sync::Arc;

use crate::{
    AppResult, AppState,
    server::{ErrorResponse, TOKEN, TokenClaims},
    settings::JWTSettings,
};
use axum::{
    Json, Router,
    extract::State,
    http::{Response, header},
    response::IntoResponse,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::Deserialize;
use serde_json::json;

pub(super) fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check_handler))
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .with_state(state)
}

async fn health_check_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "My health is fine, thank you!"
    }))
}

#[derive(Deserialize, Debug)]
struct SigninForm {
    email: String,
    password: String,
}

async fn signin_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SigninForm>,
) -> AppResult<impl IntoResponse> {
    let existing = state
        .users_service
        .signin(&payload.email, &payload.password)
        .await?;

    let token = create_token(existing.user_id, &state.jwt_settings);

    let mut response =
        Response::new(json!({"status": "success", "token": token, "user": existing}).to_string());
    response.headers_mut().insert(
        header::SET_COOKIE,
        create_cookie(&token, &state.jwt_settings).parse().unwrap(),
    );
    Ok(response)
}
#[derive(Deserialize, Debug)]
struct SignupForm {
    email: String,
    password: String,
    confirm_password: String,
}
async fn signup_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SignupForm>,
) -> AppResult<impl IntoResponse> {
    if payload.confirm_password != payload.password {
        return Ok((
            axum::http::StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                status: "error",
                message: "passwords doesn't match".into(),
            }),
        )
            .into_response());
    }
    let new_user = state
        .users_service
        .signup(&payload.email, &payload.password, None)
        .await?;
    let token = create_token(new_user.user_id, &state.jwt_settings);
    let mut response =
        Response::new(json!({"status": "success", "token": token, "user": new_user}).to_string());
    response.headers_mut().insert(
        header::SET_COOKIE,
        create_cookie(&token, &state.jwt_settings).parse().unwrap(),
    );
    Ok(response.into_response())
}

fn create_token(user_id: uuid::Uuid, jwt: &JWTSettings) -> String {
    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(jwt.expires_in)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user_id.to_string(),
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt.secret.as_ref()),
    )
    .unwrap()
}

fn create_cookie(token: &String, jwt: &JWTSettings) -> String {
    Cookie::build((TOKEN, token.to_owned()))
        .path("/")
        .max_age(time::Duration::hours(jwt.maxage))
        .same_site(SameSite::Lax)
        .http_only(true)
        .to_string()
}
