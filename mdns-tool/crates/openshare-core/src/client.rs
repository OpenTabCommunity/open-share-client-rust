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

#[cfg(test)]
mod tests {
    use std::thread::spawn;
use std::time::Duration;
use super::*;
    use storage::LocalStorage;
    use tokio::net::{TcpListener, TcpStream};
    use std::path::PathBuf;
    use storage::LocalStorage;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_client_transfer() -> Result<()> {
        // Setup temp directories
        let temp_sender = TempDir::new()?;
        let temp_receiver = TempDir::new()?;

        // Create identities
        let sender_id = Identity::generate_and_store(
            &temp_sender.path().join("sender.key")
        )?;
        let receiver_id = Identity::generate_and_store(
            &temp_receiver.path().join("receiver.key")
        )?;

        // Create storages
        let sender_storage = LocalStorage::new(temp_sender.path().to_path_buf())?;
        let receiver_storage = LocalStorage::new(temp_receiver.path().to_path_buf())?;

        // Create test file and manifest
        let test_data = b"Hello, OpenShare!";
        let chunk_id = sender_storage.put_chunk(test_data).await?;

        let mut manifest = Manifest {
            filename: "test.txt".to_string(),
            size: test_data.len() as u64,
            chunk_hashes: vec![chunk_id],
            sender_sig: None,
            sender_pubkey: None,
        };
        manifest.sign(&sender_id)?;

        // Setup listener
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // Spawn receiver
        let receiver_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let cfg = ClientConfig::default();
            let client = Client::new(receiver_id, receiver_storage, cfg);
            client.accept_and_receive(stream).await.unwrap()
        });

        // Connect and send
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let stream = TcpStream::connect(addr).await?;
        let cfg = ClientConfig::default();
        let client = Client::new(sender_id, sender_storage, cfg);
        client.send_manifest_over(stream, manifest.clone()).await?;

        // Wait for receiver
        let received_manifest = receiver_handle.await??;
        assert_eq!(received_manifest.filename, manifest.filename);
        assert_eq!(received_manifest.size, manifest.size);

        Ok(())
    }
}
