use anyhow::{Context, Result};
use shared::models::{User, UserRole};

pub struct UsersStorage {
    pool: sqlx::Pool<sqlx::Postgres>,
}
impl UsersStorage {
    #[tracing::instrument(name = "creating users storage")]
    pub async fn new() -> Result<Self> {
        let db_url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(16)
            .connect(&db_url)
            .await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
    #[tracing::instrument(name = "get all users", skip(self))]
    pub async fn get_all_users(&self, limit: i64, offset: i64) -> Result<Vec<User>> {
        let query_result = sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY user_id LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(query_result)
    }
    #[tracing::instrument(name = "get user by id", skip(self))]
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<User>> {
        let query_result = sqlx::query_as!(User, "SELECT * FROM users WHERE user_id = $1", user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(query_result)
    }
    #[tracing::instrument(name = "register new user", skip(self))]
    pub async fn register_user(&self, user_id: i64, user_name: String) -> Result<UserRole> {
        if let Some(existing_user) = self.get_user_by_id(user_id).await? {
            Ok(existing_user.user_role)
        } else {
            let limit = 100;
            let mut offset = 0;
            let mut user_role = UserRole::Admin;
            loop {
                let existing_users = self.get_all_users(limit, offset).await?;
                if existing_users.is_empty() {
                    break;
                } else if existing_users
                    .iter()
                    .any(|u| u.user_role == UserRole::Admin)
                {
                    user_role = UserRole::Guest;
                    break;
                } else {
                    offset += limit;
                }
            }
            let query_result = sqlx::query_as!(User, "INSERT INTO users (user_id, user_name, user_role) VALUES ($1, $2, $3) ON CONFLICT(user_id) DO NOTHING RETURNING *;", user_id, user_name, user_role.to_string()).fetch_one(&self.pool).await?;
            Ok(query_result.user_role)
        }
    }
    #[tracing::instrument(name = "update user role", skip(self))]
    pub async fn update_user_role(
        &self,
        user_id: i64,
        new_user_role: UserRole,
    ) -> Result<Option<User>> {
        let query_result = sqlx::query_as!(
            User,
            "UPDATE users SET user_role = $1, updated_at = now() WHERE user_id = $2 RETURNING *;",
            new_user_role.to_string(),
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(query_result)
    }
    #[tracing::instrument(name = "delete user", skip(self))]
    pub async fn delete_user(&self, user_id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
