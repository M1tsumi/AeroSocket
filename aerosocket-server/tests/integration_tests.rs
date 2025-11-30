//! Integration tests for the AeroSocket WebSocket server
//!
//! These tests verify the core functionality of the WebSocket server components.

use aerosocket_server::prelude::*;
use aerosocket_server::{
    ConnectionManager, CloseReason, ErrorContext, ServerError, ContextError
};
use aerosocket_server::handler::{FnHandler, from_fn};
use aerosocket_core::Message;
use std::time::Duration;

/// Test connection creation and basic functionality
#[tokio::test]
async fn test_connection_creation() {
    let remote_addr = "127.0.0.1:12345".parse().unwrap();
    let local_addr = "127.0.0.1:8080".parse().unwrap();
    
    // Test basic connection creation
    let connection = Connection::new(remote_addr, local_addr);
    assert_eq!(connection.remote_addr(), remote_addr);
    assert_eq!(connection.local_addr(), local_addr);
    assert_eq!(connection.state(), ConnectionState::Connecting);
    
    // Test connection with timeout
    let connection_with_timeout = Connection::with_timeout(
        remote_addr,
        local_addr,
        Box::new(MockTransportStream::new()),
        Some(Duration::from_secs(30))
    );
    assert_eq!(connection_with_timeout.remote_addr(), remote_addr);
    assert!(connection_with_timeout.time_until_timeout().is_some());
}

/// Test connection timeout functionality
#[tokio::test]
async fn test_connection_timeout() {
    let remote_addr = "127.0.0.1:12345".parse().unwrap();
    let local_addr = "127.0.0.1:8080".parse().unwrap();
    
    // Create connection with short timeout
    let connection = Connection::with_timeout(
        remote_addr,
        local_addr,
        Box::new(MockTransportStream::new()),
        Some(Duration::from_millis(100))
    );
    
    // Initially should not be timed out
    assert!(!connection.is_timed_out());
    assert!(connection.time_until_timeout().is_some());
    
    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;
    
    // Should now be timed out
    assert!(connection.is_timed_out());
    assert_eq!(connection.time_until_timeout(), Some(Duration::ZERO));
}

/// Test connection manager functionality
#[tokio::test]
async fn test_connection_manager() {
    let config = ServerConfig::default();
    let manager = ConnectionManager::new(config);
    
    // Test initial stats
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_connections, 0);
    assert_eq!(stats.total_connections, 0);
    
    // Test adding connections
    let remote_addr = "127.0.0.1:12345".parse().unwrap();
    let local_addr = "127.0.0.1:8080".parse().unwrap();
    
    let connection1 = Connection::new(remote_addr, local_addr);
    let connection2 = Connection::new(remote_addr, local_addr);
    
    let handle1 = manager.add_connection(connection1).await.unwrap();
    let handle2 = manager.add_connection(connection2).await.unwrap();
    
    // Verify stats after adding connections
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_connections, 2);
    assert_eq!(stats.total_connections, 2);
    assert_eq!(stats.peak_connections, 2);
    
    // Test getting connections
    let retrieved_handle1 = manager.get_connection(handle1.id()).await;
    assert!(retrieved_handle1.is_some());
    assert_eq!(retrieved_handle1.unwrap().id(), handle1.id());
    
    let all_connections = manager.get_all_connections().await;
    assert_eq!(all_connections.len(), 2);
    
    // Test removing connections
    manager.remove_connection(handle1.id(), CloseReason::Normal).await;
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_connections, 1);
    assert_eq!(stats.total_connections, 2);
    assert_eq!(stats.normal_closures, 1);
    
    // Test connection health monitoring
    let health_reports = manager.monitor_connections().await.unwrap();
    assert_eq!(health_reports.len(), 1);
    assert_eq!(health_reports[0].id, handle2.id());
    assert_eq!(health_reports[0].remote_addr, remote_addr);
    
    // Test closing all connections
    manager.close_all_connections().await;
    
    let stats = manager.get_stats().await;
    assert_eq!(stats.active_connections, 0);
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    // Test error context creation
    let context = ErrorContext::new()
        .with_connection_id(123)
        .with_remote_addr("127.0.0.1:8080".parse().unwrap())
        .with_operation("test_operation")
        .with_context("key1", "value1")
        .with_context("key2", "value2");
    
    assert_eq!(context.connection_id, Some(123));
    assert_eq!(context.remote_addr, Some("127.0.0.1:8080".parse().unwrap()));
    assert_eq!(context.operation, Some("test_operation".to_string()));
    assert_eq!(context.context.get("key1"), Some(&"value1".to_string()));
    assert_eq!(context.context.get("key2"), Some(&"value2".to_string()));
    
    // Test context error creation
    let server_error = ServerError::Timeout { duration: Duration::from_secs(30) };
    let context_error = ContextError {
        error: server_error,
        context: context.clone(),
    };
    
    let error_string = context_error.to_string();
    assert!(error_string.contains("Operation timed out"));
    assert!(error_string.contains("123"));
    assert!(error_string.contains("127.0.0.1:8080"));
    assert!(error_string.contains("test_operation"));
}

