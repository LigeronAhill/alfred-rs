//! Модуль для работы с пользователями
//!
//! Этот модуль содержит структуры и функции для управления пользователями системы,
//! включая их учетные данные, роли и личную информацию.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use tracing::instrument;
use validator::{Validate, ValidationError};

use crate::{AppError, AppResult};

/// Представляет пользователя системы
///
/// Содержит основную информацию о пользователе, включая учетные данные,
/// роль, личную информацию и временные метки создания/обновления.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct User {
    /// Уникальный идентификатор пользователя
    pub user_id: uuid::Uuid,

    /// Email пользователя (уникальный)
    pub email: String,

    /// Хэш пароля пользователя
    ///
    /// Поле пропускается при сериализации в ответах API для безопасности.
    #[serde(skip_serializing)]
    pub password_hash: String,

    /// Роль пользователя в системе
    pub role: UserRole,

    /// Дополнительная информация о пользователе
    pub info: UserInfo,

    /// Дата и время создания пользователя
    pub created: chrono::NaiveDateTime,

    /// Дата и время последнего обновления пользователя
    pub updated: chrono::NaiveDateTime,
}

/// Дополнительная информация о пользователе
///
/// Содержит опциональные поля с личной информацией пользователя.
/// Все поля пропускаются при сериализации, если имеют значение `None`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub struct UserInfo {
    /// Имя пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,

    /// Отчество пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,

    /// Фамилия пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,

    /// Уникальное имя пользователя (никнейм)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// URL аватара пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,

    /// Биография или описание пользователя
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
}

impl UserInfo {
    /// Возвращает полное имя пользователя в формате "Фамилия Имя Отчество"
    ///
    /// # Возвращает
    ///
    /// * `Some(String)` - если указаны хотя бы имя и фамилия
    /// * `None` - если имя или фамилия отсутствуют
    #[instrument(name = "users full name", skip(self))]
    pub fn full_name(&self) -> Option<String> {
        match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => {
                let mut parts = vec![last.as_str(), first.as_str()];
                if let Some(middle) = &self.middle_name {
                    parts.push(middle.as_str());
                }
                Some(parts.join(" "))
            }
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            _ => None,
        }
    }

    /// Проверяет, содержит ли профиль какую-либо личную информацию
    ///
    /// # Возвращает
    ///
    /// `true` если указано хотя бы одно из: имя, фамилия или имя пользователя.
    #[instrument(name = "has user profile data", skip(self))]
    pub fn has_profile_data(&self) -> bool {
        self.first_name.is_some() || self.last_name.is_some() || self.username.is_some()
    }
}

/// Роль пользователя в системе
///
/// Определяет уровень доступа и привилегии пользователя.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
pub enum UserRole {
    /// Владелец системы - полный доступ ко всем функциям
    #[serde(rename = "Владелец")]
    Owner,

    /// Администратор - доступ к управлению пользователями и настройками
    #[serde(rename = "Администратор")]
    Admin,

    /// Сотрудник - базовый доступ к рабочим функциям
    #[serde(rename = "Сотрудник")]
    Employee,

    /// Гость - минимальный доступ, только просмотр
    #[serde(rename = "Гость")]
    #[default]
    Guest,
}

