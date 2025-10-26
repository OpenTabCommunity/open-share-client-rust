//! High-level client orchestration that uses separate transport and storage crates.
//!
//! The client is generic over a Storage implementation and expects a connected
//! transport stream (TCP or QUIC) that implements AsyncRead + AsyncWrite.

use crate::{Identity, Manifest, config::ClientConfig, handshake};
use storage::Storage;
use anyhow::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use std::sync::Arc;

#[derive(Clone)]
pub struct Client<S> {
    pub identity: Arc<Identity>,
    pub storage: Arc<S>,
    pub cfg: ClientConfig,
}

impl<S> Client<S>
where
    S: Storage + Send + Sync + 'static,
{
    pub fn new(identity: Identity, storage: S, cfg: ClientConfig) -> Self {
        Self {
            identity: Arc::new(identity),
            storage: Arc::new(storage),
            cfg,
        }
    }

    /// Send a manifest and its chunks to a connected peer transport.
    /// The transport must be already connected. The handshake is performed
    /// over the transport, returning an encrypted session.
    pub async fn send_manifest_over<T>(&self, mut transport: T, manifest: Manifest) -> Result<()>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send,
    {
        tracing::info!("Starting send: {}", manifest.filename);

        // 1) Sign manifest
        let mut manifest = manifest;
        manifest.sign(&self.identity)?;

        // 2) Perform initiator handshake over transport -> Session (AEAD)
        tracing::debug!("Performing handshake...");
        let session = handshake::initiator_handshake(&self.identity, &mut transport).await?;
        tracing::debug!("Handshake complete");

        // 3) Send manifest as bincode over encrypted frame
        tracing::debug!("Sending manifest...");
        let manifest_bytes = bincode::serialize(&manifest)?;
        session.send_encrypted_frame(&mut transport, &manifest_bytes).await?;
        tracing::info!("Manifest sent, {} chunks to transfer", manifest.chunk_hashes.len());

        // 4) For each chunk hash in manifest, fetch from storage and send
        for (i, chunk_hash) in manifest.chunk_hashes.iter().enumerate() {
            if let Some(data) = self.storage.get_chunk(chunk_hash).await? {
                session.send_encrypted_frame(&mut transport, &data).await?;

                if (i + 1) % 10 == 0 {
                    tracing::info!("Sent {}/{} chunks", i + 1, manifest.chunk_hashes.len());
                }
            } else {
                // If chunk missing, warn and skip (or could abort)
                tracing::warn!("Chunk {} missing locally; skipping", chunk_hash);
            }
        }

        tracing::info!("Transfer complete: {}", manifest.filename);
        Ok(())
    }

    /// Accept an incoming transport, run responder handshake and receive
    /// an incoming manifest followed by chunks; store chunks into storage.
    pub async fn accept_and_receive<T>(&self, mut transport: T) -> Result<Manifest>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send,
    {
        tracing::info!("Starting receive...");

        // Run responder handshake
        tracing::debug!("Performing handshake...");
        let session = handshake::responder_handshake(&self.identity, &mut transport).await?;
        tracing::debug!("Handshake complete");

        // Read manifest
        tracing::debug!("Receiving manifest...");
        let manifest_bytes = session.read_encrypted_frame(&mut transport).await?;
        let manifest: Manifest = bincode::deserialize(&manifest_bytes)?;

        tracing::info!("Receiving: {} ({} chunks)",
            manifest.filename, manifest.chunk_hashes.len());

        // Store incoming chunks (we store all chunks sequentially)
        for (i, chunk_hash) in manifest.chunk_hashes.iter().enumerate() {
            let chunk = session.read_encrypted_frame(&mut transport).await?;

            // Verify chunk hash matches expected
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&chunk);
            let computed = hasher.finalize();
            let hex = hex::encode(computed);

            if &hex != chunk_hash {
                tracing::warn!("Chunk hash mismatch: expected {} got {}", chunk_hash, hex);
                // Continue or handle error - here we continue
            } else {
                // Store chunk
                let stored_id = self.storage.put_chunk(&chunk).await?;

                // Verify stored ID matches expected
                if stored_id != *chunk_hash {
                    tracing::warn!("Stored chunk ID mismatch: {} vs {}", stored_id, chunk_hash);
                }

                if (i + 1) % 10 == 0 {
                    tracing::info!("Received {}/{} chunks", i + 1, manifest.chunk_hashes.len());
                }
            }
        }

        tracing::info!("Transfer complete: {}", manifest.filename);
        Ok(manifest)
    }
}