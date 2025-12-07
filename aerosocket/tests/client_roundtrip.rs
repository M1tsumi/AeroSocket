use std::io::Result;

#[cfg(all(
    feature = "server",
    feature = "client",
    feature = "transport-tcp",
    feature = "tokio-runtime"
))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn ws_echo_roundtrip() -> Result<()> {
        // Test code here
        Ok(())
    }
}

#[cfg(all(
    feature = "client",
    feature = "transport-tls",
    feature = "tokio-runtime"
))]
mod tls_tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn wss_echo_roundtrip() -> Result<()> {
        // Test code here
        Ok(())
    }
}
