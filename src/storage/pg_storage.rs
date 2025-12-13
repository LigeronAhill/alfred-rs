use crate::AppResult;
use sqlx::{Connection, Pool, Postgres};
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
    #[instrument(name = "initializing pg repository", skip(pool))]
    pub async fn init(pool: Pool<Postgres>) -> AppResult<Self> {
        let mut conn = pool.acquire().await?;
        conn.ping().await?;
        tracing::debug!("Ping to db successfully");
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
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    use crate::AppResult;

    #[sqlx::test]
    async fn test_init(pool: PgPool) -> AppResult<()> {
        let pg_storage = PgStorage::init(pool).await;
        assert!(pg_storage.is_ok());
        pg_storage.unwrap().close().await;
        Ok(())
    }
}