impl UserRole {
    /// Проверяет, является ли роль административной
    ///
    /// Административными считаются роли `Owner` и `Admin`.
    ///
    /// # Возвращает
    ///
    /// `true` если роль `Owner` или `Admin`, иначе `false`.
    #[instrument(name = "is admin", skip(self))]
    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Owner | UserRole::Admin)
    }

    /// Возвращает срез всех возможных ролей
    ///
    /// # Возвращает
    ///
    /// Ссылку на статический массив всех ролей в порядке:
    /// `[Owner, Admin, Employee, Guest]`
    #[instrument(name = "get all roles")]
    pub fn all() -> &'static [Self] {
        &[
            UserRole::Owner,
            UserRole::Admin,
            UserRole::Employee,
            UserRole::Guest,
        ]
    }

    /// Возвращает итератор по всем ролям
    ///
    /// # Возвращает
    ///
    /// Итератор, который yields все возможные роли.
    #[instrument(name = "get roles iterator")]
    pub fn iter() -> impl Iterator<Item = &'static Self> {
        Self::all().iter()
    }

    /// Возвращает вектор всех ролей
    ///
    /// # Возвращает
    ///
    /// Вектор со всеми возможными ролями.
    /// В отличие от `all()`, возвращает владеемую коллекцию.
    #[instrument(name = "get roles vector")]
    pub fn values() -> Vec<Self> {
        vec![
            UserRole::Owner,
            UserRole::Admin,
            UserRole::Employee,
            UserRole::Guest,
        ]
    }
}

impl Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            UserRole::Owner => "Владелец",
            UserRole::Admin => "Администратор",
            UserRole::Employee => "Сотрудник",
            UserRole::Guest => "Гость",
        };
        write!(f, "{string}")
    }
}

impl FromStr for UserRole {
    type Err = AppError;

    /// Парсит строку в `UserRole`
    ///
    /// Поддерживает как русские, так и английские названия ролей
    /// в любом регистре.
    ///
    /// # Аргументы
    ///
    /// * `s` - Строка для парсинга
    ///
    /// # Возвращает
    ///
    /// * `Ok(UserRole)` - если строка соответствует одной из ролей
    /// * `Err(AppError::InvalidUserRole)` - если строка не соответствует ни одной роли
    #[instrument(name = "parse user role")]
    fn from_str(s: &str) -> AppResult<Self> {
        match s.to_lowercase().as_str() {
            "владелец" | "owner" => Ok(UserRole::Owner),
            "администратор" | "admin" => Ok(UserRole::Admin),
            "сотрудник" | "employee" => Ok(UserRole::Employee),
            "гость" | "guest" => Ok(UserRole::Guest),
            _ => Err(AppError::InvalidUserRole(s.to_string())),
        }
    }
}

impl AsRef<str> for UserRole {
    /// Возвращает строковое представление роли на русском языке
    fn as_ref(&self) -> &str {
        match self {
            UserRole::Owner => "Владелец",
            UserRole::Admin => "Администратор",
            UserRole::Employee => "Сотрудник",
            UserRole::Guest => "Гость",
        }
    }
}

/// Данные для регистрации нового пользователя
///
/// Используется при создании нового аккаунта пользователя.
/// Все поля проходят валидацию перед использованием.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default, Validate)]
pub struct SignupData {
    /// Email пользователя
    ///
    /// Должен быть валидным email адресом.
    #[validate(email)]
    pub email: String,

    /// Пароль пользователя
    ///
    /// Должен соответствовать требованиям безопасности:
    /// * 8-64 символа
    /// * Содержать цифры, буквы в разных регистрах и специальные символы
    /// * Не содержать пробелов
    /// * Не быть распространённым паролем
    #[validate(
        length(
            min = 8,
            max = 64,
            message = "Пароль должен содержать от 8 до 64 символов"
        ),
        custom(function = "validate_password")
    )]
    pub password: String,

    /// Роль нового пользователя
    pub role: UserRole,
}

impl SignupData {
    /// Создает новый `SignupData` с валидацией входных данных
    ///
    /// # Аргументы
    ///
    /// * `email` - Email пользователя (будет приведен к нижнему регистру и обрезан)
    /// * `password` - Пароль пользователя
    /// * `role` - Роль пользователя в виде строки
    ///
    /// # Возвращает
    ///
    /// * `Ok(SignupData)` - если все данные валидны
    /// * `Err(AppError::InvalidUserRole)` - если роль невалидна
    /// * `Err(AppError::ValidationErrors)` - если данные не проходят валидацию
    #[instrument(name = "try new signup data", skip(password))]
    pub fn try_new(email: &str, password: &str, role: &str) -> AppResult<Self> {
        let Ok(role) = UserRole::from_str(role) else {
            return Err(AppError::InvalidUserRole(role.to_string()));
        };
        let res = Self {
            email: email.trim().to_lowercase(),
            password: password.to_string(),
            role,
        };
        match res.validate() {
            Ok(_) => Ok(res),
            Err(err) => Err(AppError::ValidationErrors(err)),
        }
    }
}

