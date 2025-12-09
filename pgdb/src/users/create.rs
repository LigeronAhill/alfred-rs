use super::UsersStorage;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::NaiveDateTime;
use shared::models::{CreateUserDTO, User, UserInfo, UserRole};

impl UsersStorage {
    pub async fn create_user(&self, dto: CreateUserDTO) -> sqlx::Result<User> {
        let salt = SaltString::generate(&mut OsRng);
        let a2 = Argon2::default();
        let Ok(password_hash) = a2
            .hash_password(dto.password.as_bytes(), &salt)
            .map(|p| p.to_string())
        else {
            return Err(sqlx::Error::InvalidArgument(
                "Failed to hash password".into(),
            ));
        };
        if let Some(existing) = sqlx::query_as!(
            UserDTO,
            r#"
        SELECT id, email, password_hash, role as "role: UserRole", user_info_id, created_at, updated_at FROM users WHERE email = $1
        "#,
            dto.email,
        )
        .fetch_optional(&self.pool)
        .await? {
        return Ok(User {
            id: todo!(),
            email: todo!(),
            password_hash,
            role: todo!(),
            user_info: todo!(),
            created_at: todo!(),
            updated_at: todo!(),
        })
        }
        let mut tx = &self.pool.begin().await?;
        let user_info = sqlx::query_as!(
            UserInfo,
            r#"
        INSERT INTO user_infos (first_name, middle_name, last_name, username, userpic_url, bio)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *;
        "#,
            dto.user_info.first_name,
            dto.user_info.middle_name,
            dto.user_info.last_name,
            dto.user_info.username,
            dto.user_info.userpic_url,
            dto.user_info.bio,
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(res)
    }
}

pub struct UserDTO {
    pub id: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub user_info_id: uuid::Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
