//! Core time primitives for Apparatus.
//!
//! Provides a testable Clock trait and deterministic implementations.

use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Time-related errors.
#[derive(Debug, Error)]
pub enum TimeError {
    /// System time went backwards or could not be determined.
    #[error("System time error: {0}")]
    SystemTimeError(String),
}

/// A named type representing Unix milliseconds (UTC).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct UnixMs(u64);

impl UnixMs {
    /// Create a new UnixMs value from raw u64 milliseconds.
    pub fn new(ms: u64) -> Self {
        Self(ms)
    }

    /// Get the underlying u64 milliseconds.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// A trait providing the current time in Unix milliseconds (UTC).
pub trait Clock {
    /// Returns the current Unix timestamp in milliseconds.
    fn now_ms(&self) -> Result<UnixMs, TimeError>;
}

/// A clock using the real system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> Result<UnixMs, TimeError> {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| TimeError::SystemTimeError(e.to_string()))?;

        let ms = u64::try_from(duration.as_millis())
            .map_err(|e| TimeError::SystemTimeError(format!("Milliseconds overflow u64: {}", e)))?;

        Ok(UnixMs(ms))
    }
}

/// A deterministic clock for testing.
#[derive(Debug, Clone)]
pub struct DeterministicClock {
    current_ms: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl DeterministicClock {
    /// Create a new deterministic clock starting at the given Unix ms.
    pub fn new(start_ms: u64) -> Self {
        Self {
            current_ms: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(start_ms)),
        }
    }

    /// Advance the clock by the specified number of milliseconds.
    /// Returns an error if advancing would overflow u64.
    pub fn advance_ms(&self, ms: u64) -> Result<(), TimeError> {
        let current = self.current_ms.load(std::sync::atomic::Ordering::SeqCst);
        let next = current.checked_add(ms).ok_or_else(|| {
            TimeError::SystemTimeError("Clock advanced beyond u64 capacity".to_string())
        })?;
        self.current_ms
            .store(next, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
}

impl Clock for DeterministicClock {
    fn now_ms(&self) -> Result<UnixMs, TimeError> {
        Ok(UnixMs(
            self.current_ms.load(std::sync::atomic::Ordering::SeqCst),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock() {
        let clock = SystemClock;
        let t1 = clock.now_ms().unwrap();
        assert!(t1.as_u64() > 0);
    }

    #[test]
    fn test_deterministic_clock() {
        let clock = DeterministicClock::new(1000);
        assert_eq!(clock.now_ms().unwrap().as_u64(), 1000);
        clock.advance_ms(500).unwrap();
        assert_eq!(clock.now_ms().unwrap().as_u64(), 1500);
    }

    #[test]
    fn test_deterministic_clock_overflow() {
        let clock = DeterministicClock::new(u64::MAX - 10);
        assert!(clock.advance_ms(20).is_err());
    }
}
