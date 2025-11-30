//! Connection manager for WebSocket server
//!
//! This module provides connection management, monitoring, and cleanup functionality.

use crate::config::ServerConfig;
use crate::connection::{Connection, ConnectionHandle};
use aerosocket_core::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::interval;

/// Connection manager statistics
#[derive(Debug, Clone)]
pub struct ManagerStats {
    /// Total number of active connections
    pub active_connections: usize,
    /// Total number of connections since server start
    pub total_connections: u64,
    /// Number of connections closed due to timeout
    pub timeout_closures: u64,
    /// Number of connections closed due to errors
    pub error_closures: u64,
    /// Number of connections closed normally
    pub normal_closures: u64,
    /// Current memory usage in bytes
    pub memory_usage: u64,
    /// Peak number of concurrent connections
    pub peak_connections: usize,
}

impl Default for ManagerStats {
    fn default() -> Self {
        Self {
            active_connections: 0,
            total_connections: 0,
            timeout_closures: 0,
            error_closures: 0,
            normal_closures: 0,
            memory_usage: 0,
            peak_connections: 0,
        }
    }
}

/// Connection manager
#[derive(Debug)]
pub struct ConnectionManager {
    /// Server configuration
    config: ServerConfig,
    /// Active connections by ID
    connections: Arc<Mutex<HashMap<u64, ConnectionHandle>>>,
    /// Connection statistics
    stats: Arc<Mutex<ManagerStats>>,
    /// Next connection ID
    next_id: Arc<Mutex<u64>>,
    /// Cleanup interval
    cleanup_interval: Duration,
    /// Sender for cleanup notifications
    cleanup_tx: mpsc::Sender<u64>,
    /// Receiver for cleanup notifications
    cleanup_rx: Arc<Mutex<mpsc::Receiver<u64>>>,
}

