use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

use crate::storage::UsersFilterBuilderError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Custom error message: {0}")]
    Custom(String),
    #[error("Database internal error: {0}")]
    DatabaseInternalError(#[from] sqlx::Error),
    #[error("Database migration error: {0}")]
    DatabaseMigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("Entry not found")]
    EntryNotFound,
    #[error("Entry already exists")]
    EntryAlreadyExists,
    #[error("Invalid input")]
    InvalidInput,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid user role")]
    InvalidUserRole(String),
    #[error("Validation error")]
    ValidationError(#[from] ValidationError),
    #[error("Validation errors")]
    ValidationErrors(#[from] ValidationErrors),
    #[error("Error while hashing {0}")]
    CryptoError(String),
    #[error("Error parsing id {0}")]
    UuidError(#[from] uuid::Error),
    #[error("Error building struct {0}")]
    BuilderError(#[from] UsersFilterBuilderError),
    // #[error("Tokio IO error {0}")]
    // TokioIOError(#[from] tokio::io::Error),
    #[error("IO error {0}")]
    IOError(#[from] std::io::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::EntryNotFound => StatusCode::NOT_FOUND,
            AppError::EntryAlreadyExists
            | AppError::InvalidInput
            | AppError::InvalidCredentials
            | AppError::InvalidUserRole(_)
            | AppError::ValidationError(_)
            | AppError::ValidationErrors(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}
