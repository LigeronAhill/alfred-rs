//! Репозиторий пользователей для PostgreSQL
//!
//! Этот модуль содержит реализацию репозитория пользователей
//! для работы с базой данных PostgreSQL.

use std::str::FromStr;

use async_trait::async_trait;
use tracing::instrument;

use crate::{
    AppError, AppResult,
    crypto::{hash_password, verify_password},
    models::{SignupData, User, UserInfo, UserRole},
    storage::{PgStorage, UsersRepository},
};

#[async_trait]
impl UsersRepository for PgStorage {
    /// Создает нового пользователя в базе данных
    ///
    /// # Аргументы
    ///
    /// * `signup_data` - Данные для регистрации пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<User>` - Созданный пользователь или ошибку
    #[instrument(name = "create user", skip_all, fields(email = %signup_data.email))]
    async fn create(&self, signup_data: SignupData) -> AppResult<User> {
        let mut tx = self.pool.begin().await?;
        let created_user = UserDTO::create(&mut tx, signup_data).await?;
        let created_info = UserInfoDTO::create(&mut tx, created_user.user_id).await?;
        let result = User::from((created_user, created_info.into()));
        tx.commit().await?;
        Ok(result)
    }

    /// Получает пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<User>` - Найденный пользователь или ошибку если пользователь не найден
    #[instrument(name = "get user by id", skip(self))]
    async fn get(&self, id: uuid::Uuid) -> AppResult<User> {
        let user = UserDTO::get_by_id(&self.pool, id).await?;
        let info = UserInfoDTO::get_by_user_id(&self.pool, user.user_id).await?;
        let res = User::from((user, info.into()));
        Ok(res)
    }

    /// Получает список пользователей с пагинацией
    ///
    /// # Аргументы
    ///
    /// * `page` - Номер страницы (начиная с 1)
    /// * `per_page` - Количество пользователей на странице
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Vec<User>>` - Список пользователей или ошибку
    #[instrument(name = "list users", skip(self))]
    async fn list(&self, page: u32, per_page: u32) -> AppResult<Vec<User>> {
        let offset = (page.saturating_sub(1) * per_page) as i64;
        let users_with_info = sqlx::query!(
            r#"
			SELECT
				u.user_id,
				u.email,
				u.password_hash,
				u.role,
				u.created,
				u.updated,
				ui.info_id,
				ui.first_name,
				ui.middle_name,
				ui.last_name,
				ui.username,
				ui.avatar_url,
				ui.bio,
				ui.created as info_created,
				ui.updated as info_updated
			FROM users u
			LEFT JOIN user_infos ui ON u.user_id = ui.user_id
			ORDER BY u.created DESC
			LIMIT $1 OFFSET $2
			"#,
            per_page as i64,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();

        for row in users_with_info {
            let user_dto = UserDTO {
                user_id: row.user_id,
                email: row.email,
                password_hash: row.password_hash,
                role: row.role,
                created: row.created,
                updated: row.updated,
            };

            let info_dto = UserInfoDTO {
                info_id: row.info_id,
                user_id: row.user_id,
                first_name: row.first_name,
                middle_name: row.middle_name,
                last_name: row.last_name,
                username: row.username,
                avatar_url: row.avatar_url,
                bio: row.bio,
                created: row.info_created,
                updated: row.info_updated,
            };

            result.push(User::from((user_dto, info_dto.into())));
        }

        Ok(result)
    }

    /// Получает общее количество пользователей в базе данных
    ///
    /// # Возвращает
    ///
    /// * `AppResult<u32>` - Общее количество пользователей
    #[instrument(name = "count users", skip(self))]
    async fn total(&self) -> AppResult<u32> {
        let res = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(res.unwrap_or(0) as u32)
    }

    /// Находит пользователя по email адресу
    ///
    /// # Аргументы
    ///
    /// * `email` - Email адрес пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<User>` - Найденный пользователь или ошибку если пользователь не найден
    #[instrument(name = "find user by email", skip(self))]
    async fn find_by_email(&self, email: &str) -> AppResult<User> {
        let user = UserDTO::get_by_email(&self.pool, email).await?;
        let info = UserInfoDTO::get_by_user_id(&self.pool, user.user_id).await?;
        let result = User::from((user, info.into()));
        Ok(result)
    }

