use crate::AppResult;
use crate::settings::DatabaseSettings;
use sqlx::Connection;
use sqlx::postgres::PgPoolOptions;
use tracing::instrument;

/// Хранилище данных на основе PostgreSQL
///
/// Обеспечивает подключение и работу с базой данных PostgreSQL через пул соединений.
/// Использует `sqlx` для асинхронного доступа к базе данных.
#[derive(Clone)]
pub struct PgStorage {
    /// Пул соединений с базой данных PostgreSQL
    ///
    /// Позволяет эффективно управлять несколькими подключениями к базе данных.
    pub(crate) pool: sqlx::PgPool,
}

impl PgStorage {
    /// Инициализирует хранилище PostgreSQL с использованием настроек базы данных
    ///
    /// Создает пул соединений с указанными настройками, устанавливает максимальное
    /// количество соединений (8) и проверяет подключение к базе данных через ping.
    ///
    /// # Аргументы
    ///
    /// * `settings` - Настройки подключения к базе данных
    ///
    /// # Возвращает
    ///
    /// * `Ok(PgStorage)` - если подключение успешно установлено
    /// * `Err(AppError)` - если произошла ошибка при подключении
    ///
    /// # Ошибки
    ///
    /// * `AppError::DatabaseConnectionError` - если не удалось установить соединение
    /// * `AppError::DatabasePingError` - если ping к базе данных не прошел
    ///
    /// # Логирование
    ///
    /// Функция логирует следующие события:
    /// * `DEBUG` - успешный ping к базе данных
    /// * `INFO` - успешная инициализация хранилища с деталями подключения
    #[instrument(name = "initializing pg repository", skip(settings))]
    pub async fn init(settings: DatabaseSettings) -> AppResult<Self> {
        let db_url = settings.db_url();
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(db_url.as_ref())
            .await?;
        let mut conn = pool.acquire().await?;
        conn.ping().await?;
        tracing::debug!("Ping to db successfully");
        tracing::info!(
            "Postgres repository initialized on 'postgres://{host}:{port}/{db}'",
            host = settings.host,
            port = settings.port,
            db = settings.database
        );
        conn.close().await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self { pool })
    }
    /// Закрывает пул соединений с базой данных
    ///
    /// Ожидает завершения всех активных операций и освобождает ресурсы.
    /// После вызова этого метода использование хранилища невозможно.
    ///
    /// # Примечание
    ///
    /// Метод потребляет `self` по значению, что гарантирует,
    /// что после закрытия пула объект больше не может быть использован.
    #[instrument(name = "closing pg pool", skip(self))]
    pub async fn close(self) {
        self.pool.close().await;
    }
    #[cfg(test)]
    pub(crate) fn with_pool(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }
}
