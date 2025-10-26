use anyhow::{Result, Context};
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use sha2::{Digest, Sha256};
use hex;

/// Storage trait for chunk persistence.
#[async_trait]
pub trait Storage: Send + Sync {
    async fn put_chunk(&self, data: &[u8]) -> Result<String>;
    async fn get_chunk(&self, id: &str) -> Result<Option<Vec<u8>>>;
}

/// Local filesystem-based storage implementation.
#[derive(Clone)]
pub struct LocalStorage {
    chunks_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(base_dir: PathBuf) -> Result<Self> {
        let chunks_dir = base_dir.join("chunks");
        std::fs::create_dir_all(&chunks_dir)
            .context("Failed to create chunks directory")?;

        Ok(Self { chunks_dir })
    }

    fn chunk_path(&self, chunk_id: &str) -> PathBuf {
        // Use first 2 chars as subdirectory for better filesystem performance
        let prefix = &chunk_id[..2.min(chunk_id.len())];
        self.chunks_dir.join(prefix).join(chunk_id)
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn put_chunk(&self, data: &[u8]) -> Result<String> {
        // Compute chunk ID as SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let chunk_id = hex::encode(hash);

        let path = self.chunk_path(&chunk_id);

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await
                .context("Failed to create chunk subdirectory")?;
        }

        // Write chunk to disk
        fs::write(&path, data).await
            .with_context(|| format!("Failed to write chunk {}", chunk_id))?;

        tracing::debug!("Stored chunk {} ({} bytes)", chunk_id, data.len());
        Ok(chunk_id)
    }

    async fn get_chunk(&self, id: &str) -> Result<Option<Vec<u8>>> {
        let path = self.chunk_path(id);

        if !path.exists() {
            return Ok(None);
        }

        let data = fs::read(&path).await
            .with_context(|| format!("Failed to read chunk {}", id))?;

        tracing::debug!("Retrieved chunk {} ({} bytes)", id, data.len());
        Ok(Some(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_roundtrip() -> Result<()> {
        let temp = TempDir::new()?;
        let storage = LocalStorage::new(temp.path().to_path_buf())?;

        let data = b"test chunk data";
        let id = storage.put_chunk(data).await?;

        let retrieved = storage.get_chunk(&id).await?;
        assert_eq!(retrieved, Some(data.to_vec()));

        let missing = storage.get_chunk("nonexistent").await?;
        assert_eq!(missing, None);

        Ok(())
    }
}