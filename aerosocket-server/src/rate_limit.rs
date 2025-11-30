//! Rate limiting and DoS protection for WebSocket server
//!
//! This module provides rate limiting capabilities to protect against DoS attacks.

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use aerosocket_core::Result;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window for rate limiting
    pub window: Duration,
    /// Maximum concurrent connections per IP
    pub max_connections: usize,
    /// Connection timeout duration
    pub connection_timeout: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60),
            max_connections: 10,
            connection_timeout: Duration::from_secs(300),
        }
    }
}

/// Rate limiter for tracking requests and connections
pub struct RateLimiter {
    config: RateLimitConfig,
    /// Request tracking per IP
    request_counters: Mutex<HashMap<IpAddr, RequestCounter>>,
    /// Connection tracking per IP
    connection_counters: Mutex<HashMap<IpAddr, usize>>,
}

/// Request counter for a specific IP
#[derive(Debug, Clone)]
struct RequestCounter {
    count: usize,
    window_start: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            request_counters: Mutex::new(HashMap::new()),
            connection_counters: Mutex::new(HashMap::new()),
        }
    }

    /// Check if an IP is allowed to make a request
    pub async fn check_request_rate(&self, ip: IpAddr) -> Result<bool> {
        let mut counters = self.request_counters.lock().await;
        let now = Instant::now();

        let counter = counters.entry(ip).or_insert_with(|| RequestCounter {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(counter.window_start) >= self.config.window {
            counter.count = 0;
            counter.window_start = now;
        }

        // Check rate limit
        if counter.count >= self.config.max_requests {
            return Ok(false);
        }

        counter.count += 1;
        Ok(true)
    }

    /// Check if an IP can establish a new connection
    pub async fn can_connect(&self, ip: IpAddr) -> Result<bool> {
        let mut conn_counters = self.connection_counters.lock().await;
        let current_count = conn_counters.entry(ip).or_insert(0);

        if *current_count >= self.config.max_connections {
            return Ok(false);
        }

        *current_count += 1;
        Ok(true)
    }

    /// Remove a connection for an IP
    pub async fn remove_connection(&self, ip: IpAddr) {
        let mut conn_counters = self.connection_counters.lock().await;
        if let Some(count) = conn_counters.get_mut(&ip) {
            if *count > 0 {
                *count -= 1;
            }
            if *count == 0 {
                conn_counters.remove(&ip);
            }
        }
    }

    /// Cleanup expired entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        
        // Cleanup expired request counters
        {
            let mut counters = self.request_counters.lock().await;
            counters.retain(|_, counter| {
                now.duration_since(counter.window_start) < self.config.window * 2
            });
        }

        // Cleanup connection counters (they don't expire naturally)
        // This is mainly for memory management
        {
            let mut conn_counters = self.connection_counters.lock().await;
            conn_counters.retain(|_, &mut count| count > 0);
        }
    }

    /// Get statistics about rate limiting
    pub async fn get_stats(&self) -> RateLimitStats {
        let request_count = self.request_counters.lock().await.len();
        let connection_count = self.connection_counters.lock().await.len();
        
        RateLimitStats {
            tracked_ips: request_count,
            active_connections: connection_count,
        }
    }
}

/// Rate limiting statistics
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    /// Number of IPs being tracked for requests
    pub tracked_ips: usize,
    /// Number of IPs with active connections
    pub active_connections: usize,
}

/// Middleware for rate limiting WebSocket connections
pub struct RateLimitMiddleware {
    limiter: RateLimiter,
}

impl RateLimitMiddleware {
    /// Create new rate limit middleware
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: RateLimiter::new(config),
        }
    }

    /// Check if a connection is allowed
    pub async fn check_connection(&self, ip: IpAddr) -> Result<bool> {
        // Check both request rate and connection limits
        let request_ok = self.limiter.check_request_rate(ip).await?;
        let connection_ok = self.limiter.can_connect(ip).await?;

        Ok(request_ok && connection_ok)
    }

    /// Remove a connection from tracking
    pub async fn connection_closed(&self, ip: IpAddr) {
        self.limiter.remove_connection(ip).await;
    }

    /// Get rate limiting statistics
    pub async fn stats(&self) -> RateLimitStats {
        self.limiter.get_stats().await
    }

    /// Cleanup expired entries
    pub async fn cleanup(&self) {
        self.limiter.cleanup().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[tokio::test]
    async fn test_rate_limiting() {
        let config = RateLimitConfig {
            max_requests: 2,
            window: Duration::from_secs(1),
            max_connections: 1,
            connection_timeout: Duration::from_secs(60),
        };

        let limiter = RateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First request should be allowed
        assert!(limiter.check_request_rate(ip).await.unwrap());
        
        // Second request should be allowed
        assert!(limiter.check_request_rate(ip).await.unwrap());
        
        // Third request should be denied
        assert!(!limiter.check_request_rate(ip).await.unwrap());

        // Wait for window to reset
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Should be allowed again
        assert!(limiter.check_request_rate(ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_connection_limiting() {
        let config = RateLimitConfig {
            max_requests: 100,
            window: Duration::from_secs(60),
            max_connections: 2,
            connection_timeout: Duration::from_secs(60),
        };

        let limiter = RateLimiter::new(config);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First connection should be allowed
        assert!(limiter.can_connect(ip).await.unwrap());
        
        // Second connection should be allowed
        assert!(limiter.can_connect(ip).await.unwrap());
        
        // Third connection should be denied
        assert!(!limiter.can_connect(ip).await.unwrap());

        // Remove one connection
        limiter.remove_connection(ip).await;
        
        // Should be allowed again
        assert!(limiter.can_connect(ip).await.unwrap());
    }

    #[tokio::test]
    async fn test_middleware() {
        let config = RateLimitConfig::default();
        let middleware = RateLimitMiddleware::new(config);
        let ip = IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1));

        // Should allow connection
        assert!(middleware.check_connection(ip).await.unwrap());
        
        // Get stats
        let stats = middleware.stats().await;
        assert_eq!(stats.tracked_ips, 1);
        assert_eq!(stats.active_connections, 1);
        
        // Close connection
        middleware.connection_closed(ip).await;
        
        let stats = middleware.stats().await;
        assert_eq!(stats.active_connections, 0);
    }
}
