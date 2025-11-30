//! Logging utilities for the WebSocket server
//!
//! This module provides structured logging capabilities for the server.

#[cfg(feature = "logging")]
use tracing::{error, warn, info, debug, trace};

/// Log an error message
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            tracing::error!($($arg)*);
        }
        #[cfg(not(feature = "logging"))]
        {
            eprintln!("[ERROR] {}", format!($($arg)*));
        }
    };
}

/// Log a warning message
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            tracing::warn!($($arg)*);
        }
        #[cfg(not(feature = "logging"))]
        {
            eprintln!("[WARN] {}", format!($($arg)*));
        }
    };
}

/// Log an info message
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            tracing::info!($($arg)*);
        }
        #[cfg(not(feature = "logging"))]
        {
            eprintln!("[INFO] {}", format!($($arg)*));
        }
    };
}

/// Log a debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            tracing::debug!($($arg)*);
        }
        #[cfg(not(feature = "logging"))]
        {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

/// Log a trace message
#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        {
            tracing::trace!($($arg)*);
        }
        #[cfg(not(feature = "logging"))]
        {
            eprintln!("[TRACE] {}", format!($($arg)*));
        }
    };
}

/// Initialize logging subsystem
#[cfg(feature = "logging")]
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

/// Initialize logging subsystem (no-op when logging feature is disabled)
#[cfg(not(feature = "logging"))]
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_macros() {
        log_info!("Test info message");
        log_warn!("Test warning message");
        log_error!("Test error message");
        log_debug!("Test debug message");
    }

    #[test]
    fn test_init_logging() {
        assert!(init_logging().is_ok());
    }
}
