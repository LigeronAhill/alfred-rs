use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Duplicate entry: {0}")]
    DuplicateEntry(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
}

pub type DBResult<T> = std::result::Result<T, DatabaseError>;