    /// Обновляет данные пользователя
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя для обновления
    /// * `user` - Новые данные пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<User>` - Обновленного пользователя или ошибку
    #[instrument(name = "update user", skip(self, user))]
    async fn update(&self, id: uuid::Uuid, user: User) -> AppResult<User> {
        let mut tx = self.pool.begin().await?;
        let updated_info = UserInfoDTO::update(&mut tx, id, &user.info).await?;
        let updated_user = UserDTO::update(
            &mut tx,
            id,
            &user.email,
            &user.password_hash,
            user.role.as_ref(),
        )
        .await?;
        tx.commit().await?;
        let res = User::from((updated_user, updated_info.into()));
        Ok(res)
    }

    /// Удаляет пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя для удаления
    ///
    /// # Возвращает
    ///
    /// * `AppResult<User>` - Удаленного пользователя или ошибку
    #[instrument(name = "delete user by id", skip(self))]
    async fn delete(&self, id: uuid::Uuid) -> AppResult<User> {
        let mut tx = self.pool.begin().await?;
        let info = UserInfoDTO::delete(&mut tx, id).await?;
        let user = UserDTO::delete(&mut tx, id).await?;
        tx.commit().await?;
        let res = User::from((user, info.into()));
        Ok(res)
    }

    /// Проверяет пароль пользователя
    ///
    /// # Аргументы
    ///
    /// * `signup_data` - Данные для проверки (email и пароль)
    ///
    /// # Возвращает
    ///
    /// * `AppResult<bool>` - Результат проверки пароля (true если пароль верный)
    #[instrument(name = "verify user's password", skip_all, fields(email = %signup_data.email))]
    async fn verify_user(&self, signup_data: SignupData) -> AppResult<bool> {
        let user = UserDTO::get_by_email(&self.pool, &signup_data.email).await?;
        let res = verify_password(&user.password_hash, &signup_data.password)?;
        Ok(res)
    }
}

/// DTO (Data Transfer Object) для пользователя
///
/// Структура для представления данных пользователя из базы данных
struct UserDTO {
    user_id: uuid::Uuid,
    email: String,
    password_hash: String,
    role: String,
    created: chrono::NaiveDateTime,
    updated: chrono::NaiveDateTime,
}

