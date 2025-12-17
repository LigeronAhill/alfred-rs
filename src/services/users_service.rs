use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    AppError, AppResult,
    models::{SigninData, User, UserRole, UserToUpdate},
    storage::{DEFAULT_PAGE_NUM, DEFAULT_PER_PAGE, UsersFilter, UsersRepository},
};

/// Сервис для работы с пользователями
///
/// Предоставляет высокоуровневые операции над пользователями,
/// такие как создание, аутентификация, поиск и управление пользователями.
/// Инкапсулирует бизнес-логику и валидацию данных.
#[derive(Clone)]
pub struct UsersService {
    pub storage: Arc<dyn UsersRepository>,
}
impl UsersService {
    /// Создает новый экземпляр сервиса пользователей
    ///
    /// # Аргументы
    ///
    /// * `storage` - Реализация трейта `UsersRepository` в `Arc`
    ///
    /// # Возвращает
    ///
    /// Новый экземпляр `UsersService`
    pub fn new(storage: Arc<dyn UsersRepository>) -> Self {
        Self { storage }
    }
    /// Создает нового пользователя
    ///
    /// # Аргументы
    ///
    /// * `email` - Email пользователя
    /// * `password` - Пароль пользователя (будет хеширован перед сохранением)
    /// * `role` - Роль пользователя в строковом формате (опционально)
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Созданный пользователь
    /// * `Err(AppError)` - Ошибка валидации, парсинга роли или сохранения
    pub async fn signup(&self, email: &str, password: &str, role: Option<&str>) -> AppResult<User> {
        let role = role
            .and_then(|r| UserRole::from_str(r).ok())
            .unwrap_or_default();
        let data = (email, password, role.as_ref()).try_into()?;
        let new_user = self.storage.create(data).await.map_err(|e| {
            if e.to_string().contains("duplicate key") {
                AppError::EntryAlreadyExists
            } else {
                e
            }
        })?;
        Ok(new_user)
    }
    /// Получает пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя в строковом формате
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Найденный пользователь
    /// * `Err(AppError)` - Ошибка парсинга UUID или если пользователь не найден
    pub async fn get_by_id(&self, id: &str) -> AppResult<User> {
        let user_id = uuid::Uuid::parse_str(id)?;
        let user = self.storage.get(user_id).await?;
        Ok(user)
    }
    /// Получает информацию о пользователе по email
    ///
    /// # Аргументы
    ///
    /// * `email` - Email адрес пользователя
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Найденный пользователь
    /// * `Err(AppError)` - Ошибка валидации email или если пользователь не найден
    ///
    /// # Особенности
    ///
    /// - Email нормализуется (trim + lowercase)
    /// - Проверяется валидность формата email
    pub async fn get_user_info(&self, email: &str) -> AppResult<User> {
        let email = email.trim().to_lowercase();
        let data = Email { email };
        data.validate()?;
        let user = self.storage.find_by_email(&data.email).await?;
        Ok(user)
    }
    /// Получает список пользователей с пагинацией и фильтрацией
    ///
    /// # Аргументы
    ///
    /// * `page` - Номер страницы (опционально, строка)
    /// * `per_page` - Количество элементов на странице (опционально, строка)
    /// * `role` - Роль для фильтрации (опционально, строка)
    /// * `q` - Строка поиска (опционально)
    ///
    /// # Возвращает
    ///
    /// * `Ok(UsersListResponse)` - Ответ со списком пользователей и метаданными
    /// * `Err(AppError)` - Ошибка парсинга параметров или выполнения запроса
    ///
    /// # Особенности
    ///
    /// - Если параметры не указаны, используются значения по умолчанию
    /// - Роль парсится в `UserRole`, невалидная роль игнорируется
    /// - Поддерживается поиск по email, имени пользователя, имени и фамилии
    pub async fn list(
        &self,
        page: Option<String>,
        per_page: Option<String>,
        role: Option<String>,
        q: Option<String>,
    ) -> AppResult<UsersListResponse> {
        let user_role_filter = role.and_then(|r| r.try_into().ok());
        let filter = UsersFilter::builder()
            .page(
                page.and_then(|p| p.parse().ok())
                    .unwrap_or(DEFAULT_PAGE_NUM),
            )
            .per_page(
                per_page
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(DEFAULT_PER_PAGE),
            )
            .role(user_role_filter)
            .search_string(q)
            .build()?;
        let users = self.storage.list(filter.clone()).await?;
        let total = self.storage.total(filter.clone()).await?;
        let res = UsersListResponse {
            current_filter: filter,
            total,
            users,
        };
        Ok(res)
    }
    /// Выполняет аутентификацию пользователя
    ///
    /// # Аргументы
    ///
    /// * `email` - Email пользователя
    /// * `password` - Пароль пользователя
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Аутентифицированный пользователь
    /// * `Err(AppError::InvalidCredentials)` - Неверные учетные данные
    /// * `Err(AppError)` - Другие ошибки (валидация, поиск пользователя и т.д.)
    pub async fn signin(&self, email: &str, password: &str) -> AppResult<User> {
        let signin_data = SigninData::try_from((email, password))?;
        let is_verified = self.storage.verify_user(signin_data.clone()).await?;
        if is_verified {
            let user = self.storage.find_by_email(&signin_data.email).await?;
            Ok(user)
        } else {
            Err(crate::AppError::InvalidCredentials)
        }
    }
    /// Удаляет пользователя по идентификатору
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя в строковом формате
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Удаленный пользователь
    /// * `Err(AppError)` - Ошибка парсинга UUID или если пользователь не найден
    pub async fn delete(&self, id: &str) -> AppResult<User> {
        let user_id = uuid::Uuid::parse_str(id)?;
        let deleted_user = self.storage.delete(user_id).await?;
        Ok(deleted_user)
    }
    /// Обновляет данные пользователя
    ///
    /// # Аргументы
    ///
    /// * `id` - UUID пользователя в строковом формате
    /// * `user` - Новые данные пользователя
    ///
    /// # Возвращает
    ///
    /// * `Ok(User)` - Обновленный пользователь
    /// * `Err(AppError)` - Ошибка парсинга UUID или если пользователь не найден
    pub async fn update(&self, id: &str, user: UserToUpdate) -> AppResult<User> {
        let user_id = uuid::Uuid::parse_str(id)?;
        let updated_user = self.storage.update(user_id, user).await?;
        Ok(updated_user)
    }
}

