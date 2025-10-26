//! QUIC transport layer (placeholder for future implementation)
//!
//! This module will provide QUIC-based transport using quinn or similar.
//! For now, we use TCP as the primary transport.

use tokio::io::{AsyncRead, AsyncWrite};
use std::pin::Pin;

/// A trait object combining AsyncRead + AsyncWrite + Unpin + Send
/// We use a custom trait to avoid the E0225 error with multiple non-auto traits
pub trait StreamTrait: AsyncRead + AsyncWrite + Unpin + Send {}

// Blanket implementation for any type that satisfies the constraints
impl<T> StreamTrait for T where T: AsyncRead + AsyncWrite + Unpin + Send {}

/// Type alias for dynamic stream (avoids trait object issues)
pub type DynStream = Pin<Box<dyn StreamTrait>>;

/// Placeholder for QUIC connection
pub struct QuicConnection {
    // Future: quinn connection
}

impl QuicConnection {
    /// Create a new QUIC connection (not yet implemented)
    pub async fn connect(_addr: &str) -> anyhow::Result<Self> {
        anyhow::bail!("QUIC transport not yet implemented")
    }

    /// Accept a QUIC connection (not yet implemented)
    pub async fn accept() -> anyhow::Result<Self> {
        anyhow::bail!("QUIC transport not yet implemented")
    }
}

// Future: Implement AsyncRead and AsyncWrite for QuicConnection