//! Core time primitives for Apparatus.
//!
//! Provides a testable Clock trait and deterministic implementations.

use std::time::{SystemTime, UNIX_EPOCH};

/// A trait providing the current time in Unix milliseconds (UTC).
pub trait Clock {
    /// Returns the current Unix timestamp in milliseconds.
    fn now_ms(&self) -> u64;
}

/// A clock using the real system time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis() as u64
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
    pub fn advance_ms(&self, ms: u64) {
        self.current_ms
            .fetch_add(ms, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Clock for DeterministicClock {
    fn now_ms(&self) -> u64 {
        self.current_ms.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_clock() {
        let clock = SystemClock;
        let t1 = clock.now_ms();
        assert!(t1 > 0);
    }

    #[test]
    fn test_deterministic_clock() {
        let clock = DeterministicClock::new(1000);
        assert_eq!(clock.now_ms(), 1000);
        clock.advance_ms(500);
        assert_eq!(clock.now_ms(), 1500);
    }
}
