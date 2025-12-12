//! Telemetry and tracing setup

mod tracing_setup;

pub use tracing_setup::{
    init_tracing, init_tracing_with_config, try_init_tracing, try_init_tracing_with_config,
    TracingConfig, TracingError,
};