impl ConnectionManager {
    /// Create a new connection manager
    pub fn new(config: ServerConfig) -> Self {
        let (cleanup_tx, cleanup_rx) = mpsc::channel(1000);

        Self {
            cleanup_interval: Duration::from_secs(30), // Default cleanup interval
            config,
            connections: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(ManagerStats::default())),
            next_id: Arc::new(Mutex::new(1)),
            cleanup_tx,
            cleanup_rx: Arc::new(Mutex::new(cleanup_rx)),
        }
    }

    /// Set the cleanup interval
    pub fn set_cleanup_interval(&mut self, interval: Duration) {
        self.cleanup_interval = interval;
    }

    /// Add a new connection
    pub async fn add_connection(&self, connection: Connection) -> Result<ConnectionHandle> {
        let mut next_id = self.next_id.lock().await;
        let id = *next_id;
        *next_id += 1;

        let handle = ConnectionHandle::new(id, connection);

        let mut connections = self.connections.lock().await;
        connections.insert(id, handle.clone());

        // Update statistics
        let mut stats = self.stats.lock().await;
        stats.active_connections = connections.len();
        stats.total_connections += 1;
        stats.peak_connections = stats.peak_connections.max(stats.active_connections);

        Ok(handle)
    }

    /// Remove a connection
    pub async fn remove_connection(&self, id: u64, reason: CloseReason) {
        let mut connections = self.connections.lock().await;
        if connections.remove(&id).is_some() {
            // Update statistics
            let mut stats = self.stats.lock().await;
            stats.active_connections = connections.len();

            match reason {
                CloseReason::Timeout => stats.timeout_closures += 1,
                CloseReason::Error => stats.error_closures += 1,
                CloseReason::Normal => stats.normal_closures += 1,
            }
        }
    }

    /// Get connection by ID
    pub async fn get_connection(&self, id: u64) -> Option<ConnectionHandle> {
        let connections = self.connections.lock().await;
        connections.get(&id).cloned()
    }

    /// Get all active connections
    pub async fn get_all_connections(&self) -> Vec<ConnectionHandle> {
        let connections = self.connections.lock().await;
        connections.values().cloned().collect()
    }

    /// Get current connection count
    pub async fn connection_count(&self) -> usize {
        let connections = self.connections.lock().await;
        connections.len()
    }

    /// Get connection manager statistics
    pub async fn get_stats(&self) -> ManagerStats {
        let stats = self.stats.lock().await;
        ManagerStats {
            active_connections: stats.active_connections,
            total_connections: stats.total_connections,
            timeout_closures: stats.timeout_closures,
            error_closures: stats.error_closures,
            normal_closures: stats.normal_closures,
            memory_usage: stats.memory_usage,
            peak_connections: stats.peak_connections,
        }
    }

    /// Start the cleanup task
    pub async fn start_cleanup_task(&self) {
        let connections = self.connections.clone();
        let stats = self.stats.clone();
        let cleanup_rx = self.cleanup_rx.clone();
        let cleanup_interval = self.cleanup_interval;
        let idle_timeout = self.config.idle_timeout;

        tokio::spawn(async move {
            let mut cleanup_interval_timer = interval(cleanup_interval);
            let mut cleanup_receiver = cleanup_rx.lock().await;

            loop {
                tokio::select! {
                    _ = cleanup_interval_timer.tick() => {
                        // Periodic cleanup
                        Self::cleanup_idle_connections(&connections, &stats, idle_timeout).await;
                    }
                    Some(id) = cleanup_receiver.recv() => {
                        // Immediate cleanup for specific connection
                        Self::remove_connection_internal(&connections, &stats, id, CloseReason::Timeout).await;
                    }
                }
            }
        });
    }

    /// Cleanup idle connections
    async fn cleanup_idle_connections(
        connections: &Arc<Mutex<HashMap<u64, ConnectionHandle>>>,
        stats: &Arc<Mutex<ManagerStats>>,
        idle_timeout: Duration,
    ) {
        let mut connections_map = connections.lock().await;
        let mut to_remove = Vec::new();

        for (id, handle) in connections_map.iter() {
            if let Ok(connection) = handle.try_lock().await {
                if connection.is_timed_out() {
                    to_remove.push(*id);
                }
            }
        }

        for id in to_remove {
            connections_map.remove(&id);
            let mut stats = stats.lock().await;
            stats.active_connections = connections_map.len();
            stats.timeout_closures += 1;
        }
    }

    /// Internal connection removal
    async fn remove_connection_internal(
        connections: &Arc<Mutex<HashMap<u64, ConnectionHandle>>>,
        stats: &Arc<Mutex<ManagerStats>>,
        id: u64,
        reason: CloseReason,
    ) {
        let mut connections_map = connections.lock().await;
        if connections_map.remove(&id).is_some() {
            let mut stats = stats.lock().await;
            stats.active_connections = connections_map.len();

            match reason {
                CloseReason::Timeout => stats.timeout_closures += 1,
                CloseReason::Error => stats.error_closures += 1,
                CloseReason::Normal => stats.normal_closures += 1,
            }
        }
    }

    /// Monitor connection health
    pub async fn monitor_connections(&self) -> Result<Vec<ConnectionHealth>> {
        let connections = self.connections.lock().await;
        let mut health_reports = Vec::new();

        for (id, handle) in connections.iter() {
            if let Ok(connection) = handle.try_lock().await {
                let health = ConnectionHealth {
                    id: *id,
                    remote_addr: connection.remote_addr(),
                    state: connection.state(),
                    uptime: connection.metadata().established_at.elapsed(),
                    last_activity: connection.metadata().last_activity_at.elapsed(),
                    messages_sent: connection.metadata().messages_sent,
                    messages_received: connection.metadata().messages_received,
                    bytes_sent: connection.metadata().bytes_sent,
                    bytes_received: connection.metadata().bytes_received,
                    time_until_timeout: connection.time_until_timeout(),
                };
                health_reports.push(health);
            }
        }

        Ok(health_reports)
    }

    /// Close all connections
    pub async fn close_all_connections(&self) {
        let connections = self.connections.lock().await;
        let handles: Vec<_> = connections.values().cloned().collect();
        let connection_count = connections.len();
        drop(connections);

        for handle in handles {
            if let Ok(mut connection) = handle.try_lock().await {
                let _ = connection.close(Some(1000), Some("Server shutdown")).await;
            }
        }

        // Clear all connections and update stats
        let mut connections_map = self.connections.lock().await;
        connections_map.clear();

        // Update statistics
        let mut stats = self.stats.lock().await;
        stats.active_connections = 0;
        stats.normal_closures += connection_count as u64;
    }
}

impl Drop for ConnectionManager {
    fn drop(&mut self) {
        // Ensure all connections are closed when manager is dropped
        let connections = self.connections.clone();
        tokio::spawn(async move {
            let manager = ConnectionManager {
                config: ServerConfig::default(),
                connections,
                stats: Arc::new(Mutex::new(ManagerStats::default())),
                next_id: Arc::new(Mutex::new(0)),
                cleanup_interval: Duration::ZERO,
                cleanup_tx: mpsc::channel(1).0,
                cleanup_rx: Arc::new(Mutex::new(mpsc::channel(1).1)),
            };
            manager.close_all_connections().await;
        });
    }
}

/// Connection close reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    /// Connection closed due to timeout
    Timeout,
    /// Connection closed due to error
    Error,
    /// Connection closed normally
    Normal,
}

/// Connection health information
#[derive(Debug, Clone)]
pub struct ConnectionHealth {
    /// Connection ID
    pub id: u64,
    /// Remote address
    pub remote_addr: std::net::SocketAddr,
    /// Connection state
    pub state: crate::connection::ConnectionState,
    /// How long the connection has been active
    pub uptime: Duration,
    /// Time since last activity
    pub last_activity: Duration,
    /// Number of messages sent
    pub messages_sent: u64,
    /// Number of messages received
    pub messages_received: u64,
    /// Number of bytes sent
    pub bytes_sent: u64,
    /// Number of bytes received
    pub bytes_received: u64,
    /// Time until connection times out
    pub time_until_timeout: Option<Duration>,
}
