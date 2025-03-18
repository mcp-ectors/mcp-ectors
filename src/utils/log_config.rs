

use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Configuration for dynamic logging.
#[derive(Clone)]
pub struct LogConfig {
    pub log_dir: String,
    pub log_file: String,
    pub level: Level,
}

/// Initializes a tracing subscriber that logs to both stdout and a rolling file.
pub fn init_logging(config: &LogConfig) {
    // Create a rolling file appender that rotates daily.
    let file_appender = RollingFileAppender::new(Rotation::DAILY, &config.log_dir, &config.log_file);
    
    // Create a layer that writes logs to stdout.
    let stdout_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(false)       // Disable target module info if desired.
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Create a layer that writes logs to the file.
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    // Build an EnvFilter from the default environment plus our log level.
    let env_filter = EnvFilter::from_default_env().add_directive(config.level.into());

    // Build the subscriber using the Registry and attach both layers.
    let subscriber = Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer);

    // Set this subscriber as the global default.
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up global tracing subscriber");
}