impl UserDTO {
    /// Создает нового пользователя в базе данных
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `signup_data` - Данные для регистрации
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Созданный DTO пользователя
    async fn create(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        signup_data: SignupData,
    ) -> AppResult<Self> {
        let password_hash = hash_password(&signup_data.password)?;
        let created_user = sqlx::query_as!(
            UserDTO,
            r#"
			INSERT INTO users (email, password_hash, role)
			VALUES ($1, $2, $3)
			RETURNING *;
			"#,
            signup_data.email,
            password_hash,
            signup_data.role.to_string(),
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(created_user)
    }

    /// Получает пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `pool` - Пул соединений с базой данных
    /// * `id` - UUID пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Найденный DTO пользователя или ошибку
    async fn get_by_id(pool: &sqlx::PgPool, id: uuid::Uuid) -> AppResult<Self> {
        let res = sqlx::query_as!(
            UserDTO,
            r#"
			SELECT * FROM users WHERE user_id = $1;
			"#,
            id,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(res)
    }

    /// Получает пользователя по email адресу
    ///
    /// # Аргументы
    ///
    /// * `pool` - Пул соединений с базой данных
    /// * `email` - Email адрес пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Найденный DTO пользователя или ошибку
    async fn get_by_email(pool: &sqlx::PgPool, email: &str) -> AppResult<Self> {
        let res = sqlx::query_as!(
            UserDTO,
            r#"
			SELECT * FROM users WHERE email = $1;
			"#,
            email,
        )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(res)
    }

    /// Удаляет пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `id` - UUID пользователя для удаления
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Удаленный DTO пользователя или ошибку
    async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: uuid::Uuid,
    ) -> AppResult<Self> {
        let res = sqlx::query_as!(
            UserDTO,
            r#"DELETE FROM users WHERE user_id = $1 RETURNING *;"#,
            id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(res)
    }

    /// Обновляет данные пользователя
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `id` - UUID пользователя для обновления
    /// * `email` - Новый email
    /// * `password_hash` - Новый хеш пароля
    /// * `role` - Новая роль пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Обновленный DTO пользователя или ошибку
    async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        id: uuid::Uuid,
        email: &str,
        password_hash: &str,
        role: &str,
    ) -> AppResult<Self> {
        let res = sqlx::query_as!(
            UserDTO,
            r#"
			UPDATE users
			SET
				email = $2,
				password_hash = $3,
				role = $4,
				updated = NOW()
			WHERE user_id = $1
			RETURNING *;
			"#,
            id,
            email,
            password_hash,
            role,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(res)
    }
}

/// DTO (Data Transfer Object) для дополнительной информации о пользователе
///
/// Структура для представления дополнительных данных пользователя из базы данных
struct UserInfoDTO {
    #[allow(unused)]
    info_id: uuid::Uuid,
    #[allow(unused)]
    user_id: uuid::Uuid,
    first_name: Option<String>,
    middle_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
    #[allow(unused)]
    created: chrono::NaiveDateTime,
    #[allow(unused)]
    updated: chrono::NaiveDateTime,
}

impl UserInfoDTO {
    /// Создает запись с дополнительной информацией о пользователе
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `user_id` - UUID пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Созданный DTO информации о пользователе
    async fn create(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: uuid::Uuid,
    ) -> AppResult<Self> {
        let info = sqlx::query_as!(
            UserInfoDTO,
            r#"
			INSERT INTO user_infos (user_id)
			VALUES ($1)
			RETURNING *;
			"#,
            user_id,
        )
        .fetch_one(&mut **tx)
        .await?;
        Ok(info)
    }

    /// Получает дополнительную информацию о пользователе по его идентификатору
    ///
    /// # Аргументы
    ///
    /// * `pool` - Пул соединений с базой данных
    /// * `user_id` - UUID пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Найденный DTO информации о пользователе
    async fn get_by_user_id(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> AppResult<Self> {
        let info = sqlx::query_as!(
            UserInfoDTO,
            r#"
			SELECT * FROM user_infos WHERE user_id = $1;
			"#,
            user_id,
        )
        .fetch_one(pool)
        .await?;
        Ok(info)
    }

    /// Удаляет дополнительную информацию о пользователе
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `user_id` - UUID пользователя
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Удаленный DTO информации о пользователе или ошибку
    async fn delete(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: uuid::Uuid,
    ) -> AppResult<Self> {
        let res = sqlx::query_as!(
            UserInfoDTO,
            r#"
			DELETE FROM user_infos WHERE user_id = $1 RETURNING *;
			"#,
            user_id,
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(res)
    }

    /// Обновляет дополнительную информацию о пользователе
    ///
    /// # Аргументы
    ///
    /// * `tx` - Транзакция базы данных
    /// * `user_id` - UUID пользователя
    /// * `info` - Новая информация о пользователе
    ///
    /// # Возвращает
    ///
    /// * `AppResult<Self>` - Обновленный DTO информации о пользователе или ошибку
    async fn update(
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user_id: uuid::Uuid,
        info: &UserInfo,
    ) -> AppResult<Self> {
        let updated_info = sqlx::query_as!(
            UserInfoDTO,
            r#"
			UPDATE user_infos
			SET
				first_name = $2,
				middle_name = $3,
				last_name = $4,
				username = $5,
				avatar_url = $6,
				bio = $7,
				updated = NOW()
			WHERE user_id = $1
			RETURNING *;
			"#,
            user_id,
            info.first_name,
            info.middle_name,
            info.last_name,
            info.username,
            info.avatar_url,
            info.bio
        )
        .fetch_optional(&mut **tx)
        .await?
        .ok_or(AppError::EntryNotFound)?;
        Ok(updated_info)
    }
}

impl From<UserInfoDTO> for UserInfo {
    fn from(value: UserInfoDTO) -> Self {
        Self {
            first_name: value.first_name,
            middle_name: value.middle_name,
            last_name: value.last_name,
            username: value.username,
            avatar_url: value.avatar_url,
            bio: value.bio,
        }
    }
}

impl From<(UserDTO, UserInfo)> for User {
    fn from((user, info): (UserDTO, UserInfo)) -> Self {
        let role = UserRole::from_str(&user.role).unwrap_or_default();
        Self {
            user_id: user.user_id,
            email: user.email,
            password_hash: user.password_hash,
            role,
            info,
            created: user.created,
            updated: user.updated,
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        AppError, AppResult,
        models::{SignupData, User, UserInfo},
        storage::{PgStorage, UsersRepository},
    };
    #[sqlx::test]
    async fn create_user_success_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);
        let signup_data = SignupData {
            email: "test@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Guest,
        };
        let created = pg_users_repo.create(signup_data.clone()).await?;
        assert_eq!(created.email, signup_data.email);
        let verify = pg_users_repo.verify_user(signup_data.clone()).await?;
        assert!(verify);
        Ok(())
    }
    #[sqlx::test]
    async fn create_user_failed_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);
        let signup_data = SignupData {
            email: "test@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Guest,
        };
        let _created = pg_users_repo.create(signup_data.clone()).await?;
        let failed = pg_users_repo.create(signup_data).await;
        assert!(failed.is_err());
        Ok(())
    }
    #[sqlx::test]
    async fn get_user_success_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);
        let signup_data = SignupData {
            email: "test@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Guest,
        };
        let created = pg_users_repo.create(signup_data.clone()).await?;
        let verify_created = pg_users_repo
            .verify_user(SignupData {
                email: created.email,
                password: signup_data.password.clone(),
                role: created.role,
            })
            .await?;
        assert!(verify_created);
        let retrieved_by_id = pg_users_repo.get(created.user_id).await?;
        assert_eq!(retrieved_by_id.email, signup_data.email);
        let verify_retrieved_by_id = pg_users_repo
            .verify_user(SignupData {
                email: retrieved_by_id.email,
                password: signup_data.password.clone(),
                role: retrieved_by_id.role,
            })
            .await?;
        assert!(verify_retrieved_by_id);
        let retrieved_by_email = pg_users_repo.find_by_email(&signup_data.email).await?;
        assert_eq!(retrieved_by_email.email, signup_data.email);
        let verify_retrieved_by_email = pg_users_repo
            .verify_user(SignupData {
                email: retrieved_by_email.email,
                password: signup_data.password.clone(),
                role: retrieved_by_email.role,
            })
            .await?;
        assert!(verify_retrieved_by_email);
        Ok(())
    }
    #[sqlx::test]
    async fn get_user_not_found_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);
        let non_existent_id = uuid::Uuid::new_v4();

        // Тест get с несуществующим ID
        let result = pg_users_repo.get(non_existent_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        // Тест find_by_email с несуществующим email
        let result = pg_users_repo.find_by_email("nonexistent@example.com").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn update_user_success_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Создаем пользователя
        let signup_data = SignupData {
            email: "test@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Guest,
        };
        let created = pg_users_repo.create(signup_data.clone()).await?;

        // Подготавливаем обновленные данные
        let mut user_to_update = created.clone();
        user_to_update.email = "updated@example.com".to_string();
        user_to_update.role = crate::models::UserRole::Admin;
        user_to_update.info = UserInfo {
            first_name: Some("John".to_string()),
            last_name: Some("Doe".to_string()),
            username: Some("johndoe".to_string()),
            ..Default::default()
        };

        // Обновляем пользователя
        let updated = pg_users_repo
            .update(created.user_id, user_to_update)
            .await?;

        // Проверяем обновленные данные
        assert_eq!(updated.email, "updated@example.com");
        assert_eq!(updated.role, crate::models::UserRole::Admin);
        assert_eq!(updated.info.first_name, Some("John".to_string()));
        assert_eq!(updated.info.last_name, Some("Doe".to_string()));
        assert_eq!(updated.info.username, Some("johndoe".to_string()));

        // Проверяем, что updated timestamp изменился
        assert!(updated.updated > updated.created);

        // Проверяем, что данные сохранились в БД
        let retrieved = pg_users_repo.get(created.user_id).await?;
        assert_eq!(retrieved.email, "updated@example.com");
        assert_eq!(retrieved.info.username, Some("johndoe".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn update_user_not_found_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let non_existent_id = uuid::Uuid::new_v4();
        let user = User {
            user_id: non_existent_id,
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role: crate::models::UserRole::Guest,
            info: UserInfo::default(),
            created: chrono::Utc::now().naive_utc(),
            updated: chrono::Utc::now().naive_utc(),
        };

        let result = pg_users_repo.update(non_existent_id, user).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_success_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Создаем пользователя с дополнительной информацией
        let signup_data = SignupData {
            email: "test@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Guest,
        };
        let created = pg_users_repo.create(signup_data.clone()).await?;

        // Добавляем информацию о пользователе
        let mut user_with_info = created.clone();
        user_with_info.info = UserInfo {
            username: Some("testuser".to_string()),
            ..Default::default()
        };
        pg_users_repo
            .update(created.user_id, user_with_info)
            .await?;

        // Удаляем пользователя
        let deleted = pg_users_repo.delete(created.user_id).await?;
        assert_eq!(deleted.user_id, created.user_id);
        assert_eq!(deleted.info.username, Some("testuser".to_string()));

        // Проверяем, что пользователь удален
        let result = pg_users_repo.get(created.user_id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        // Проверяем, что информация также удалена (каскадно)
        let mut conn = pg_users_repo.pool.acquire().await?;
        let info_exists: Option<(bool,)> =
            sqlx::query_as("SELECT EXISTS(SELECT 1 FROM user_infos WHERE user_id = $1)")
                .bind(created.user_id)
                .fetch_optional(&mut *conn)
                .await?;

        assert!(info_exists.is_none() || !info_exists.unwrap().0);

        Ok(())
    }

    #[sqlx::test]
    async fn delete_user_not_found_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let non_existent_id = uuid::Uuid::new_v4();
        let result = pg_users_repo.delete(non_existent_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn list_users_pagination_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Создаем несколько пользователей
        for i in 0..15 {
            let signup_data = SignupData {
                email: format!("user{}@example.com", i),
                password: "str0nGp@ssw0rD".to_string(),
                role: if i % 2 == 0 {
                    crate::models::UserRole::Guest
                } else {
                    crate::models::UserRole::Employee
                },
            };
            pg_users_repo.create(signup_data).await?;
        }

        // Тестируем пагинацию
        let page1 = pg_users_repo.list(1, 10).await?;
        assert_eq!(page1.len(), 10);

        let page2 = pg_users_repo.list(2, 10).await?;
        assert_eq!(page2.len(), 5);

        let page3 = pg_users_repo.list(3, 10).await?;
        assert_eq!(page3.len(), 0);

        // Проверяем сортировку по created DESC
        let page1_timestamps: Vec<_> = page1.iter().map(|u| u.created).collect();
        assert!(page1_timestamps.windows(2).all(|w| w[0] >= w[1]));

        Ok(())
    }

    #[sqlx::test]
    async fn list_users_empty_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let users = pg_users_repo.list(1, 10).await?;
        assert!(users.is_empty());

        Ok(())
    }

    #[sqlx::test]
    async fn total_users_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Начальное количество
        let initial_total = pg_users_repo.total().await?;
        assert_eq!(initial_total, 0);

        // Создаем пользователей
        for i in 0..5 {
            let signup_data = SignupData {
                email: format!("user{}@example.com", i),
                password: "str0nGp@ssw0rD".to_string(),
                role: crate::models::UserRole::Guest,
            };
            pg_users_repo.create(signup_data).await?;
        }

        // Проверяем общее количество
        let final_total = pg_users_repo.total().await?;
        assert_eq!(final_total, 5);

        Ok(())
    }

    #[sqlx::test]
    async fn verify_user_success_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let password = "str0nGp@ssw0rD";
        let signup_data = SignupData {
            email: "verify@example.com".to_string(),
            password: password.to_string(),
            role: crate::models::UserRole::Guest,
        };

        pg_users_repo.create(signup_data.clone()).await?;

        // Правильный пароль
        let is_valid = pg_users_repo.verify_user(signup_data).await?;
        assert!(is_valid);

        Ok(())
    }

    #[sqlx::test]
    async fn verify_user_wrong_password_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let signup_data = SignupData {
            email: "verify@example.com".to_string(),
            password: "CorrectPass123!".to_string(),
            role: crate::models::UserRole::Guest,
        };

        pg_users_repo.create(signup_data.clone()).await?;

        // Неправильный пароль
        let wrong_signup_data = SignupData {
            email: "verify@example.com".to_string(),
            password: "WrongPass123!".to_string(),
            role: crate::models::UserRole::Guest,
        };

        let is_valid = pg_users_repo.verify_user(wrong_signup_data).await?;
        assert!(!is_valid);

        Ok(())
    }

    #[sqlx::test]
    async fn verify_user_not_found_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        let signup_data = SignupData {
            email: "nonexistent@example.com".to_string(),
            password: "AnyPass123!".to_string(),
            role: crate::models::UserRole::Guest,
        };

        let result = pg_users_repo.verify_user(signup_data).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AppError::EntryNotFound));

        Ok(())
    }

    #[sqlx::test]
    async fn user_info_operations_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Создаем пользователя
        let signup_data = SignupData {
            email: "info@example.com".to_string(),
            password: "str0nGp@ssw0rD".to_string(),
            role: crate::models::UserRole::Employee,
        };
        let created = pg_users_repo.create(signup_data).await?;

        // Проверяем, что информация создалась пустая
        assert!(created.info.first_name.is_none());
        assert!(created.info.last_name.is_none());
        assert!(created.info.username.is_none());

        // Обновляем информацию
        let mut updated_user = created.clone();
        updated_user.info = UserInfo {
            first_name: Some("Alice".to_string()),
            last_name: Some("Smith".to_string()),
            username: Some("alice_smith".to_string()),
            bio: Some("Software Developer".to_string()),
            avatar_url: Some("https://example.com/avatar.jpg".to_string()),
            ..Default::default()
        };

        let result = pg_users_repo.update(created.user_id, updated_user).await?;

        // Проверяем обновленную информацию
        assert_eq!(result.info.first_name, Some("Alice".to_string()));
        assert_eq!(result.info.last_name, Some("Smith".to_string()));
        assert_eq!(result.info.username, Some("alice_smith".to_string()));
        assert_eq!(result.info.bio, Some("Software Developer".to_string()));
        assert_eq!(
            result.info.avatar_url,
            Some("https://example.com/avatar.jpg".to_string())
        );

        // Проверяем методы UserInfo
        let full_name = result.info.full_name();
        assert_eq!(full_name, Some("Smith Alice".to_string()));

        let has_profile_data = result.info.has_profile_data();
        assert!(has_profile_data);

        Ok(())
    }

    #[sqlx::test]
    async fn user_role_conversion_test(pool: PgPool) -> AppResult<()> {
        let pg_users_repo = PgStorage::with_pool(pool);

        // Тестируем создание с разными ролями
        let roles = [
            crate::models::UserRole::Owner,
            crate::models::UserRole::Admin,
            crate::models::UserRole::Employee,
            crate::models::UserRole::Guest,
        ];

        for (i, role) in roles.iter().enumerate() {
            let signup_data = SignupData {
                email: format!("role{}@example.com", i),
                password: "str0nGp@ssw0rD".to_string(),
                role: role.clone(),
            };

            let user = pg_users_repo.create(signup_data).await?;
            assert_eq!(user.role, *role);

            // Проверяем is_admin для ролей
            if user.role.is_admin() {
                assert!(matches!(
                    user.role,
                    crate::models::UserRole::Owner | crate::models::UserRole::Admin
                ));
            }
        }

        Ok(())
    }
}
