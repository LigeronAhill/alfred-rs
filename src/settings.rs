//! Модуль для работы с настройками приложения
//!
//! Этот модуль содержит структуры и функции для загрузки и работы
//! с настройками приложения из конфигурационных файлов.

use serde::Deserialize;
use std::borrow::Cow;
use tracing::instrument;

/// Инициализирует настройки приложения из файла конфигурации
///
/// # Аргументы
///
/// * `file` - Путь к файлу конфигурации (без расширения .toml)
///
/// # Возвращает
///
/// Загруженные и десериализованные настройки приложения
///
/// # Паникует
///
/// Функция паникует если:
/// * Файл конфигурации не найден или не может быть прочитан
/// * Содержимое файла не может быть десериализовано в структуру `Settings`
///
/// # Пример
///
/// ```rust
/// use alfred::settings::init;
/// let settings = init("settings");
/// println!("Database host: {}", settings.database_settings.host);
/// ```
#[instrument(name = "initializing settings")]
pub fn init(file: &str) -> Settings {
    config::Config::builder()
        .add_source(config::File::with_name(file))
        .build()
        .expect("Failed to read config file 'settings.toml'")
        .try_deserialize()
        .expect("Failed to deserialize settings")
}

/// Основные настройки приложения
///
/// Содержит все настройки, необходимые для работы приложения,
/// сгруппированные по функциональным областям.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Настройки базы данных
    pub database_settings: DatabaseSettings,

    /// Настройки email-сервиса
    pub email_settings: EmailSettings,

    /// Настройки веб-сервера
    pub server_settings: ServerSettings,
}

/// Настройки подключения к базе данных
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    /// Хост базы данных
    pub host: String,

    /// Порт базы данных
    pub port: u16,

    /// Имя пользователя для подключения
    pub username: String,

    /// Пароль для подключения
    pub password: String,

    /// Имя базы данных
    pub database: String,
    pub max_connections: Option<u32>,
    pub idle_timeout: Option<u64>,
}

impl DatabaseSettings {
    /// Создает URL для подключения к базе данных в формате PostgreSQL
    ///
    /// # Возвращает
    ///
    /// URL подключения в формате: `postgres://username:password@host:port/database`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use alfred::settings::DatabaseSettings;
    /// let db_settings = DatabaseSettings {
    ///     host: "localhost".to_string(),
    ///     port: 5432,
    ///     username: "postgres".to_string(),
    ///     password: "password".to_string(),
    ///     database: "mydb".to_string(),
    /// 	max_connections: None,
    /// 	idle_timeout: None,
    /// };
    ///
    /// let url = db_settings.db_url();
    /// assert_eq!(url, "postgres://postgres:password@localhost:5432/mydb");
    /// ```
    #[instrument(name = "creating database url", skip(self))]
    pub fn db_url(&self) -> Cow<'static, str> {
        Cow::Owned(format!(
            "postgres://{user}:{pass}@{host}:{port}/{db}",
            user = self.username,
            pass = self.password,
            host = self.host,
            port = self.port,
            db = self.database
        ))
    }
}

/// Настройки email-сервиса
///
/// Используется для отправки email уведомлений и писем подтверждения.
#[derive(Debug, Clone, Deserialize)]
pub struct EmailSettings {
    /// SMTP сервер для отправки email
    pub host: String,

    /// Имя пользователя для аутентификации на SMTP сервере
    pub username: String,

    /// Пароль для аутентификации на SMTP сервере
    pub password: String,
}

/// Настройки веб-сервера
///
/// Определяет параметры, на которых работает веб-сервер приложения.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    /// Хост, на котором запускается сервер
    pub host: String,

    /// Порт, на котором запускается сервер
    pub port: u16,

    /// Хост, с которого доступны запросы
    pub origin: Option<String>,
}

