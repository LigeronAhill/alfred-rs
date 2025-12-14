mod pg_users_repository;
use crate::{
    AppResult,
    models::{SigninData, SignupData, User, UserRole},
};
use async_trait::async_trait;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Номер страницы по умолчанию (первая страница)
pub const DEFAULT_PAGE_NUM: u32 = 1;
/// Минимальное количество элементов на странице
pub const DEFAULT_PER_PAGE: u32 = 10;
/// Максимальное количество элементов на странице
pub const MAX_PER_PAGE: u32 = 100;

/// Трейт репозитория пользователей
///
/// Определяет контракт для операций с пользователями в базе данных.
/// Все методы асинхронны и возвращают `AppResult<T>` для обработки ошибок.
#[async_trait]
pub trait UsersRepository: Send + Sync {
    /// Создает нового пользователя в базе данных
    async fn create(&self, signup_data: SignupData) -> AppResult<User>;
    /// Получает пользователя по идентификатору
    async fn get(&self, id: uuid::Uuid) -> AppResult<User>;
    /// Получает список пользователей с применением фильтров и пагинации
    async fn list(&self, filter: UsersFilter) -> AppResult<Vec<User>>;
    /// Получает общее количество пользователей, соответствующих фильтрам
    async fn total(&self, filter: UsersFilter) -> AppResult<u32>;
    /// Находит пользователя по email адресу
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    /// Обновляет данные пользователя
    async fn update(&self, id: uuid::Uuid, user: User) -> AppResult<User>;
    /// Удаляет пользователя по идентификатору
    async fn delete(&self, id: uuid::Uuid) -> AppResult<User>;
    /// Проверяет правильность пароля пользователя
    async fn verify_user(&self, signin_data: SigninData) -> AppResult<bool>;
}
/// Фильтр для поиска пользователей с поддержкой пагинации
///
/// Используется для фильтрации, поиска и пагинации пользователей в методах
/// `list` и `total`. Поддерживает поиск по нескольким полям и фильтрацию по роли.
#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
pub struct UsersFilter {
    /// Номер страницы (начиная с 1)
    #[builder(setter(custom), default = DEFAULT_PAGE_NUM)]
    page: u32,
    /// Количество элементов на странице
    ///
    /// Значение автоматически ограничивается диапазоном `MIN_PER_PAGE`..=`MAX_PER_PAGE`.
    #[builder(setter(custom), default = DEFAULT_PER_PAGE)]
    per_page: u32,
    /// Фильтр по роли пользователя
    ///
    /// Если установлено `None`, фильтрация по роли не применяется.
    #[builder(default)]
    role: Option<UserRole>,
    /// Строка для поиска пользователей
    ///
    /// Поиск выполняется по email, имени пользователя, имени и фамилии.
    /// Используется регистронезависимый поиск (ILIKE).
    #[builder(default)]
    search_string: Option<String>,
}
impl Default for UsersFilter {
    /// Создает фильтр со значениями по умолчанию:
    /// - page = `DEFAULT_PAGE_NUM` (1)
    /// - per_page = `MIN_PER_PAGE` (10)
    /// - role = `None`
    /// - search_string = `None`
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}
impl UsersFilter {
    /// Создает новый билдер для `UsersFilter`
    ///
    /// # Возвращает
    ///
    /// Новый экземпляр `UsersFilterBuilder`
    pub fn builder() -> UsersFilterBuilder {
        UsersFilterBuilder::default()
    }
    /// Возвращает номер текущей страницы
    pub fn page(&self) -> u32 {
        self.page
    }
    /// Возвращает количество элементов на странице
    pub fn per_page(&self) -> u32 {
        self.per_page
    }
    /// Возвращает фильтр по роли в виде строкового среза
    ///
    /// # Возвращает
    ///
    /// - `Some(&str)` - если фильтр по роли установлен
    /// - `None` - если фильтр по роли не установлен
    pub fn role(&self) -> Option<&str> {
        self.role.as_ref().map(|r| r.as_ref())
    }
    /// Возвращает строку поиска
    ///
    /// # Возвращает
    ///
    /// - `Some(&String)` - если строка поиска установлена
    /// - `None` - если строка поиска не установлена
    pub fn search_string(&self) -> Option<&String> {
        self.search_string.as_ref()
    }
}