impl TryFrom<(&str, &str, &str)> for SignupData {
    type Error = AppError;

    /// Создает `SignupData` из кортежа строк
    ///
    /// # Аргументы
    ///
    /// * `(email, password, role)` - Кортеж строк (email, пароль, роль)
    ///
    /// # Возвращает
    ///
    /// * `Ok(SignupData)` - если все данные валидны
    /// * `Err(AppError)` - если данные невалидны
    fn try_from((email, password, role): (&str, &str, &str)) -> Result<Self, Self::Error> {
        Self::try_new(email, password, role)
    }
}

/// Проверяет пароль на соответствие требованиям безопасности
///
/// # Аргументы
///
/// * `password` - Пароль для проверки
///
/// # Возвращает
///
/// * `Ok(())` - если пароль соответствует всем требованиям
/// * `Err(ValidationError)` - если пароль не соответствует требованиям,
///   с описанием всех найденных проблем
#[instrument(name = "validate password", skip(password))]
fn validate_password(password: &str) -> Result<(), ValidationError> {
    let mut errors = Vec::new();

    // Проверка на пробелы
    if password.contains(' ') {
        errors.push("Пароль не должен содержать пробелы");
    }

    // Проверка на распространённые пароли
    let common_passwords = [
        "password",
        "12345678",
        "qwerty",
        "admin123",
        "letmein",
        "welcome",
        "monkey",
        "sunshine",
        "password1",
        "123123",
        "11111111",
        "abcd1234",
        "trustno1",
        "dragon",
        "baseball",
    ];
    if common_passwords
        .iter()
        .any(|&p| password.to_lowercase() == p)
    {
        errors.push("Пароль слишком распространён");
    }

    // Проверка наличия цифр
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push("Пароль должен содержать хотя бы одну цифру");
    }

    // Проверка наличия букв в верхнем регистре
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        errors.push("Пароль должен содержать хотя бы одну заглавную букву");
    }

    // Проверка наличия букв в нижнем регистре
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        errors.push("Пароль должен содержать хотя бы одну строчную букву");
    }

    // Проверка наличия специальных символов
    if !password.chars().any(is_special_char) {
        errors.push("Пароль должен содержать хотя бы один специальный символ");
    }

    if !errors.is_empty() {
        let mut error = validator::ValidationError::new("password");
        error.message = Some(format!("Требования к паролю: {}", errors.join(", ")).into());
        return Err(error);
    }

    Ok(())
}

