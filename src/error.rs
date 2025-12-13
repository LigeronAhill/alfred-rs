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
}

pub type AppResult<T> = Result<T, AppError>;
