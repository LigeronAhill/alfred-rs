/// Инициализирует глобальный журнал с заданным уровнем.
///
/// Настраивает журнал с помощью `tracing_subscriber::fmt`, который будет:
/// - отображать только сообщения с уровнем не ниже указанного `level`
/// - включать информацию о файле и номере строки
/// - исключать отображение цели (target)
///
/// # Аргументы
///
/// * `level` - максимальный уровень логирования (от `DEBUG` до `ERROR`)
///
/// # Паника
///
/// Функция паникует, если не удается установить глобальный подписчик.
pub fn init(level: tracing::Level) {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to initialize logger");
    tracing::debug!("DEBUG messages allowed");
    tracing::info!("INFO messages allowed");
    tracing::warn!("WARN messages allowed");
    tracing::error!("ERROR messages allowed");
    tracing::info!("logger initialized with level: '{level}");
}
