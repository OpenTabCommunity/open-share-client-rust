use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand_core::OsRng;
use std::path::Path;
use std::fs;
use anyhow::{Context, Result};
use hex;

/// Identity wrapper for Ed25519 keypair used for device identity.
///
/// NOTE: Production should use OS keystore/secure enclave. This is a simple
/// on-disk representation for development and testing.
#[derive(Clone)]
pub struct Identity {
    pub signing_key: SigningKey,
}

impl Identity {
    /// Generate a new identity keypair and persist to `path`.
    /// The file stores the 32-byte secret key.
    pub fn generate_and_store(path: &Path) -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Store the secret key bytes
        fs::write(path, signing_key.to_bytes()).context("writing identity file")?;
        tracing::info!("Generated new identity at {:?}", path);
        Ok(Self { signing_key })
    }

    /// Load an identity from path.
    pub fn load(path: &Path) -> Result<Self> {
        let data = fs::read(path).context("reading identity file")?;
        if data.len() != 32 {
            anyhow::bail!("Invalid key file length: expected 32 bytes, got {}", data.len());
        }
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&data);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        tracing::info!("Loaded identity from {:?}", path);
        Ok(Self { signing_key })
    }

    /// Load existing identity or generate a new one if not found.
    pub fn load_or_generate(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            Self::generate_and_store(path)
        }
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.signing_key.verifying_key().to_bytes()
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Get a short fingerprint for display (first 8 hex chars of pubkey).
    pub fn fingerprint(&self) -> String {
        let pubkey = self.public_key_bytes();
        hex::encode(&pubkey[..4])
    }

    /// Get full fingerprint for verification.
    pub fn full_fingerprint(&self) -> String {
        hex::encode(self.public_key_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing_key.sign(msg)
    }

    /// Verify a signature by a public key.
    pub fn verify_with_pubkey(
        pubkey: &[u8; 32],
        msg: &[u8],
        sig: &Signature
    ) -> Result<(), ed25519_dalek::SignatureError> {
        let pk = VerifyingKey::from_bytes(pubkey)?;
        pk.verify(msg, sig)
    }
}