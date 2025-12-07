use aerosocket::prelude::*;
use std::net::SocketAddr;
use std::time::Duration;

#[cfg(all(feature = "server", feature = "client", feature = "transport-tcp", feature = "tokio-runtime"))]
mod tests {
    use super::*;
    use aerosocket::Error;

    #[tokio::test]
    async fn ws_echo_roundtrip() -> Result<()> {
        // Start an echo server on a local port
        let addr: SocketAddr = "127.0.0.1:19090".parse().unwrap();

        let server = Server::builder()
            .bind(&addr.to_string())
            .max_connections(100)
            .build()?;

        let server_task = tokio::spawn(async move {
            let _ = server
                .serve(|mut conn| async move {
                    while let Some(msg) = conn.next().await? {
                        match msg {
                            Message::Text(text) => {
                                conn.send_text(text).await?;
                            }
                            Message::Binary(data) => {
                                conn.send_binary(data).await?;
                            }
                            _ => {}
                        }
                    }
                    Ok(())
                })
                .await;
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Create a client and perform a simple echo round-trip over TCP
        let config = ClientConfig::default();
        let client = Client::new(addr).with_config(config);
        let mut conn = client.connect().await?;

        conn.send_text("hello").await?;
        match conn.next().await? {
            Some(Message::Text(reply)) => {
                assert_eq!(reply.as_str(), "hello");
            }
            other => panic!("expected text reply, got {:?}", other),
        }

        conn.close(Some(1000), Some("done")).await?;

        // Shut down server task
        server_task.abort();

        Ok(())
    }
}

#[cfg(all(feature = "client", feature = "transport-tls", feature = "tokio-runtime"))]
mod tls_tests {
    use super::*;
    use aerosocket_core::handshake::{
        create_server_handshake, parse_client_handshake, response_to_string, validate_client_handshake,
        HandshakeConfig,
    };
    use aerosocket_core::frame::Frame;
    use bytes::BytesMut;
    use rcgen::{CertificateParams, Certificate, DistinguishedName, DnType, IsCa, BasicConstraints};
    use rustls::{Certificate as RustlsCert, PrivateKey as RustlsKey, ServerConfig as RustlsServerConfig};
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn wss_echo_roundtrip() -> Result<()> {
        // Generate a self-signed CA and server certificate for localhost
        let mut ca_params = CertificateParams::default();
        ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        let mut ca_dn = DistinguishedName::new();
        ca_dn.push(DnType::CommonName, "test-ca");
        ca_params.distinguished_name = ca_dn;
        let ca_cert = Certificate::from_params(ca_params).map_err(|e| Error::Other(e.to_string()))?;

        let mut server_params = CertificateParams::new(vec!["localhost".to_string()]);
        let mut server_dn = DistinguishedName::new();
        server_dn.push(DnType::CommonName, "localhost");
        server_params.distinguished_name = server_dn;
        server_params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
        server_params.is_ca = IsCa::NoCa;
        // Sign server cert with CA
        let server_cert = Certificate::from_params(server_params).map_err(|e| Error::Other(e.to_string()))?;
        let server_der = server_cert
            .serialize_der_with_signer(&ca_cert)
            .map_err(|e| Error::Other(e.to_string()))?;
        let ca_pem = ca_cert.serialize_pem().map_err(|e| Error::Other(e.to_string()))?;
        let server_key_der = server_cert.serialize_private_key_der();

        let rustls_server_cert = RustlsCert(server_der);
        let rustls_server_key = RustlsKey(server_key_der);

        let server_config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![rustls_server_cert], rustls_server_key)?;

        // Start a TLS listener on a local port
        let addr: SocketAddr = "127.0.0.1:19443".parse().unwrap();
        let listener = TcpListener::bind(addr).await?;
        let tls_acceptor = TlsAcceptor::from(Arc::new(server_config));

        // Write CA cert to a temp file so client TlsConfig.ca_file can use it
        let mut ca_file = NamedTempFile::new()?;
        use std::io::Write as _;
        ca_file.write_all(ca_pem.as_bytes())?;
        let ca_path = ca_file.into_temp_path();

        // Spawn minimal TLS WebSocket echo server
        let server_task = tokio::spawn(async move {
            loop {
                let (tcp_stream, _peer) = match listener.accept().await {
                    Ok(v) => v,
                    Err(_) => break,
                };

                let acceptor = tls_acceptor.clone();
                tokio::spawn(async move {
                    if let Ok(mut tls_stream) = acceptor.accept(tcp_stream).await {
                        let mut buf = Vec::new();
                        let mut tmp = [0u8; 1024];

                        // Read HTTP request until CRLFCRLF
                        loop {
                            match tls_stream.read(&mut tmp).await {
                                Ok(0) => return,
                                Ok(n) => {
                                    buf.extend_from_slice(&tmp[..n]);
                                    if buf.windows(4).any(|w| w == b"\r\n\r\n") {
                                        break;
                                    }
                                }
                                Err(_) => return,
                            }
                        }

                        let req_str = String::from_utf8_lossy(&buf);
                        let request = match parse_client_handshake(&req_str) {
                            Ok(r) => r,
                            Err(_) => return,
                        };

                        let handshake_config = HandshakeConfig {
                            protocols: vec![],
                            extensions: vec![],
                            origin: None,
                            host: None,
                            extra_headers: Default::default(),
                        };

                        if validate_client_handshake(&request, &handshake_config).is_err() {
                            return;
                        }

                        let response = match create_server_handshake(&request, &handshake_config) {
                            Ok(r) => r,
                            Err(_) => return,
                        };
                        let resp_str = response_to_string(&response);
                        if tls_stream.write_all(resp_str.as_bytes()).await.is_err() {
                            return;
                        }
                        if tls_stream.flush().await.is_err() {
                            return;
                        }

                        // After handshake, echo a single text frame at WebSocket level
                        let mut frame_buf = BytesMut::new();

                        // Read bytes until we can parse a WebSocket frame
                        loop {
                            let mut tmp_frame = [0u8; 1024];
                            match tls_stream.read(&mut tmp_frame).await {
                                Ok(0) => return,
                                Ok(n) => {
                                    frame_buf.extend_from_slice(&tmp_frame[..n]);
                                    match Frame::parse(&mut frame_buf) {
                                        Ok(frame) => {
                                            // Echo text frames back to the client
                                            if let aerosocket_core::protocol::Opcode::Text = frame.opcode {
                                                let reply = Frame::text(frame.payload.to_vec());
                                                let _ = tls_stream.write_all(&reply.to_bytes()).await;
                                                let _ = tls_stream.flush().await;
                                            }
                                            return;
                                        }
                                        Err(_) => {
                                            // Need more data, continue reading
                                            continue;
                                        }
                                    }
                                }
                                Err(_) => return,
                            }
                        }
                    }
                });
            }
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Configure TLS client to trust our test CA and use SNI localhost
        let tls_config = TlsConfig {
            verify: true,
            ca_file: Some(ca_path.to_string_lossy().to_string()),
            cert_file: None,
            key_file: None,
            server_name: Some("localhost".to_string()),
            min_version: None,
            max_version: None,
        };

        let mut client_config = ClientConfig::default();
        client_config.tls = Some(tls_config);

        // Create client and perform WSS echo round-trip
        let client = Client::new(addr).with_config(client_config);
        let mut conn = client.connect().await?;

        conn.send_text("hello over tls").await?;
        match conn.next().await? {
            Some(Message::Text(reply)) => {
                assert_eq!(reply.as_str(), "hello over tls");
            }
            other => panic!("expected text reply over tls, got {:?}", other),
        }

        conn.close(Some(1000), Some("done"))
            .await?;

        server_task.abort();

        Ok(())
    }
}
