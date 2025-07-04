use tracing::Level;

/// Initialize tracing subscriber with the given log level.
/// In debug builds, include file and line number without timestamp.
/// In release builds, include timestamp and log level.
pub fn init_tracing(level: Level) {
    let builder = tracing_subscriber::fmt().with_max_level(level);
    #[cfg(debug_assertions)]
    let subscriber = builder
        .with_target(false)
        .with_file(true)
        .with_line_number(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .finish();
    #[cfg(not(debug_assertions))]
    let subscriber = builder
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");
}
