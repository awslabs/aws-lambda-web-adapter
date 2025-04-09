// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

pub(crate) struct Checkpoint {
    start: Instant,
    interval_ms: u128,
    next_ms: u128,
}

impl Checkpoint {
    pub fn new() -> Checkpoint {
        // Read interval from environment variable with a reasonable default
        // Allow configurable health check intervals
        // The default function timeout is 3 seconds. This will alert the users. See #520

        let interval_ms = std::env::var("AWS_LWA_HEALTH_CHECK_INTERVAL_MS")
            .unwrap_or_else(|_| "2000".to_string())
            .parse()
            .unwrap_or(2000);

        let start = Instant::now();
        Checkpoint {
            start,
            interval_ms,
            next_ms: start.elapsed().as_millis() + interval_ms,
        }
    }

    pub const fn next_ms(&self) -> u128 {
        self.next_ms
    }

    pub const fn increment(&mut self) {
        self.next_ms += self.interval_ms;
    }

    pub fn lapsed(&self) -> bool {
        self.start.elapsed().as_millis() >= self.next_ms
    }
}

// Add a backoff helper for health checks
#[derive(Clone, Copy)]
pub struct HealthCheckBackoff {
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    current_backoff_ms: u64,
    max_retries: Option<u32>,
    retry_count: u32,
}

impl HealthCheckBackoff {
    pub fn new() -> Self {
        // Read configuration from environment variables with reasonable defaults
        let initial_backoff_ms = std::env::var("AWS_LWA_HEALTH_CHECK_BACKOFF_INITIAL_MS")
            .unwrap_or_else(|_| "50".to_string()) // Slower polling by default (50ms instead of 10ms)
            .parse()
            .unwrap_or(50);
            
        let max_backoff_ms = std::env::var("AWS_LWA_HEALTH_CHECK_BACKOFF_MAX_MS")
            .unwrap_or_else(|_| "1000".to_string()) // Cap at 1 second
            .parse()
            .unwrap_or(1000);
            
        let max_retries = std::env::var("AWS_LWA_HEALTH_CHECK_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse().ok());

        Self {
            initial_backoff_ms,
            max_backoff_ms,
            current_backoff_ms: initial_backoff_ms,
            max_retries,
            retry_count: 0,
        }
    }

    // Get the current backoff duration
    pub fn current_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.current_backoff_ms)
    }

    // Apply exponential backoff with jitter and return the next delay
    pub fn next_backoff(&mut self) -> Option<std::time::Duration> {
        self.retry_count += 1;
        
        // Check if we've exceeded max retries
        if let Some(max) = self.max_retries {
            if self.retry_count > max {
                return None;
            }
        }
        
        // Apply exponential backoff with 10% jitter
        let base_backoff = std::cmp::min(
            self.max_backoff_ms,
            self.current_backoff_ms.saturating_mul(2)
        );
        
        // Add jitter to prevent thundering herd issues
        let jitter = (base_backoff as f64 * 0.1) as u64;
        let jitter_range = if jitter > 0 { 
            fastrand::u64(0..jitter) 
        } else { 
            0 
        };
        
        self.current_backoff_ms = base_backoff.saturating_sub(jitter_range);
        
        Some(std::time::Duration::from_millis(self.current_backoff_ms))
    }
    
    // Reset backoff to initial state
    pub fn reset(&mut self) {
        self.current_backoff_ms = self.initial_backoff_ms;
        self.retry_count = 0;
    }
}