impl UsersFilterBuilder {
    /// Устанавливает номер страницы с валидацией
    ///
    /// # Аргументы
    ///
    /// * `page` - Номер страницы
    ///
    /// # Поведение
    ///
    /// - Если `page < 1`, устанавливается `DEFAULT_PAGE_NUM`
    /// - В противном случае устанавливается переданное значение
    pub fn page(&mut self, page: u32) -> &mut Self {
        if page < 1 {
            let _ = self.page.insert(DEFAULT_PAGE_NUM);
        } else {
            let _ = self.page.insert(page);
        }
        self
    }
    /// Устанавливает количество элементов на странице с валидацией
    ///
    /// # Аргументы
    ///
    /// * `per_page` - Количество элементов на странице
    ///
    /// # Поведение
    ///
    /// - Если `per_page < MIN_PER_PAGE`, устанавливается `MIN_PER_PAGE`
    /// - Если `per_page > MAX_PER_PAGE`, устанавливается `MAX_PER_PAGE`
    /// - В противном случае устанавливается переданное значение
    pub fn per_page(&mut self, per_page: u32) -> &mut Self {
        if per_page < 1 {
            let _ = self.per_page.insert(DEFAULT_PER_PAGE);
        } else if per_page > MAX_PER_PAGE {
            let _ = self.per_page.insert(MAX_PER_PAGE);
        } else {
            let _ = self.per_page.insert(per_page);
        }
        self
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let filter = UsersFilter::default();

        assert_eq!(filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(filter.per_page(), DEFAULT_PER_PAGE);
        assert!(filter.role().is_none());
        assert!(filter.search_string().is_none());
    }

    #[test]
    fn test_builder_with_all_fields() {
        let filter = UsersFilter::builder()
            .page(2)
            .per_page(20)
            .role(Some(UserRole::Admin))
            .search_string(Some("john".to_string()))
            .build()
            .unwrap();

        assert_eq!(filter.page(), 2);
        assert_eq!(filter.per_page(), 20);
        assert_eq!(filter.role(), Some("Администратор"));
        assert_eq!(filter.search_string(), Some(&"john".to_string()));
    }

    #[test]
    fn test_builder_partial_fields() {
        let filter = UsersFilter::builder().page(3).per_page(15).build().unwrap();

        assert_eq!(filter.page(), 3);
        assert_eq!(filter.per_page(), 15);
        assert!(filter.role().is_none());
        assert!(filter.search_string().is_none());
    }

    #[test]
    fn test_builder_min_page_validation() {
        // Страница 0 должна превратиться в DEFAULT_PAGE_NUM
        let filter = UsersFilter::builder().page(0).build().unwrap();
        assert_eq!(filter.page(), DEFAULT_PAGE_NUM);

        // Страница 1 должна остаться 1
        let filter = UsersFilter::builder().page(1).build().unwrap();
        assert_eq!(filter.page(), 1);

        // Страница 100 должна остаться 100
        let filter = UsersFilter::builder().page(100).build().unwrap();
        assert_eq!(filter.page(), 100);
    }

    #[test]
    fn test_builder_per_page_validation() {
        // Меньше минимума
        let filter = UsersFilter::builder().per_page(0).build().unwrap();
        assert_eq!(filter.per_page(), DEFAULT_PER_PAGE);

        let filter = UsersFilter::builder().per_page(5).build().unwrap();
        assert_eq!(filter.per_page(), 5);

        // В пределах диапазона
        let filter = UsersFilter::builder().per_page(25).build().unwrap();
        assert_eq!(filter.per_page(), 25);

        let filter = UsersFilter::builder().per_page(50).build().unwrap();
        assert_eq!(filter.per_page(), 50);

        // Больше максимума
        let filter = UsersFilter::builder().per_page(150).build().unwrap();
        assert_eq!(filter.per_page(), MAX_PER_PAGE);

        let filter = UsersFilter::builder().per_page(200).build().unwrap();
        assert_eq!(filter.per_page(), MAX_PER_PAGE);

        // Граничные значения
        let filter = UsersFilter::builder()
            .per_page(DEFAULT_PER_PAGE)
            .build()
            .unwrap();
        assert_eq!(filter.per_page(), DEFAULT_PER_PAGE);

        let filter = UsersFilter::builder()
            .per_page(MAX_PER_PAGE)
            .build()
            .unwrap();
        assert_eq!(filter.per_page(), MAX_PER_PAGE);
    }

    #[test]
    fn test_builder_role_conversion() {
        // Все возможные роли
        let roles = vec![
            UserRole::Owner,
            UserRole::Admin,
            UserRole::Employee,
            UserRole::Guest,
        ];

        for role in roles {
            let filter = UsersFilter::builder()
                .role(Some(role.clone()))
                .build()
                .unwrap();

            assert_eq!(filter.role(), Some(role.as_ref()));
        }

        // Отсутствие роли
        let filter = UsersFilter::builder().role(None).build().unwrap();

        assert!(filter.role().is_none());
    }

    #[test]
    fn test_builder_search_string() {
        // С поисковым запросом
        let search = "test query".to_string();
        let filter = UsersFilter::builder()
            .search_string(Some(search.clone()))
            .build()
            .unwrap();

        assert_eq!(filter.search_string(), Some(&search));

        // Без поискового запроса
        let filter = UsersFilter::builder().search_string(None).build().unwrap();

        assert!(filter.search_string().is_none());

        // Пустая строка поиска
        let filter = UsersFilter::builder()
            .search_string(Some("".to_string()))
            .build()
            .unwrap();

        assert_eq!(filter.search_string(), Some(&"".to_string()));
    }

    #[test]
    fn test_builder_fluent_interface() {
        let filter = UsersFilter::builder()
            .page(2)
            .per_page(30)
            .role(Some(UserRole::Employee))
            .search_string(Some("alice".to_string()))
            .build()
            .unwrap();

        assert_eq!(filter.page(), 2);
        assert_eq!(filter.per_page(), 30);
        assert_eq!(filter.role(), Some(UserRole::Employee.as_ref()));
        assert_eq!(filter.search_string(), Some(&"alice".to_string()));
    }

    #[test]
    fn test_builder_method_chaining() {
        let filter = UsersFilter::builder().page(1).per_page(10).build().unwrap();

        let filter2 = UsersFilter::builder()
            .page(2)
            .per_page(20)
            .role(Some(UserRole::Admin))
            .build()
            .unwrap();

        assert_eq!(filter.page(), 1);
        assert_eq!(filter.per_page(), 10);
        assert!(filter.role().is_none());

        assert_eq!(filter2.page(), 2);
        assert_eq!(filter2.per_page(), 20);
        assert_eq!(filter2.role(), Some(UserRole::Admin.as_ref()));
    }

    #[test]
    fn test_builder_clone() {
        let filter1 = UsersFilter::builder()
            .page(2)
            .per_page(25)
            .role(Some(UserRole::Guest))
            .search_string(Some("test".to_string()))
            .build()
            .unwrap();

        let filter2 = filter1.clone();

        assert_eq!(filter1.page(), filter2.page());
        assert_eq!(filter1.per_page(), filter2.per_page());
        assert_eq!(filter1.role(), filter2.role());
        assert_eq!(filter1.search_string(), filter2.search_string());
    }

    #[test]
    fn test_builder_debug() {
        let filter = UsersFilter::builder()
            .page(1)
            .per_page(10)
            .role(Some(UserRole::Owner))
            .search_string(Some("debug".to_string()))
            .build()
            .unwrap();

        let debug_output = format!("{:?}", filter);

        assert!(debug_output.contains("page"));
        assert!(debug_output.contains("per_page"));
        assert!(debug_output.contains("role"));
        assert!(debug_output.contains("search_string"));
    }

    #[test]
    fn test_builder_serialize() {
        use serde_json;

        let filter = UsersFilter::builder()
            .page(1)
            .per_page(20)
            .role(Some(UserRole::Admin))
            .search_string(Some("serialize".to_string()))
            .build()
            .unwrap();

        let json = serde_json::to_string(&filter).unwrap();

        assert!(json.contains("\"page\":1"));
        assert!(json.contains("\"per_page\":20"));
        assert!(json.contains("\"role\":\"Администратор\""));
        assert!(json.contains("\"search_string\":\"serialize\""));

        // Тест десериализации
        let deserialized: UsersFilter = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.page(), 1);
        assert_eq!(deserialized.per_page(), 20);
        assert_eq!(deserialized.role(), Some("Администратор"));
        assert_eq!(deserialized.search_string(), Some(&"serialize".to_string()));
    }

