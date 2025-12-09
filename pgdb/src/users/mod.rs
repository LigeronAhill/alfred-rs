mod create;

pub struct UsersStorage {
    #[allow(unused)]
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl UsersStorage {
    pub async fn init(db_url: &str) -> sqlx::Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(16)
            .connect(db_url)
            .await?;
        if let Err(e) = sqlx::migrate!().run(&pool).await {
            tracing::error!("Migration failed: {e:?}");
        }
        let storage = Self { pool };
        Ok(storage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init_failed() {
        let db_url = "postgres://user:password@localhost:5432/dbname";
        let result = UsersStorage::init(db_url).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_init_success() {
        tracing_subscriber::fmt::init();
        let db_url = std::env::var("DATABASE_URL").unwrap();
        let result = UsersStorage::init(&db_url).await.inspect_err(|e| {
            dbg!(e);
        });
        assert!(result.is_ok());
    }
}