// Helper function to create a retry strategy with exponential backoff
pub(crate) fn create_backoff_strategy() -> impl tokio_retry::Strategy {
    let backoff_ms = std::env::var("AWS_LWA_HEALTH_CHECK_BACKOFF_MS")
        .unwrap_or_else(|_| "50".to_string()) // Slower polling by default (50ms instead of 10ms)
        .parse()
        .unwrap_or(50);
        
    tracing::debug!("Health check configured with backoff of {}ms", backoff_ms);
    
    tokio_retry::strategy::ExponentialBackoff::from_millis(backoff_ms)
        .factor(1.5) // Increase by 50% each time
        .max_delay(std::time::Duration::from_secs(2)) // Cap at 2 seconds
        .map(|delay| {
            // Add jitter to avoid thundering herd issues
            let jitter = (delay.as_millis() as f64 * 0.1) as u64;
            std::time::Duration::from_millis(
                delay.as_millis() as u64 - fastrand::u64(0..jitter.max(1))
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkpoint_new() {
        // Test with default value
        std::env::remove_var("AWS_LWA_HEALTH_CHECK_INTERVAL_MS");
        let checkpoint = Checkpoint::new();
        assert_eq!(checkpoint.interval_ms, 2000);
        assert!(!checkpoint.lapsed());
        
        // Test with custom value
        std::env::set_var("AWS_LWA_HEALTH_CHECK_INTERVAL_MS", "5000");
        let checkpoint = Checkpoint::new();
        assert_eq!(checkpoint.interval_ms, 5000);
        assert!(!checkpoint.lapsed());
        
        // Reset
        std::env::remove_var("AWS_LWA_HEALTH_CHECK_INTERVAL_MS");
    }

    #[test]
    fn test_checkpoint_increment() {
        let mut checkpoint = Checkpoint::new();
        let initial_next = checkpoint.next_ms;
        checkpoint.increment();
        assert_eq!(checkpoint.next_ms(), initial_next + checkpoint.interval_ms);
        assert!(!checkpoint.lapsed());
    }

    #[test]
    fn test_checkpoint_lapsed() {
        let checkpoint = Checkpoint {
            start: Instant::now(),
            interval_ms: 0,
            next_ms: 0,
        };
        assert!(checkpoint.lapsed());
    }
    
    #[test]
    fn test_health_check_backoff() {
        let mut backoff = HealthCheckBackoff::new();
        
        // Initial backoff should be 50ms by default
        assert_eq!(backoff.current_delay().as_millis(), 50);
        
        // First backoff should roughly double (with jitter)
        let next_delay1 = backoff.next_backoff().unwrap();
        assert!(next_delay1.as_millis() >= 90 && next_delay1.as_millis() <= 100);
        
        // Second backoff should roughly double again
        let next_delay2 = backoff.next_backoff().unwrap();
        assert!(next_delay2.as_millis() >= 180 && next_delay2.as_millis() <= 200);
        
        // Reset should go back to initial
        backoff.reset();
        assert_eq!(backoff.current_delay().as_millis(), 50);
        
        // Test max backoff
        let mut max_backoff = HealthCheckBackoff {
            initial_backoff_ms: 500,
            max_backoff_ms: 1000,
            current_backoff_ms: 500,
            max_retries: None,
            retry_count: 0,
        };
        
        // First backoff should be under max
        let next_delay = max_backoff.next_backoff().unwrap();
        assert!(next_delay.as_millis() >= 900 && next_delay.as_millis() <= 1000);
        
        // Second backoff should hit max
        let next_delay = max_backoff.next_backoff().unwrap();
        assert!(next_delay.as_millis() >= 900 && next_delay.as_millis() <= 1000);
        
        // Test max retries
        let mut limited_backoff = HealthCheckBackoff {
            initial_backoff_ms: 50,
            max_backoff_ms: 1000,
            current_backoff_ms: 50,
            max_retries: Some(2),
            retry_count: 0,
        };
        
        // First two backoffs should work
        assert!(limited_backoff.next_backoff().is_some());
        assert!(limited_backoff.next_backoff().is_some());
        
        // Third backoff should return None
        assert!(limited_backoff.next_backoff().is_none());
    }
    
    #[test]
    fn test_create_backoff_strategy() {
        // Test default strategy creation
        std::env::remove_var("AWS_LWA_HEALTH_CHECK_BACKOFF_MS");
        let strategy = create_backoff_strategy();
        
        // We can't directly test the generated delays, but we can make sure it runs
        let mut iter = strategy.iter();
        let first_delay = iter.next().unwrap();
        let second_delay = iter.next().unwrap();
        
        // Second delay should be longer than first
        assert!(second_delay > first_delay);
        
        // Test with custom configuration
        std::env::set_var("AWS_LWA_HEALTH_CHECK_BACKOFF_MS", "100");
        let strategy = create_backoff_strategy();
        
        // Clean up
        std::env::remove_var("AWS_LWA_HEALTH_CHECK_BACKOFF_MS");
    }
}