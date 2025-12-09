use serde::Deserialize;

pub fn init() -> Settings {
    config::Config::builder()
        .add_source(config::File::with_name("settings.toml"))
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
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
}
