//! Tracing and logging setup
//!
//! Configures the `tracing` subscriber with environment-based filtering.

use tracing::Level;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Tracing configuration options
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Log level filter (e.g., "info", "debug", "trace")
    pub level: Level,
    /// Enable JSON output format
    pub json: bool,
    /// Include span events (new, close)
    pub span_events: bool,
    /// Include file and line numbers
    pub file_line: bool,
    /// Include thread names
    pub thread_names: bool,
    /// Include thread IDs
    pub thread_ids: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            json: false,
            span_events: false,
            file_line: true,
            thread_names: false,
            thread_ids: false,
        }
    }
}

impl TracingConfig {
    /// Create a development configuration with debug logging
    #[must_use]
    pub fn development() -> Self {
        Self {
            level: Level::DEBUG,
            json: false,
            span_events: true,
            file_line: true,
            thread_names: true,
            thread_ids: false,
        }
    }

    /// Create a production configuration with JSON logging
    #[must_use]
    pub fn production() -> Self {
        Self {
            level: Level::INFO,
            json: true,
            span_events: false,
            file_line: false,
            thread_names: false,
            thread_ids: false,
        }
    }
}

/// Initialize the tracing subscriber with default configuration
///
/// Uses `RUST_LOG` environment variable for filtering if set,
/// otherwise defaults to "info" level.
///
/// # Panics
/// Panics if the subscriber cannot be initialized (usually means it's already set).
pub fn init_tracing() {
    init_tracing_with_config(TracingConfig::default());
}

/// Initialize the tracing subscriber with custom configuration
///
/// # Panics
/// Panics if the subscriber cannot be initialized (usually means it's already set).
pub fn init_tracing_with_config(config: TracingConfig) {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.level.to_string()));

    if config.json {
        // JSON format for production/structured logging
        let fmt_layer = fmt::layer()
            .json()
            .with_file(config.file_line)
            .with_line_number(config.file_line)
            .with_thread_names(config.thread_names)
            .with_thread_ids(config.thread_ids)
            .with_span_events(if config.span_events {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            });

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    } else {
        // Pretty format for development
        let fmt_layer = fmt::layer()
            .with_file(config.file_line)
            .with_line_number(config.file_line)
            .with_thread_names(config.thread_names)
            .with_thread_ids(config.thread_ids)
            .with_span_events(if config.span_events {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            });

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    }
}

/// Try to initialize tracing, returning Ok if successful or already initialized
///
/// Unlike `init_tracing`, this function will not panic if called multiple times.
pub fn try_init_tracing() -> Result<(), TracingError> {
    try_init_tracing_with_config(TracingConfig::default())
}

/// Try to initialize tracing with custom configuration
///
/// Unlike `init_tracing_with_config`, this function will not panic if called multiple times.
pub fn try_init_tracing_with_config(config: TracingConfig) -> Result<(), TracingError> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.level.to_string()));

    if config.json {
        let fmt_layer = fmt::layer()
            .json()
            .with_file(config.file_line)
            .with_line_number(config.file_line)
            .with_thread_names(config.thread_names)
            .with_thread_ids(config.thread_ids)
            .with_span_events(if config.span_events {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            });

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|_| TracingError::AlreadyInitialized)
    } else {
        let fmt_layer = fmt::layer()
            .with_file(config.file_line)
            .with_line_number(config.file_line)
            .with_thread_names(config.thread_names)
            .with_thread_ids(config.thread_ids)
            .with_span_events(if config.span_events {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            });

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init()
            .map_err(|_| TracingError::AlreadyInitialized)
    }
}

/// Tracing initialization errors
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Tracing subscriber already initialized")]
    AlreadyInitialized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TracingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert!(!config.json);
        assert!(!config.span_events);
        assert!(config.file_line);
    }

    #[test]
    fn test_development_config() {
        let config = TracingConfig::development();
        assert_eq!(config.level, Level::DEBUG);
        assert!(!config.json);
        assert!(config.span_events);
        assert!(config.file_line);
        assert!(config.thread_names);
    }

    #[test]
    fn test_production_config() {
        let config = TracingConfig::production();
        assert_eq!(config.level, Level::INFO);
        assert!(config.json);
        assert!(!config.span_events);
        assert!(!config.file_line);
    }

    // Note: We can't easily test init_tracing in unit tests because
    // the global subscriber can only be set once per process.
    // Integration tests would need to run in separate processes.
}