/// Test handler functionality
#[tokio::test]
async fn test_handler_creation() {
    // Test echo handler
    let echo_handler = EchoHandler::new();
    let remote_addr = "127.0.0.1:12345".parse().unwrap();
    let local_addr = "127.0.0.1:8080".parse().unwrap();
    let connection = Connection::new(remote_addr, local_addr);
    let handle = ConnectionHandle::new(1, connection);
    
    // Handler should be cloneable
    let echo_handler2 = echo_handler.clone();
    
    // Test default handler
    let default_handler = DefaultHandler::new();
    let default_handler2 = default_handler.clone();
    
    // Test function handler
    let fn_handler = from_fn(|_connection| async move {
        Ok(())
    });
    let fn_handler2 = fn_handler.clone();
}

/// Test message creation and manipulation
#[tokio::test]
async fn test_message_handling() {
    // Test text message
    let text_msg = Message::text("Hello, World!".to_string());
    match text_msg {
        Message::Text(text) => assert_eq!(text.as_str(), "Hello, World!"),
        _ => panic!("Expected text message"),
    }
    
    // Test binary message
    let binary_data = vec![1, 2, 3, 4, 5];
    let binary_msg = Message::binary(binary_data.clone());
    match binary_msg {
        Message::Binary(data) => assert_eq!(data.as_bytes(), &binary_data[..]),
        _ => panic!("Expected binary message"),
    }
    
    // Test ping message
    let ping_data = vec![9, 8, 7];
    let ping_msg = Message::ping(Some(ping_data.clone()));
    match ping_msg {
        Message::Ping(data) => assert_eq!(data.as_bytes(), &ping_data[..]),
        _ => panic!("Expected ping message"),
    }
    
    // Test pong message
    let pong_data = vec![6, 5, 4];
    let pong_msg = Message::pong(Some(pong_data.clone()));
    match pong_msg {
        Message::Pong(data) => assert_eq!(data.as_bytes(), &pong_data[..]),
        _ => panic!("Expected pong message"),
    }
    
    // Test close message
    let close_msg = Message::close(Some(1000), Some("Normal closure".to_string()));
    match close_msg {
        Message::Close(code_and_reason) => {
            assert_eq!(code_and_reason.code(), Some(1000));
            assert_eq!(code_and_reason.reason(), "Normal closure");
        }
        _ => panic!("Expected close message"),
    }
}

/// Mock transport stream for testing
struct MockTransportStream {
    closed: bool,
}

impl MockTransportStream {
    fn new() -> Self {
        Self { closed: false }
    }
}

#[async_trait::async_trait]
impl aerosocket_core::transport::TransportStream for MockTransportStream {
    async fn read(&mut self, _buf: &mut [u8]) -> aerosocket_core::Result<usize> {
        Ok(0) // Simulate EOF
    }
    
    async fn write(&mut self, _buf: &[u8]) -> aerosocket_core::Result<usize> {
        Ok(_buf.len())
    }
    
    async fn write_all(&mut self, _buf: &[u8]) -> aerosocket_core::Result<()> {
        Ok(())
    }
    
    async fn flush(&mut self) -> aerosocket_core::Result<()> {
        Ok(())
    }
    
    async fn close(&mut self) -> aerosocket_core::Result<()> {
        self.closed = true;
        Ok(())
    }
    
    fn remote_addr(&self) -> aerosocket_core::Result<std::net::SocketAddr> {
        Ok("127.0.0.1:12345".parse().unwrap())
    }
    
    fn local_addr(&self) -> aerosocket_core::Result<std::net::SocketAddr> {
        Ok("127.0.0.1:8080".parse().unwrap())
    }
}
