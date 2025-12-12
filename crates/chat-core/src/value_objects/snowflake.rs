//! Snowflake ID - Discord-compatible 64-bit unique identifier
//!
//! Structure:
//! - Bits 63-22: Timestamp (milliseconds since custom epoch)
//! - Bits 21-12: Worker ID (0-1023)
//! - Bits 11-0:  Sequence number (0-4095)

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Discord-compatible Snowflake ID (64-bit)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Snowflake(i64);

impl Snowflake {
    /// Custom epoch: 2024-01-01 00:00:00 UTC (milliseconds)
    pub const EPOCH: i64 = 1704067200000;

    /// Create a new Snowflake from a raw i64 value
    #[inline]
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    /// Get the inner i64 value
    #[inline]
    pub const fn into_inner(self) -> i64 {
        self.0
    }

    /// Check if the Snowflake is zero (uninitialized)
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    /// Extract timestamp (milliseconds since Unix epoch)
    #[inline]
    pub fn timestamp(&self) -> i64 {
        (self.0 >> 22) + Self::EPOCH
    }

    /// Extract worker ID (0-1023)
    #[inline]
    pub fn worker_id(&self) -> u16 {
        ((self.0 >> 12) & 0x3FF) as u16
    }

    /// Extract sequence number (0-4095)
    #[inline]
    pub fn sequence(&self) -> u16 {
        (self.0 & 0xFFF) as u16
    }

    /// Convert timestamp to DateTime<Utc>
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        use chrono::{TimeZone, Utc};
        Utc.timestamp_millis_opt(self.timestamp())
            .single()
            .unwrap_or_else(|| Utc.timestamp_millis_opt(0).unwrap())
    }

    /// Parse from string representation
    pub fn parse(s: &str) -> Result<Self, SnowflakeParseError> {
        s.parse::<i64>()
            .map(Snowflake)
            .map_err(|_| SnowflakeParseError::InvalidFormat)
    }
}

/// Error when parsing a Snowflake from string
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum SnowflakeParseError {
    #[error("invalid snowflake format")]
    InvalidFormat,
}

impl fmt::Display for Snowflake {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for Snowflake {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<Snowflake> for i64 {
    fn from(id: Snowflake) -> Self {
        id.0
    }
}

impl std::str::FromStr for Snowflake {
    type Err = SnowflakeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Snowflake::parse(s)
    }
}

// Serialize as string for JSON (JavaScript BigInt safety)
impl Serialize for Snowflake {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

// Deserialize from string or number
impl<'de> Deserialize<'de> for Snowflake {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};

        struct SnowflakeVisitor;

        impl<'de> Visitor<'de> for SnowflakeVisitor {
            type Value = Snowflake;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or integer representing a snowflake ID")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Snowflake, E>
            where
                E: de::Error,
            {
                Ok(Snowflake(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Snowflake, E>
            where
                E: de::Error,
            {
                Ok(Snowflake(value as i64))
            }

            fn visit_str<E>(self, value: &str) -> Result<Snowflake, E>
            where
                E: de::Error,
            {
                value
                    .parse::<i64>()
                    .map(Snowflake)
                    .map_err(|_| de::Error::custom("invalid snowflake string"))
            }
        }

        deserializer.deserialize_any(SnowflakeVisitor)
    }
}

/// Thread-safe Snowflake ID generator
///
/// Generates unique IDs at up to 4096 per millisecond per worker.
/// Uses lock-free atomic operations for high performance.
pub struct SnowflakeGenerator {
    worker_id: u16,
    sequence: AtomicI64,
    last_timestamp: AtomicI64,
}

impl SnowflakeGenerator {
    /// Create a new generator with the given worker ID
    ///
    /// # Panics
    /// Panics if worker_id >= 1024
    pub fn new(worker_id: u16) -> Self {
        assert!(worker_id < 1024, "Worker ID must be < 1024");
        Self {
            worker_id,
            sequence: AtomicI64::new(0),
            last_timestamp: AtomicI64::new(0),
        }
    }

