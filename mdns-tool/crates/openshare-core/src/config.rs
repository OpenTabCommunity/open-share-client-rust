use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// Directory for storing chunks, manifests, and local cache
    pub data_dir: PathBuf,

    /// Default chunk size for file splitting (256 KiB as per architecture)
    pub chunk_size: usize,

    /// Port to listen on for incoming connections
    pub listen_port: u16,

    /// mDNS service type
    pub service_type: String,

    /// Account hash for discovery filtering
    pub account_hash: String,

    /// Device ID
    pub device_id: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            data_dir: std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join(".openshare"),
            chunk_size: 256 * 1024, // 256 KiB
            listen_port: 9876,
            service_type: "_openshare._tcp.local.".to_string(),
            account_hash: "".to_string(),
            device_id: "".to_string(),
        }
    }
}

impl ClientConfig {
    pub fn with_account(mut self, account_hash: String, device_id: String) -> Self {
        self.account_hash = account_hash;
        self.device_id = device_id;
        self
    }

    pub fn ensure_data_dir(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(self.data_dir.join("chunks"))?;
        std::fs::create_dir_all(self.data_dir.join("manifests"))?;
        Ok(())
    }
}