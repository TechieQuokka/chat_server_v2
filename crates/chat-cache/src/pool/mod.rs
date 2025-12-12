//! Redis connection pool module.
//!
//! Provides connection pooling for Redis using deadpool-redis.

mod redis_pool;

pub use redis_pool::{
    create_shared_pool, RedisPool, RedisPoolConfig, RedisPoolError, RedisResult, SharedRedisPool,
};