/// Проверяет, является ли символ специальным
///
/// Специальные символы включают: !@#$%^&*()_-+=<>?/{}~|[]"\\'`
///
/// # Аргументы
///
/// * `c` - Символ для проверки
///
/// # Возвращает
///
/// `true` если символ является специальным, иначе `false`
const fn is_special_char(c: char) -> bool {
    matches!(
        c,
        '!' | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '_'
            | '-'
            | '+'
            | '='
            | '<'
            | '>'
            | '?'
            | '/'
            | '{'
            | '}'
            | '~'
            | '|'
            | '['
            | ']'
            | '"'
            | '\\'
            | '\''
            | '`'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn test_user_role_from_str() {
        // Русские названия в разных регистрах
        assert_eq!("владелец".parse::<UserRole>().unwrap(), UserRole::Owner);
        assert_eq!("ВЛАДЕЛЕЦ".parse::<UserRole>().unwrap(), UserRole::Owner);
        assert_eq!("Владелец".parse::<UserRole>().unwrap(), UserRole::Owner);

        // Английские названия в разных регистрах
        assert_eq!("owner".parse::<UserRole>().unwrap(), UserRole::Owner);
        assert_eq!("OWNER".parse::<UserRole>().unwrap(), UserRole::Owner);
        assert_eq!("Owner".parse::<UserRole>().unwrap(), UserRole::Owner);

        // Все роли
        assert_eq!(
            "администратор".parse::<UserRole>().unwrap(),
            UserRole::Admin
        );
        assert_eq!("admin".parse::<UserRole>().unwrap(), UserRole::Admin);
        assert_eq!("сотрудник".parse::<UserRole>().unwrap(), UserRole::Employee);
        assert_eq!("employee".parse::<UserRole>().unwrap(), UserRole::Employee);
        assert_eq!("гость".parse::<UserRole>().unwrap(), UserRole::Guest);
        assert_eq!("guest".parse::<UserRole>().unwrap(), UserRole::Guest);

        // Невалидные роли
        assert!("неизвестная".parse::<UserRole>().is_err());
        assert!("".parse::<UserRole>().is_err());
        assert!("user".parse::<UserRole>().is_err());
    }

    #[test]
    fn test_user_role_display() {
        assert_eq!(UserRole::Owner.to_string(), "Владелец");
        assert_eq!(UserRole::Admin.to_string(), "Администратор");
        assert_eq!(UserRole::Employee.to_string(), "Сотрудник");
        assert_eq!(UserRole::Guest.to_string(), "Гость");
    }

    #[test]
    fn test_user_role_is_admin() {
        assert!(UserRole::Owner.is_admin());
        assert!(UserRole::Admin.is_admin());
        assert!(!UserRole::Employee.is_admin());
        assert!(!UserRole::Guest.is_admin());
    }

    #[test]
    fn test_user_role_methods() {
        // all()
        let all_roles = UserRole::all();
        assert_eq!(all_roles.len(), 4);
        assert_eq!(all_roles[0], UserRole::Owner);
        assert_eq!(all_roles[1], UserRole::Admin);
        assert_eq!(all_roles[2], UserRole::Employee);
        assert_eq!(all_roles[3], UserRole::Guest);

        // iter()
        let mut iter = UserRole::iter();
        assert_eq!(iter.next(), Some(&UserRole::Owner));
        assert_eq!(iter.next(), Some(&UserRole::Admin));
        assert_eq!(iter.next(), Some(&UserRole::Employee));
        assert_eq!(iter.next(), Some(&UserRole::Guest));
        assert_eq!(iter.next(), None);

        // values()
        let values = UserRole::values();
        assert_eq!(
            values,
            vec![
                UserRole::Owner,
                UserRole::Admin,
                UserRole::Employee,
                UserRole::Guest,
            ]
        );
    }

    #[test]
    fn test_user_role_as_ref() {
        assert_eq!(UserRole::Owner.as_ref(), "Владелец");
        assert_eq!(UserRole::Admin.as_ref(), "Администратор");
        assert_eq!(UserRole::Employee.as_ref(), "Сотрудник");
        assert_eq!(UserRole::Guest.as_ref(), "Гость");
    }

    #[test]
    fn test_user_info_full_name() {
        // Полное имя с отчеством
        let info = UserInfo {
            first_name: Some("Иван".to_string()),
            last_name: Some("Иванов".to_string()),
            middle_name: Some("Иванович".to_string()),
            ..Default::default()
        };
        assert_eq!(info.full_name(), Some("Иванов Иван Иванович".to_string()));

        // Полное имя без отчества
        let info = UserInfo {
            first_name: Some("Иван".to_string()),
            last_name: Some("Иванов".to_string()),
            middle_name: None,
            ..Default::default()
        };
        assert_eq!(info.full_name(), Some("Иванов Иван".to_string()));

        // Только имя
        let info = UserInfo {
            first_name: Some("Иван".to_string()),
            last_name: None,
            ..Default::default()
        };
        assert_eq!(info.full_name(), Some("Иван".to_string()));

        // Только фамилия
        let info = UserInfo {
            first_name: None,
            last_name: Some("Иванов".to_string()),
            ..Default::default()
        };
        assert_eq!(info.full_name(), Some("Иванов".to_string()));

        // Нет имени и фамилии
        let info = UserInfo::default();
        assert_eq!(info.full_name(), None);
    }

    #[test]
    fn test_user_info_has_profile_data() {
        // Есть данные
        let info = UserInfo {
            first_name: Some("Иван".to_string()),
            ..Default::default()
        };
        assert!(info.has_profile_data());

        let info = UserInfo {
            last_name: Some("Иванов".to_string()),
            ..Default::default()
        };
        assert!(info.has_profile_data());

        let info = UserInfo {
            username: Some("ivan".to_string()),
            ..Default::default()
        };
        assert!(info.has_profile_data());

        // Нет данных
        let info = UserInfo::default();
        assert!(!info.has_profile_data());
    }

    #[test]
    fn test_signup_data_try_new() {
        // Валидные данные
        let signup = SignupData::try_new("test@example.com", "ValidPass123!", "admin");
        assert!(signup.is_ok());

        let signup_data = signup.unwrap();
        assert_eq!(signup_data.email, "test@example.com");
        assert_eq!(signup_data.password, "ValidPass123!");
        assert_eq!(signup_data.role, UserRole::Admin);

        // Email приводится к нижнему регистру и обрезается
        let signup = SignupData::try_new("  TEST@EXAMPLE.COM  ", "ValidPass123!", "guest");
        assert!(signup.is_ok());
        assert_eq!(signup.unwrap().email, "test@example.com");

        // Невалидная роль
        let signup = SignupData::try_new("test@example.com", "ValidPass123!", "invalid_role");
        assert!(signup.is_err());
        assert!(matches!(signup.unwrap_err(), AppError::InvalidUserRole(_)));

        // Невалидный пароль (слишком короткий)
        let signup = SignupData::try_new("test@example.com", "short", "admin");
        assert!(signup.is_err());
        assert!(matches!(signup.unwrap_err(), AppError::ValidationErrors(_)));
    }

    #[test]
    fn test_signup_data_try_from() {
        let signup = SignupData::try_from(("test@example.com", "ValidPass123!", "employee"));
        assert!(signup.is_ok());
        assert_eq!(signup.unwrap().role, UserRole::Employee);
    }
    #[test]
    fn test_validate_password() {
        // Проверяем различные кейсы валидации пароля
        // (функция не проверяет длину - это делает макрос #[validate(length(...))])

        // ВАЛИДНЫЕ пароли (удовлетворяют всем требованиям кроме длины)
        let valid_passwords = [
            "ValidPass123!",    // Есть всё: заглавные, строчные, цифры, спецсимвол
            "Test@123Password", // Другой спецсимвол
            "My_Pass123",       // Нижнее подчёркивание
            "Secure#123Pass",   // Решётка
            "Password-123",     // Дефис
        ];

        for password in valid_passwords {
            assert!(
                validate_password(password).is_ok(),
                "Пароль '{}' должен быть валидным",
                password
            );
        }

        // НЕВАЛИДНЫЕ пароли (не хватает хотя бы одного требования)

        // Нет цифр
        assert!(validate_password("NoDigitsHere!").is_err());

        // Нет заглавных букв
        assert!(validate_password("nocaps123!").is_err());

        // Нет строчных букв
        assert!(validate_password("NOCAPS123!").is_err());

        // Нет специальных символов
        assert!(validate_password("NoSpecial123").is_err());

        // Содержит пробелы
        assert!(validate_password("Pass with spaces123!").is_err());
        assert!(validate_password("  StartSpace123!").is_err());
        assert!(validate_password("EndSpace123!  ").is_err());

        // Распространённые пароли (точное совпадение в нижнем регистре)
        let common_passwords = [
            "password",
            "12345678",
            "qwerty",
            "admin123",
            "letmein",
            "welcome",
            "monkey",
            "sunshine",
            "password1",
            "123123",
            "11111111",
            "abcd1234",
            "trustno1",
            "dragon",
            "baseball",
        ];

        for password in common_passwords {
            assert!(
                validate_password(password).is_err(),
                "Пароль '{}' должен быть отклонён как распространённый",
                password
            );
        }

        // Проверяем, что похожие на распространённые пароли проходят
        assert!(validate_password("Password123!").is_ok()); // Не "password"
        assert!(validate_password("Qwerty123!").is_ok()); // С заглавной
        assert!(validate_password("adMin123!").is_ok()); // Со спецсимволом

        // Граничные случаи
        assert!(validate_password("").is_err()); // Пустой пароль
        assert!(validate_password(" ").is_err()); // Только пробел
        assert!(validate_password("A!1").is_err()); // Нет строчной буквы
        assert!(validate_password("a!1").is_err()); // Нет заглавной буквы
        assert!(validate_password("Aa!").is_err()); // Нет цифры
        assert!(validate_password("Aa1").is_err()); // Нет спецсимвола
    }

    #[test]
    fn test_is_special_char() {
        // Специальные символы
        assert!(is_special_char('!'));
        assert!(is_special_char('@'));
        assert!(is_special_char('#'));
        assert!(is_special_char('$'));
        assert!(is_special_char('%'));
        assert!(is_special_char('^'));
        assert!(is_special_char('&'));
        assert!(is_special_char('*'));
        assert!(is_special_char('('));
        assert!(is_special_char(')'));
        assert!(is_special_char('_'));
        assert!(is_special_char('-'));
        assert!(is_special_char('+'));
        assert!(is_special_char('='));
        assert!(is_special_char('<'));
        assert!(is_special_char('>'));
        assert!(is_special_char('?'));
        assert!(is_special_char('/'));
        assert!(is_special_char('{'));
        assert!(is_special_char('}'));
        assert!(is_special_char('~'));
        assert!(is_special_char('|'));
        assert!(is_special_char('['));
        assert!(is_special_char(']'));
        assert!(is_special_char('"'));
        assert!(is_special_char('\\'));
        assert!(is_special_char('\''));
        assert!(is_special_char('`'));

        // Не специальные символы
        assert!(!is_special_char('a'));
        assert!(!is_special_char('Z'));
        assert!(!is_special_char('1'));
        assert!(!is_special_char(' '));
        assert!(!is_special_char('.'));
        assert!(!is_special_char(','));
        assert!(!is_special_char(':'));
        assert!(!is_special_char(';'));
    }

    #[test]
    fn test_user_serialization() {
        // Создаем NaiveDateTime без deprecated метода
        let datetime = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();

        // Проверяем, что password_hash пропускается при сериализации
        let user = User {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
            role: UserRole::Admin,
            info: UserInfo::default(),
            created: datetime,
            updated: datetime,
        };

        let json = serde_json::to_string(&user).unwrap();
        assert!(!json.contains("password_hash"));
        assert!(json.contains("test@example.com"));
        assert!(json.contains("Администратор"));
    }

    #[test]
    fn test_user_info_serialization_skip_none() {
        // Проверяем, что None поля пропускаются
        let info = UserInfo {
            first_name: Some("Иван".to_string()),
            last_name: None,
            ..Default::default()
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("first_name"));
        assert!(json.contains("Иван"));
        assert!(!json.contains("last_name"));
        assert!(!json.contains("middle_name"));
        assert!(!json.contains("username"));
        assert!(!json.contains("avatar_url"));
        assert!(!json.contains("bio"));
    }

    #[test]
    fn test_user_role_serialization() {
        // Проверяем сериализацию ролей
        let owner = UserRole::Owner;
        let admin = UserRole::Admin;
        let employee = UserRole::Employee;
        let guest = UserRole::Guest;

        assert_eq!(serde_json::to_string(&owner).unwrap(), "\"Владелец\"");
        assert_eq!(serde_json::to_string(&admin).unwrap(), "\"Администратор\"");
        assert_eq!(serde_json::to_string(&employee).unwrap(), "\"Сотрудник\"");
        assert_eq!(serde_json::to_string(&guest).unwrap(), "\"Гость\"");

        // Проверяем десериализацию
        let owner_deserialized: UserRole = serde_json::from_str("\"Владелец\"").unwrap();
        assert_eq!(owner_deserialized, UserRole::Owner);

        let guest_deserialized: UserRole = serde_json::from_str("\"Гость\"").unwrap();
        assert_eq!(guest_deserialized, UserRole::Guest);
    }

    #[test]
    fn test_signup_data_validation() {
        // Валидные данные
        let valid_signup = SignupData {
            email: "test@example.com".to_string(),
            password: "ValidPass123!".to_string(),
            role: UserRole::Guest,
        };
        assert!(valid_signup.validate().is_ok());

        // Невалидный email
        let invalid_email = SignupData {
            email: "not-an-email".to_string(),
            password: "ValidPass123!".to_string(),
            role: UserRole::Guest,
        };
        assert!(invalid_email.validate().is_err());

        // Невалидный пароль (слишком короткий)
        let short_password = SignupData {
            email: "test@example.com".to_string(),
            password: "short".to_string(),
            role: UserRole::Guest,
        };
        assert!(short_password.validate().is_err());
    }

    #[test]
    fn test_default_values() {
        // UserRole по умолчанию
        let default_role = UserRole::default();
        assert_eq!(default_role, UserRole::Guest);

        // UserInfo по умолчанию
        let default_info = UserInfo::default();
        assert!(default_info.first_name.is_none());
        assert!(default_info.last_name.is_none());
        assert!(default_info.username.is_none());

        // SignupData по умолчанию
        let default_signup = SignupData::default();
        assert!(default_signup.email.is_empty());
        assert!(default_signup.password.is_empty());
        assert_eq!(default_signup.role, UserRole::Guest);
    }

    #[test]
    fn test_equality_and_hash() {
        // Создаем NaiveDateTime без deprecated метода
        let datetime = chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc();

        let user1 = User {
            user_id: uuid::Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hash1".to_string(),
            role: UserRole::Admin,
            info: UserInfo::default(),
            created: datetime,
            updated: datetime,
        };

        let user2 = User {
            user_id: user1.user_id, // Тот же UUID
            email: "test@example.com".to_string(),
            password_hash: "hash2".to_string(), // РАЗНЫЙ хэш
            role: UserRole::Admin,
            info: UserInfo::default(),
            created: datetime,
            updated: datetime,
        };

        // Два пользователя НЕ равны, потому что password_hash разный!
        // #[derive(PartialEq)] сравнивает ВСЕ поля
        assert_ne!(user1, user2); // Изменили с assert_eq! на assert_ne!

        // Проверяем, что Hash работает корректно
        // Разные password_hash -> разные хэши -> оба добавляются в HashSet
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(user1.clone());
        set.insert(user2.clone());
        assert_eq!(set.len(), 2); // ОБА добавляются, так как они разные!

        // Проверяем равенство при одинаковых ВСЕХ полях
        let user3 = User {
            user_id: user1.user_id,
            email: user1.email.clone(),
            password_hash: user1.password_hash.clone(), // Тот же хэш
            role: user1.role.clone(),
            info: user1.info.clone(),
            created: user1.created,
            updated: user1.updated,
        };

        assert_eq!(user1, user3); // Теперь они равны

        // Проверяем HashSet с одинаковыми пользователями
        let mut set2 = HashSet::new();
        set2.insert(user1.clone());
        set2.insert(user3);
        assert_eq!(set2.len(), 1); // Дубликат не добавляется
    }
}
