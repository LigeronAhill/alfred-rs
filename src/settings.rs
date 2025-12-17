//! Модуль для работы с настройками приложения
//!
//! Этот модуль содержит структуры и функции для загрузки и работы
//! с настройками приложения из конфигурационных файлов.

use serde::Deserialize;
use std::{borrow::Cow, sync::Arc};
use tracing::instrument;

#[instrument(name = "initializing settings")]
pub fn init(file: &str) -> Settings {
    config::Config::builder()
        .add_source(config::File::with_name(file))
        .build()
        .expect("Failed to read config file 'settings.toml'")
        .try_deserialize()
        .expect("Failed to deserialize settings")
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub database_settings: DatabaseSettings,
    pub email_settings: EmailSettings,
    pub server_settings: ServerSettings,
    pub jwt_settings: JWTSettings,
}
impl Settings {
    pub fn jwt(&self) -> Arc<JWTSettings> {
        Arc::new(self.jwt_settings.clone())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: Option<u32>,
    pub idle_timeout: Option<u64>,
}

impl DatabaseSettings {
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
#[derive(Debug, Clone, Deserialize)]
pub struct EmailSettings {
    pub host: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub port: u16,
    pub origin: String,
}

impl ServerSettings {
    pub fn server_address(&self) -> String {
        format!("{host}:{port}", host = self.host, port = self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JWTSettings {
    pub secret: String,
    pub expires_in: i64,
    pub maxage: i64,
}
