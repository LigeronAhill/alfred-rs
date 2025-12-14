use std::sync::Arc;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::{Response, header},
    response::IntoResponse,
    routing::*,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::Deserialize;
use serde_json::json;

use crate::{
    AppError, AppResult, AppState,
    models::{User, UserToUpdate},
    server::TOKEN,
    services::UsersListResponse,
};

pub(super) fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/{id}",
            get(get_by_id_handler)
                .put(update_handler)
                .delete(delete_handler),
        )
        .route("/me", get(getme_handler))
        .route("/", get(list_handler))
        .route("/logout", get(logout_handler))
        .with_state(state)
}

async fn logout_handler() -> impl IntoResponse {
    let cookie = Cookie::build((TOKEN, ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    response
}

async fn getme_handler(Extension(user): Extension<User>) -> AppResult<Json<User>> {
    Ok(Json(user))
}
async fn get_by_id_handler(
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<User>> {
    match uuid::Uuid::parse_str(&id) {
        Ok(parsed_id) => {
            if !user.role.is_admin() && user.user_id != parsed_id {
                return Err(AppError::AccessDenied);
            }
            let founded = state.users_service.get_by_id(&id).await?;
            Ok(Json(founded))
        }
        Err(_) => Err(AppError::InvalidInput),
    }
}
async fn delete_handler(
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> AppResult<Json<User>> {
    match uuid::Uuid::parse_str(&id) {
        Ok(_) => {
            if !user.role.is_admin() {
                return Err(AppError::AccessDenied);
            }
            let deleted = state.users_service.delete(&id).await?;
            Ok(Json(deleted))
        }
        Err(_) => Err(AppError::InvalidInput),
    }
}

async fn list_handler(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<Filter>,
) -> AppResult<Json<UsersListResponse>> {
    if !user.role.is_admin() {
        return Err(AppError::AccessDenied);
    }
    let result = state
        .users_service
        .list(filter.page, filter.per_page, filter.role, filter.q)
        .await?;
    Ok(Json(result))
}

#[derive(Clone, Debug, Deserialize)]
struct Filter {
    page: Option<String>,
    per_page: Option<String>,
    role: Option<String>,
    q: Option<String>,
}

#[axum::debug_handler]
async fn update_handler(
    Extension(user): Extension<User>,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UserToUpdate>,
) -> AppResult<Json<User>> {
    match uuid::Uuid::parse_str(&id) {
        Ok(parsed_id) => {
            if !user.role.is_admin() && user.user_id != parsed_id {
                return Err(AppError::AccessDenied);
            }
            let updated = state.users_service.update(&id, payload).await?;
            Ok(Json(updated))
        }
        Err(_) => Err(AppError::InvalidInput),
    }
}