    /// Generate a new unique Snowflake ID
    pub fn generate(&self) -> Snowflake {
        loop {
            let mut timestamp = self.current_timestamp();
            let last = self.last_timestamp.load(Ordering::Acquire);

            if timestamp < last {
                // Clock moved backwards, wait for it to catch up
                std::thread::sleep(std::time::Duration::from_millis((last - timestamp) as u64));
                timestamp = self.current_timestamp();
            }

            let sequence = if timestamp == last {
                let seq = self.sequence.fetch_add(1, Ordering::Relaxed) & 0xFFF;
                if seq == 0 {
                    // Sequence overflow, wait for next millisecond
                    while self.current_timestamp() <= last {
                        std::hint::spin_loop();
                    }
                    timestamp = self.current_timestamp();
                    self.sequence.store(1, Ordering::Relaxed);
                    0
                } else {
                    seq
                }
            } else {
                self.sequence.store(1, Ordering::Relaxed);
                0
            };

            // Try to update last_timestamp atomically
            match self.last_timestamp.compare_exchange(
                last,
                timestamp,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    let id = ((timestamp - Snowflake::EPOCH) << 22)
                        | ((self.worker_id as i64) << 12)
                        | sequence;
                    return Snowflake::new(id);
                }
                Err(_) => {
                    // Another thread updated timestamp, retry
                    continue;
                }
            }
        }
    }

    /// Get current timestamp in milliseconds since Unix epoch
    #[inline]
    fn current_timestamp(&self) -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    /// Get the worker ID of this generator
    pub fn worker_id(&self) -> u16 {
        self.worker_id
    }
}

impl Default for SnowflakeGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

// SnowflakeGenerator is automatically Send + Sync because:
// - u16 is Send + Sync
// - AtomicI64 is Send + Sync

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_snowflake_creation() {
        let sf = Snowflake::new(123456789);
        assert_eq!(sf.into_inner(), 123456789);
    }

    #[test]
    fn test_snowflake_zero() {
        let sf = Snowflake::default();
        assert!(sf.is_zero());

        let sf = Snowflake::new(1);
        assert!(!sf.is_zero());
    }

    #[test]
    fn test_snowflake_parse() {
        let sf = Snowflake::parse("123456789").unwrap();
        assert_eq!(sf.into_inner(), 123456789);

        assert!(Snowflake::parse("invalid").is_err());
    }

    #[test]
    fn test_snowflake_display() {
        let sf = Snowflake::new(123456789);
        assert_eq!(sf.to_string(), "123456789");
    }

    #[test]
    fn test_snowflake_serialize_json() {
        let sf = Snowflake::new(123456789012345678);
        let json = serde_json::to_string(&sf).unwrap();
        assert_eq!(json, "\"123456789012345678\"");
    }

    #[test]
    fn test_snowflake_deserialize_string() {
        let sf: Snowflake = serde_json::from_str("\"123456789012345678\"").unwrap();
        assert_eq!(sf.into_inner(), 123456789012345678);
    }

    #[test]
    fn test_snowflake_deserialize_number() {
        let sf: Snowflake = serde_json::from_str("12345").unwrap();
        assert_eq!(sf.into_inner(), 12345);
    }

    #[test]
    fn test_snowflake_ordering() {
        let sf1 = Snowflake::new(100);
        let sf2 = Snowflake::new(200);
        assert!(sf1 < sf2);
    }

    #[test]
    fn test_generator_creates_unique_ids() {
        let gen = SnowflakeGenerator::new(1);
        let mut ids = HashSet::new();

        for _ in 0..1000 {
            let id = gen.generate();
            assert!(ids.insert(id), "Duplicate ID generated");
        }
    }

    #[test]
    fn test_generator_ids_are_monotonic() {
        let gen = SnowflakeGenerator::new(1);
        let mut last = Snowflake::new(0);

        for _ in 0..1000 {
            let id = gen.generate();
            assert!(id > last, "IDs should be monotonically increasing");
            last = id;
        }
    }

    #[test]
    fn test_generator_worker_id_preserved() {
        let gen = SnowflakeGenerator::new(42);
        let id = gen.generate();
        assert_eq!(id.worker_id(), 42);
    }

    #[test]
    fn test_generator_thread_safety() {
        let gen = Arc::new(SnowflakeGenerator::new(1));
        let mut handles = vec![];
        let ids = Arc::new(std::sync::Mutex::new(HashSet::new()));

        for _ in 0..4 {
            let gen = Arc::clone(&gen);
            let ids = Arc::clone(&ids);

            handles.push(thread::spawn(move || {
                let mut local_ids = Vec::with_capacity(1000);
                for _ in 0..1000 {
                    local_ids.push(gen.generate());
                }
                ids.lock().unwrap().extend(local_ids);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(ids.lock().unwrap().len(), 4000, "All IDs should be unique");
    }

    #[test]
    #[should_panic(expected = "Worker ID must be < 1024")]
    fn test_generator_invalid_worker_id() {
        SnowflakeGenerator::new(1024);
    }

    #[test]
    fn test_snowflake_timestamp_extraction() {
        let gen = SnowflakeGenerator::new(1);
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let id = gen.generate();

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let timestamp = id.timestamp();
        assert!(
            timestamp >= before && timestamp <= after,
            "Timestamp should be within generation window"
        );
    }
}