/// Структура для валидации email
///
/// Используется для проверки формата email перед выполнением операций.
#[derive(Validate)]
struct Email {
    #[validate(email)]
    email: String,
}

/// Ответ со списком пользователей
///
/// Содержит список пользователей, метаданные пагинации и общее количество.
/// Используется для возврата результатов поиска пользователей с пагинацией.
///
/// # Сериализация
///
/// Структура реализует `Serialize` и `Deserialize` для использования в API.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UsersListResponse {
    pub current_filter: UsersFilter,
    pub total: u32,
    pub users: Vec<User>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::AppError;
    use crate::crypto::{hash_password, verify_password};
    use crate::models::{SignupData, UserInfo};
    use crate::storage::MAX_PER_PAGE;
    use async_trait::async_trait;
    use uuid::Uuid;

    /// Тестовый репозиторий для модульного тестирования
    ///
    /// Используется для изоляции тестов сервиса от реальной базы данных.
    /// Позволяет контролировать возвращаемые данные и ошибки.
    struct TestUsersRepo {
        users: Arc<Mutex<Vec<User>>>,
    }

    impl TestUsersRepo {
        fn new() -> Self {
            Self {
                users: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn with_users(users: Vec<User>) -> Self {
            Self {
                users: Arc::new(Mutex::new(users)),
            }
        }
    }

    #[async_trait]
    impl UsersRepository for TestUsersRepo {
        async fn create(&self, signup_data: SignupData) -> AppResult<User> {
            let password_hash = hash_password(&signup_data.password)?;
            let user = User {
                user_id: Uuid::new_v4(),
                email: signup_data.email.clone(),
                password_hash,
                role: signup_data.role,
                info: crate::models::UserInfo::default(),
                created: chrono::Utc::now().naive_utc(),
                updated: chrono::Utc::now().naive_utc(),
            };
            self.users.lock().unwrap().push(user.clone());
            Ok(user)
        }

        async fn get(&self, id: Uuid) -> AppResult<User> {
            self.users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.user_id == id)
                .cloned()
                .ok_or(AppError::EntryNotFound)
        }

        async fn list(&self, filter: UsersFilter) -> AppResult<Vec<User>> {
            let mut users = self.users.lock().unwrap().clone();
            let mut result = Vec::new();

            // Фильтрация по роли
            if let Some(role_str) = filter.role() {
                let role = UserRole::from_str(role_str).unwrap_or_default();
                users.iter().for_each(|u| {
                    if u.role == role {
                        result.push(u.clone());
                    }
                });
            }
            if !result.is_empty() {
                users = result.clone();
            }

            // Поиск (упрощенная имитация)
            if let Some(search) = filter.search_string() {
                users.iter().for_each(|u| {
                    if u.email.contains(search)
                        || u.info
                            .username
                            .as_ref()
                            .is_some_and(|un| un.contains(search))
                        || u.info
                            .first_name
                            .as_ref()
                            .is_some_and(|n| n.contains(search))
                        || u.info
                            .last_name
                            .as_ref()
                            .is_some_and(|ln| ln.contains(search))
                    {
                        result.push(u.clone());
                    }
                });
            }

            if !result.is_empty() {
                users = result;
            }

            // Пагинация
            let page = filter.page() as usize;
            let per_page = filter.per_page() as usize;
            let start = (page - 1) * per_page;
            let end = std::cmp::min(start + per_page, users.len());

            if start >= users.len() {
                return Ok(Vec::new());
            }

            Ok(users[start..end].to_vec())
        }

        async fn total(&self, filter: UsersFilter) -> AppResult<u32> {
            let mut users = self.users.lock().unwrap().clone();

            // Фильтрация по роли
            if let Some(role_str) = filter.role() {
                let role = UserRole::from_str(role_str).unwrap_or_default();
                users.retain(|u| u.role == role);
            }

            // Поиск
            if let Some(search) = filter.search_string() {
                users.retain(|u| {
                    u.email.contains(search)
                        || u.info
                            .username
                            .as_ref()
                            .is_some_and(|un| un.contains(search))
                        || u.info
                            .first_name
                            .as_ref()
                            .is_some_and(|n| n.contains(search))
                        || u.info
                            .last_name
                            .as_ref()
                            .is_some_and(|ln| ln.contains(search))
                });
            }

            Ok(users.len() as u32)
        }

        async fn find_by_email(&self, email: &str) -> AppResult<User> {
            self.users
                .lock()
                .unwrap()
                .iter()
                .find(|u| u.email == email)
                .cloned()
                .ok_or(AppError::EntryNotFound)
        }

        async fn update(&self, id: Uuid, user: UserToUpdate) -> AppResult<User> {
            let mut users = self.users.lock().unwrap();

            if let Some(existing_user) = users.iter_mut().find(|u| u.user_id == id) {
                existing_user.email = user.email;
                existing_user.role = user.role;
                existing_user.info = user.info;
                Ok(existing_user.clone())
            } else {
                Err(AppError::EntryNotFound)
            }
        }

        async fn delete(&self, id: Uuid) -> AppResult<User> {
            let mut users = self.users.lock().unwrap();
            let pos = users.iter().position(|u| u.user_id == id);

            if let Some(pos) = pos {
                Ok(users.remove(pos))
            } else {
                Err(AppError::EntryNotFound)
            }
        }

        async fn verify_user(&self, signin_data: SigninData) -> AppResult<bool> {
            match self.find_by_email(&signin_data.email).await {
                Ok(user) => {
                    let verified = verify_password(&user.password_hash, &signin_data.password)?;
                    Ok(verified)
                }
                Err(AppError::EntryNotFound) => Err(AppError::EntryNotFound),
                Err(e) => Err(e),
            }
        }
    }

    /// Создает тестового пользователя
    fn create_test_user(id: Uuid, email: &str, role: UserRole, username: Option<&str>) -> User {
        User {
            user_id: id,
            email: email.to_string(),
            password_hash: hash_password("test_p@sSword1123").unwrap(),
            role,
            info: UserInfo {
                username: username.map(|s| s.to_string()),
                first_name: Some("Test".to_string()),
                last_name: Some("User".to_string()),
                ..Default::default()
            },
            created: chrono::Utc::now().naive_utc(),
            updated: chrono::Utc::now().naive_utc(),
        }
    }

    /// Тест создания сервиса
    #[tokio::test]
    async fn test_users_service_creation() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        assert!(Arc::strong_count(&service.storage) > 0);
    }

    /// Тест успешного создания пользователя
    #[tokio::test]
    async fn test_create_user_success() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service
            .signup("test@example.com", "p@sSword123", Some("Admin"))
            .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.role, UserRole::Admin);
    }

    /// Тест создания пользователя с ролью по умолчанию
    #[tokio::test]
    async fn test_create_user_with_default_role() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service
            .signup("test@example.com", "p@sSword123", None)
            .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.role, UserRole::Guest); // Guest - роль по умолчанию
    }

    /// Тест создания пользователя с невалидной ролью
    #[tokio::test]
    async fn test_create_user_with_invalid_role() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service
            .signup("test@example.com", "p@sSword123", Some("InvalidRole"))
            .await;

        assert!(result.is_ok()); // Невалидная роль должна игнорироваться и использоваться роль по умолчанию
        let user = result.unwrap();
        assert_eq!(user.role, UserRole::Guest);
    }

    /// Тест создания пользователя с невалидным email (через get_user_info)
    #[tokio::test]
    async fn test_create_user_with_invalid_email() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        // Создаем пользователя с валидным email
        let result = service
            .signup("valid@example.com", "p@sSword123", None)
            .await;
        assert!(result.is_ok());

        // Пытаемся получить с невалидным email
        let result = service.get_user_info("not-an-email").await;
        assert!(result.is_err());
    }

    /// Тест получения пользователя по ID
    #[tokio::test]
    async fn test_get_by_id_success() {
        let user_id = Uuid::new_v4();
        let test_user = create_test_user(
            user_id,
            "test@example.com",
            UserRole::Employee,
            Some("testuser"),
        );

        let test_repo = TestUsersRepo::with_users(vec![test_user.clone()]);
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_by_id(&user_id.to_string()).await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.user_id, user_id);
        assert_eq!(user.email, "test@example.com");
    }

    /// Тест получения пользователя с невалидным UUID
    #[tokio::test]
    async fn test_get_by_id_invalid_uuid() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_by_id("not-a-valid-uuid").await;
        assert!(result.is_err());
    }

    /// Тест получения несуществующего пользователя
    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_by_id(&Uuid::new_v4().to_string()).await;
        assert!(result.is_err());
    }

    /// Тест получения информации о пользователе по email
    #[tokio::test]
    async fn test_get_user_info_success() {
        let test_user = create_test_user(Uuid::new_v4(), "user@example.com", UserRole::Guest, None);

        let test_repo = TestUsersRepo::with_users(vec![test_user.clone()]);
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_user_info("USER@EXAMPLE.COM").await; // Проверка нормализации
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "user@example.com");
    }

    /// Тест получения информации с невалидным email
    #[tokio::test]
    async fn test_get_user_info_invalid_email() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_user_info("not-an-email").await;
        assert!(result.is_err());
    }

    /// Тест получения информации о несуществующем пользователе
    #[tokio::test]
    async fn test_get_user_info_not_found() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.get_user_info("nonexistent@example.com").await;
        assert!(result.is_err());
    }

    /// Тест получения списка пользователей с пагинацией
    #[tokio::test]
    async fn test_list_users_with_pagination() {
        let users = (0..5)
            .map(|i| {
                create_test_user(
                    Uuid::new_v4(),
                    &format!("user{}@example.com", i),
                    if i % 2 == 0 {
                        UserRole::Guest
                    } else {
                        UserRole::Employee
                    },
                    Some(&format!("user{}", i)),
                )
            })
            .collect::<Vec<_>>();

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        // Первая страница, 2 элемента
        let result = service
            .list(Some("1".to_string()), Some("2".to_string()), None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 2);
        assert_eq!(response.total, 5);
        assert_eq!(response.current_filter.page(), 1);
        assert_eq!(response.current_filter.per_page(), 2);

        // Вторая страница
        let result = service
            .list(Some("2".to_string()), Some("2".to_string()), None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 2);

        // Третья страница
        let result = service
            .list(Some("3".to_string()), Some("2".to_string()), None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 1);
    }

    /// Тест получения списка пользователей с фильтрацией по роли
    #[tokio::test]
    async fn test_list_users_with_role_filter() {
        let users = vec![
            create_test_user(Uuid::new_v4(), "user1@example.com", UserRole::Guest, None),
            create_test_user(
                Uuid::new_v4(),
                "admin@example.com",
                UserRole::Admin,
                Some("admin"),
            ),
            create_test_user(Uuid::new_v4(), "user2@example.com", UserRole::Guest, None),
        ];

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        let result = service
            .list(
                Some("1".to_string()),
                Some("10".to_string()),
                Some("Admin".to_string()),
                None,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 1);
        assert_eq!(response.users[0].role, UserRole::Admin);
        assert_eq!(response.total, 1);
    }

    /// Тест получения списка пользователей с поиском
    #[tokio::test]
    async fn test_list_users_with_search() {
        let users = vec![
            create_test_user(
                Uuid::new_v4(),
                "john@example.com",
                UserRole::Guest,
                Some("john_doe"),
            ),
            create_test_user(
                Uuid::new_v4(),
                "jane@example.com",
                UserRole::Employee,
                Some("jane_smith"),
            ),
            create_test_user(
                Uuid::new_v4(),
                "bob@example.com",
                UserRole::Guest,
                Some("bob_johnson"),
            ),
        ];

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        // Поиск по email
        let result = service
            .list(
                Some("1".to_string()),
                Some("10".to_string()),
                None,
                Some("john@".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 1);
        assert!(response.users[0].email.contains("john@"));

        // Поиск по username
        let result = service
            .list(
                Some("1".to_string()),
                Some("10".to_string()),
                None,
                Some("smith".to_string()),
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 1);
        assert_eq!(
            response.users[0].info.username,
            Some("jane_smith".to_string())
        );

        // Поиск по имени
        let result = service
            .list(
                Some("1".to_string()),
                Some("10".to_string()),
                None,
                Some("Test".to_string()), // Все пользователи имеют first_name = "Test"
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.users.len(), 3);
    }

    /// Тест получения списка пользователей с параметрами по умолчанию
    #[tokio::test]
    async fn test_list_users_default_params() {
        let users = vec![
            create_test_user(Uuid::new_v4(), "user1@example.com", UserRole::Guest, None),
            create_test_user(Uuid::new_v4(), "user2@example.com", UserRole::Guest, None),
        ];

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.list(None, None, None, None).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(response.current_filter.per_page(), DEFAULT_PER_PAGE);
        assert_eq!(response.users.len(), 2);
    }

    /// Тест успешной аутентификации
    #[tokio::test]
    async fn test_login_success() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        // Создаем пользователя
        let created = service
            .signup("user@example.com", "correct_p@sSword123", None)
            .await
            .unwrap();

        // Пытаемся войти
        let result = service.signin(&created.email, "correct_p@sSword123").await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.email, "user@example.com");
    }

    /// Тест аутентификации с неверными учетными данными
    #[tokio::test]
    async fn test_login_invalid_credentials() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        // Создаем пользователя
        service
            .signup("user@example.com", "correct_p@sSword123", None)
            .await
            .unwrap();

        // Пытаемся войти с неправильным паролем
        let result = service
            .signin("user@example.com", "wrong_p@sSword123")
            .await;

        assert!(result.is_err());
    }

    /// Тест аутентификации несуществующего пользователя
    #[tokio::test]
    async fn test_login_user_not_found() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service
            .signin("nonexistent@example.com", "p@sSword123")
            .await;
        assert!(result.is_err());
    }

    /// Тест удаления пользователя
    #[tokio::test]
    async fn test_delete_user_success() {
        let user_id = Uuid::new_v4();
        let test_user = create_test_user(user_id, "delete@example.com", UserRole::Guest, None);

        let test_repo = TestUsersRepo::with_users(vec![test_user.clone()]);
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.delete(&user_id.to_string()).await;
        assert!(result.is_ok());
        let deleted_user = result.unwrap();
        assert_eq!(deleted_user.user_id, user_id);

        // Проверяем, что пользователь действительно удален
        let get_result = service.get_by_id(&user_id.to_string()).await;
        assert!(get_result.is_err());
    }

    /// Тест удаления несуществующего пользователя
    #[tokio::test]
    async fn test_delete_user_not_found() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.delete(&Uuid::new_v4().to_string()).await;
        assert!(result.is_err());
    }

    /// Тест удаления пользователя с невалидным UUID
    #[tokio::test]
    async fn test_delete_user_invalid_uuid() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let result = service.delete("invalid-uuid").await;
        assert!(result.is_err());
    }

    /// Тест обновления пользователя
    #[tokio::test]
    async fn test_update_user_success() {
        let user_id = Uuid::new_v4();
        let test_user =
            create_test_user(user_id, "old@example.com", UserRole::Guest, Some("olduser"));

        let test_repo = TestUsersRepo::with_users(vec![test_user.clone()]);
        let service = UsersService::new(Arc::new(test_repo));

        // Создаем обновленного пользователя
        let mut updated_user = test_user.clone();
        updated_user.email = "new@example.com".to_string();
        updated_user.info.username = Some("newuser".to_string());
        updated_user.role = UserRole::Admin;

        let result = service
            .update(&user_id.to_string(), updated_user.clone().into())
            .await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.email, "new@example.com");
        assert_eq!(updated.role, UserRole::Admin);
        assert_eq!(updated.info.username, Some("newuser".to_string()));

        // Проверяем, что данные обновились
        let get_result = service.get_by_id(&user_id.to_string()).await.unwrap();
        assert_eq!(get_result.email, "new@example.com");
    }

    /// Тест обновления несуществующего пользователя
    #[tokio::test]
    async fn test_update_user_not_found() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        let user = create_test_user(Uuid::new_v4(), "test@example.com", UserRole::Guest, None);
        let result = service
            .update(&Uuid::new_v4().to_string(), user.into())
            .await;
        assert!(result.is_err());
    }

    /// Тест пограничных случаев пагинации
    #[tokio::test]
    async fn test_pagination_edge_cases() {
        let users = vec![create_test_user(
            Uuid::new_v4(),
            "user1@example.com",
            UserRole::Guest,
            None,
        )];

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        // Страница за пределами диапазона
        let result = service
            .list(
                Some("10".to_string()), // Несуществующая страница
                Some("10".to_string()),
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.users.is_empty());
        assert_eq!(response.total, 1);

        // Нулевая страница (должна стать 1)
        let result = service
            .list(Some("0".to_string()), Some("10".to_string()), None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(response.users.len(), 1);
    }

    /// Тест пограничных значений per_page
    #[tokio::test]
    async fn test_per_page_edge_cases() {
        let users = (0..101)
            .map(|i| {
                create_test_user(
                    Uuid::new_v4(),
                    &format!("user{}@example.com", i),
                    UserRole::Guest,
                    None,
                )
            })
            .collect();

        let test_repo = TestUsersRepo::with_users(users);
        let service = UsersService::new(Arc::new(test_repo));

        // per_page меньше минимума
        let result = service
            .list(Some("1".to_string()), Some("5".to_string()), None, None)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_filter.per_page(), 5);
        assert_eq!(response.users.len(), 5);

        // per_page больше максимума
        let result = service
            .list(
                Some("1".to_string()),
                Some("150".to_string()), // Больше MAX_PER_PAGE
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_filter.per_page(), MAX_PER_PAGE);
        assert_eq!(response.users.len(), MAX_PER_PAGE as usize);

        // per_page на границе максимума
        let result = service
            .list(
                Some("1".to_string()),
                Some("100".to_string()), // Равно MAX_PER_PAGE
                None,
                None,
            )
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.current_filter.per_page(), MAX_PER_PAGE);
        assert_eq!(response.users.len(), MAX_PER_PAGE as usize);
    }

    /// Интеграционный тест: полный цикл операций
    #[tokio::test]
    async fn test_integration_workflow() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        // 1. Создаем пользователя
        let created = service
            .signup("integration@example.com", "p@sSword123", Some("Employee"))
            .await
            .unwrap();

        let user_id = created.user_id;

        // 2. Получаем пользователя по ID
        let retrieved = service.get_by_id(&user_id.to_string()).await.unwrap();
        assert_eq!(retrieved.user_id, user_id);
        assert_eq!(retrieved.email, "integration@example.com");
        assert_eq!(retrieved.role, UserRole::Employee);

        // 3. Получаем по email (с нормализацией)
        let by_email = service
            .get_user_info("INTEGRATION@EXAMPLE.COM")
            .await
            .unwrap();
        assert_eq!(by_email.user_id, user_id);

        // 4. Обновляем пользователя
        let mut updated_user = retrieved.clone();
        updated_user.info.username = Some("integration_user".to_string());
        let updated = service
            .update(&user_id.to_string(), updated_user.into())
            .await
            .unwrap();
        assert_eq!(updated.info.username, Some("integration_user".to_string()));

        // 5. Ищем пользователя в списке
        let list_result = service
            .list(
                Some("1".to_string()),
                Some("10".to_string()),
                Some("Employee".to_string()),
                Some("integration".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(list_result.total, 1);
        assert_eq!(list_result.users[0].user_id, user_id);

        // 6. Входим в систему
        let login_result = service
            .signin("integration@example.com", "p@sSword123")
            .await
            .unwrap();
        assert_eq!(login_result.user_id, user_id);

        // 7. Удаляем пользователя
        let deleted = service.delete(&user_id.to_string()).await.unwrap();
        assert_eq!(deleted.user_id, user_id);

        // 8. Проверяем, что пользователь удален
        let get_after_delete = service.get_by_id(&user_id.to_string()).await;
        assert!(get_after_delete.is_err());
    }

    /// Тест обработки невалидных входных данных
    #[tokio::test]
    async fn test_invalid_input_handling() {
        let test_repo = TestUsersRepo::new();
        let service = UsersService::new(Arc::new(test_repo));

        // Пустой email
        let result = service.signup("", "p@sSword123", None).await;
        assert!(result.is_err());

        // Пустой пароль
        let result = service.signup("test@example.com", "", None).await;
        assert!(result.is_err());

        // Невалидный email для поиска
        let result = service.get_user_info("").await;
        assert!(result.is_err());
    }
}
