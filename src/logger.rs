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
