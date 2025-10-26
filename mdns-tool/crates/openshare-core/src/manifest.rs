use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use anyhow::{Result, Context};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use hex::encode as hex_encode;
use crate::Identity;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Manifest describing a file transfer: filename, size, ordered chunk hashes,
/// and an optional sender signature over the manifest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub filename: String,
    pub size: u64,
    pub chunk_hashes: Vec<String>,
    pub sender_sig: Option<Vec<u8>>,
    pub sender_pubkey: Option<Vec<u8>>, // Store sender's public key for verification
}

impl Manifest {
    /// Build a manifest by chunking a file from disk using chunk_size.
    pub fn from_file(path: &str, chunk_size: usize) -> Result<Self> {
        let mut f = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path))?;

        let size = f.seek(SeekFrom::End(0))?;
        f.seek(SeekFrom::Start(0))?;

        let mut chunk_hashes = Vec::new();
        let mut buf = vec![0u8; chunk_size];

        loop {
            let n = f.read(&mut buf)?;
            if n == 0 { break; }

            let mut hasher = Sha256::new();
            hasher.update(&buf[..n]);
            let h = hasher.finalize();
            chunk_hashes.push(hex_encode(h));
        }

        // Extract just the filename, not the full path
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();

        Ok(Self {
            filename,
            size,
            chunk_hashes,
            sender_sig: None,
            sender_pubkey: None,
        })
    }

    /// Sign the manifest using identity (the signature covers the manifest with
    /// sender_sig set to None).
    pub fn sign(&mut self, identity: &Identity) -> Result<()> {
        // Store the sender's public key
        self.sender_pubkey = Some(identity.public_key_bytes().to_vec());

        // Create a copy without signature for signing
        let mut copy = self.clone();
        copy.sender_sig = None;

        let ser = bincode::serialize(&copy)
            .context("Failed to serialize manifest for signing")?;
        let sig = identity.sign(&ser);

        self.sender_sig = Some(sig.to_bytes().to_vec());
        Ok(())
    }

    /// Verify the manifest signature using the stored public key.
    pub fn verify(&self) -> Result<()> {
        let _sig_bytes = self.sender_sig.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing signature"))?;

        let pubkey_bytes = self.sender_pubkey.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing sender public key"))?;

        if pubkey_bytes.len() != 32 {
            anyhow::bail!("Invalid public key length");
        }

        let mut pubkey_arr = [0u8; 32];
        pubkey_arr.copy_from_slice(pubkey_bytes);

        self.verify_with_pubkey(&pubkey_arr)
    }

    /// Verify the manifest signature using a specific public key.
    pub fn verify_with_pubkey(&self, pubkey_bytes: &[u8; 32]) -> Result<()> {
        let sig_bytes = self.sender_sig.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing signature"))?;

        // Convert Vec<u8> to [u8; 64] for Signature
        if sig_bytes.len() != 64 {
            anyhow::bail!("Invalid signature length: expected 64 bytes, got {}", sig_bytes.len());
        }
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(sig_bytes);

        let sig = Signature::from_bytes(&sig_arr);

        let mut copy = self.clone();
        copy.sender_sig = None;

        let ser = bincode::serialize(&copy)
            .context("Failed to serialize manifest for verification")?;

        let pk = VerifyingKey::from_bytes(pubkey_bytes)
            .context("Invalid public key")?;

        pk.verify(&ser, &sig)
            .context("Signature verification failed")?;

        Ok(())
    }

    /// Get a summary string for display.
    pub fn summary(&self) -> String {
        format!(
            "{} ({} bytes, {} chunks)",
            self.filename,
            self.size,
            self.chunk_hashes.len()
        )
    }
}