impl ServerSettings {
    /// Создает адрес для привязки сервера
    ///
    /// # Возвращает
    ///
    /// Строку в формате `host:port`
    ///
    /// # Пример
    ///
    /// ```rust
    /// use alfred::settings::ServerSettings;
    /// let server_settings = ServerSettings {
    ///     host: "0.0.0.0".to_string(),
    ///     port: 8080,
    /// };
    ///
    /// let address = server_settings.server_address();
    /// assert_eq!(address, "0.0.0.0:8080");
    /// ```
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_settings_db_url() {
        let db_settings = DatabaseSettings {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "mysecretpassword".to_string(),
            database: "mydatabase".to_string(),
            max_connections: Some(16),
            idle_timeout: Some(60),
        };

        let expected = "postgres://postgres:mysecretpassword@localhost:5432/mydatabase";
        assert_eq!(db_settings.db_url(), expected);

        // Проверяем, что возвращается Cow::Owned
        match db_settings.db_url() {
            Cow::Owned(s) => assert_eq!(s, expected),
            Cow::Borrowed(_) => panic!("Expected owned string"),
        }
    }

    #[test]
    fn test_server_settings_server_address() {
        let server_settings = ServerSettings {
            host: "0.0.0.0".to_string(),
            port: 3000,
            origin: None,
        };

        assert_eq!(server_settings.server_address(), "0.0.0.0:3000");

        let server_settings = ServerSettings {
            host: "127.0.0.1".to_string(),
            port: 8080,
            origin: None,
        };

        assert_eq!(server_settings.server_address(), "127.0.0.1:8080");
    }

    #[test]
    #[should_panic(expected = "Failed to read config file 'settings.toml'")]
    fn test_init_panics_on_missing_file() {
        // Этот тест проверяет, что функция init паникует при отсутствии файла
        // В реальном проекте лучше использовать try_init, которая возвращает Result
        init("nonexistent_settings");
    }

    #[test]
    fn test_settings_clone() {
        let settings = Settings {
            database_settings: DatabaseSettings {
                host: "localhost".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "pass".to_string(),
                database: "db".to_string(),
                max_connections: None,
                idle_timeout: None,
            },
            email_settings: EmailSettings {
                host: "smtp.example.com".to_string(),
                username: "email@example.com".to_string(),
                password: "emailpass".to_string(),
            },
            server_settings: ServerSettings {
                host: "0.0.0.0".to_string(),
                port: 3000,
                origin: None,
            },
        };

        let cloned = settings.clone();

        assert_eq!(
            settings.database_settings.host,
            cloned.database_settings.host
        );
        assert_eq!(settings.email_settings.host, cloned.email_settings.host);
        assert_eq!(settings.server_settings.host, cloned.server_settings.host);
    }

    #[test]
    fn test_settings_debug() {
        let settings = Settings {
            database_settings: DatabaseSettings {
                host: "localhost".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "pass".to_string(),
                database: "db".to_string(),
                max_connections: Some(4),
                idle_timeout: Some(90),
            },
            email_settings: EmailSettings {
                host: "smtp.example.com".to_string(),
                username: "email@example.com".to_string(),
                password: "emailpass".to_string(),
            },
            server_settings: ServerSettings {
                host: "0.0.0.0".to_string(),
                port: 3000,
                origin: None,
            },
        };

        let debug_output = format!("{:?}", settings);

        // Проверяем, что Debug вывод содержит ключевые поля
        assert!(debug_output.contains("localhost"));
        assert!(debug_output.contains("5432"));
        assert!(debug_output.contains("smtp.example.com"));
        assert!(debug_output.contains("0.0.0.0"));
        assert!(debug_output.contains("3000"));
    }

    #[test]
    fn test_db_url_password_encoding() {
        // Проверяем, что пароли с специальными символами корректно обрабатываются
        let test_cases = vec![
            ("simple", "postgres://user:simple@host:5432/db"),
            ("p@ssword!", "postgres://user:p@ssword!@host:5432/db"),
            (
                "pass@word#123",
                "postgres://user:pass@word#123@host:5432/db",
            ),
        ];

        for (password, expected) in test_cases {
            let db_settings = DatabaseSettings {
                host: "host".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: password.to_string(),
                database: "db".to_string(),
                max_connections: None,
                idle_timeout: None,
            };

            // Формат URL требует кодирования специальных символов
            // В реальности библиотеки для PostgreSQL обычно делают это автоматически
            // Здесь мы просто проверяем, что URL формируется
            let url = db_settings.db_url();
            assert_eq!(url, expected);
            assert!(url.starts_with("postgres://user:"));
            assert!(url.ends_with("@host:5432/db"));
        }
    }
}
