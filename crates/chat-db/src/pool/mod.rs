//! Database connection pool management

mod postgres;

pub use postgres::{create_pool, create_pool_from_env, DatabaseConfig};

// Re-export PgPool for convenience
pub use sqlx::postgres::PgPool;
