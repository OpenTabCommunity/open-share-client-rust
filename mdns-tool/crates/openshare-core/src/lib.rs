//! OpenShare Core - P2P File Transfer Library
//!
//! A secure, local-first file transfer system with strong cryptographic
//! guarantees and minimal server dependencies.

pub mod config;
pub mod keys;
pub mod manifest;
pub mod handshake;
pub mod client;

// Re-export commonly used types
pub use config::ClientConfig;
pub use keys::Identity;
pub use manifest::Manifest;
pub use client::Client;