    #[test]
    fn test_builder_with_only_search() {
        let filter = UsersFilter::builder()
            .search_string(Some("only search".to_string()))
            .build()
            .unwrap();

        assert_eq!(filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(filter.per_page(), DEFAULT_PER_PAGE);
        assert!(filter.role().is_none());
        assert_eq!(filter.search_string(), Some(&"only search".to_string()));
    }

    #[test]
    fn test_builder_with_only_role() {
        let filter = UsersFilter::builder()
            .role(Some(UserRole::Employee))
            .build()
            .unwrap();

        assert_eq!(filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(filter.per_page(), DEFAULT_PER_PAGE);
        assert_eq!(filter.role(), Some("Сотрудник"));
        assert!(filter.search_string().is_none());
    }

    #[test]
    fn test_builder_edge_cases() {
        // Очень большие значения
        let filter = UsersFilter::builder()
            .page(u32::MAX)
            .per_page(u32::MAX)
            .build()
            .unwrap();

        assert_eq!(filter.page(), u32::MAX);
        assert_eq!(filter.per_page(), MAX_PER_PAGE); // ограничено MAX_PER_PAGE

        // Специальные символы в поиске
        let special_search = "test@email.com #tag $100".to_string();
        let filter = UsersFilter::builder()
            .search_string(Some(special_search.clone()))
            .build()
            .unwrap();

        assert_eq!(filter.search_string(), Some(&special_search));
    }

    #[test]
    fn test_builder_methods_return_self() {
        let mut builder = UsersFilter::builder();

        // Проверяем, что методы возвращают &mut Self
        let builder = builder.page(1);
        let builder = builder.per_page(10);
        let builder = builder.role(Some(UserRole::Guest));
        let builder = builder.search_string(Some("test".to_string()));

        let filter = builder.build().unwrap();

        assert_eq!(filter.page(), 1);
        assert_eq!(filter.per_page(), 10);
        assert_eq!(filter.role(), Some(UserRole::Guest.as_ref()));
        assert_eq!(filter.search_string(), Some(&"test".to_string()));
    }

    #[test]
    fn test_constants_are_used() {
        let default_filter = UsersFilter::default();

        assert_eq!(default_filter.page(), DEFAULT_PAGE_NUM);
        assert_eq!(default_filter.per_page(), DEFAULT_PER_PAGE);

        let custom_filter = UsersFilter::builder()
            .per_page(DEFAULT_PER_PAGE)
            .build()
            .unwrap();

        assert_eq!(custom_filter.per_page(), DEFAULT_PER_PAGE);
    }
}
