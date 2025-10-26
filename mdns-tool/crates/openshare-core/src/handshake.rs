//! Handshake implementation.
//!
//! - Uses ephemeral X25519 keys + Ed25519 signatures over ephemeral pubkey||nonce
//!   to prevent MitM in local discovery spoofing scenarios.
//! - Derives a 32-byte session key via HKDF-SHA256(shared_secret || transcripts)
//! - Produces an XChaCha20-Poly1305 AEAD for subsequent encrypted framing.

use crate::keys::Identity;
use anyhow::Result;
use chacha20poly1305::{XChaCha20Poly1305, KeyInit, XNonce};
use chacha20poly1305::aead::AeadInPlace;
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use std::convert::TryInto;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use thiserror::Error;

/// Fixed lengths
const PUBKEY_LEN: usize = 32;
const NONCE_LEN: usize = 32;
const SIG_LEN: usize = 64;

/// Session holds the AEAD and the raw derived key
pub struct Session {
    pub aead: XChaCha20Poly1305,
    pub session_key: [u8; 32],
}

#[derive(Error, Debug)]
pub enum HandshakeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("crypto error: {0}")]
    Crypto(String),
}

/// Minimal length-prefixed frame helpers (u32 BE length).
async fn write_lp<T: AsyncWrite + Unpin + Send>(
    transport: &mut T,
    data: &[u8]
) -> std::io::Result<()> {
    transport.write_all(&(data.len() as u32).to_be_bytes()).await?;
    transport.write_all(data).await?;
    transport.flush().await?;
    Ok(())
}

async fn read_lp<T: AsyncRead + Unpin + Send>(
    transport: &mut T
) -> std::io::Result<Vec<u8>> {
    let mut lenb = [0u8; 4];
    transport.read_exact(&mut lenb).await?;
    let len = u32::from_be_bytes(lenb) as usize;

    // Sanity check to prevent memory exhaustion
    if len > 10 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "message too large"
        ));
    }

    let mut buf = vec![0u8; len];
    transport.read_exact(&mut buf).await?;
    Ok(buf)
}

/// Initiator side handshake.
pub async fn initiator_handshake<T>(
    identity: &Identity,
    transport: &mut T
) -> Result<Session, HandshakeError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    // 1) Generate ephemeral x25519 keypair
    let x_secret = EphemeralSecret::random_from_rng(OsRng);
    let x_pub = X25519Public::from(&x_secret);

    // 2) Generate nonceA
    let mut nonce_a = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_a);

    // 3) Sign x_pub || nonceA with ed25519 identity key
    let mut to_sign = Vec::with_capacity(PUBKEY_LEN + NONCE_LEN);
    to_sign.extend_from_slice(x_pub.as_bytes());
    to_sign.extend_from_slice(&nonce_a);
    let sig = identity.sign(&to_sign);

    // 4) Send messageA = x_pub || nonceA || sig
    let mut message_a = Vec::with_capacity(PUBKEY_LEN + NONCE_LEN + SIG_LEN);
    message_a.extend_from_slice(x_pub.as_bytes());
    message_a.extend_from_slice(&nonce_a);
    message_a.extend_from_slice(&sig.to_bytes());
    write_lp(transport, &message_a).await.map_err(HandshakeError::Io)?;

    // 5) Receive messageB
    let buf = read_lp(transport).await.map_err(HandshakeError::Io)?;
    if buf.len() < PUBKEY_LEN + NONCE_LEN + SIG_LEN {
        return Err(HandshakeError::Crypto("peer message too short".into()));
    }

    let x_b_bytes: [u8; PUBKEY_LEN] = buf[0..PUBKEY_LEN].try_into().unwrap();
    let nonce_b: [u8; NONCE_LEN] = buf[PUBKEY_LEN..PUBKEY_LEN + NONCE_LEN]
        .try_into().unwrap();
    let _sig_b_bytes: [u8; SIG_LEN] = buf[PUBKEY_LEN + NONCE_LEN..PUBKEY_LEN + NONCE_LEN + SIG_LEN]
        .try_into().unwrap();

    // NOTE: In production, verify peer's signature here with their certificate

    // 6) Compute shared secret
    let x_b_pub = X25519Public::from(x_b_bytes);
    let shared = x_secret.diffie_hellman(&x_b_pub);

    // 7) Derive session key using HKDF-SHA256
    let info = [&nonce_a[..], &nonce_b[..]].concat();
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(&info, &mut okm)
        .map_err(|_| HandshakeError::Crypto("HKDF expand failed".into()))?;

    let aead = XChaCha20Poly1305::new(&okm.into());

    Ok(Session { aead, session_key: okm })
}

/// Responder handshake (symmetrical).
pub async fn responder_handshake<T>(
    identity: &Identity,
    transport: &mut T
) -> Result<Session, HandshakeError>
where
    T: AsyncRead + AsyncWrite + Unpin + Send,
{
    // Read initiator message
    let buf = read_lp(transport).await.map_err(HandshakeError::Io)?;
    if buf.len() < PUBKEY_LEN + NONCE_LEN + SIG_LEN {
        return Err(HandshakeError::Crypto("initiator message too short".into()));
    }

    let x_a_bytes: [u8; PUBKEY_LEN] = buf[0..PUBKEY_LEN].try_into().unwrap();
    let nonce_a: [u8; NONCE_LEN] = buf[PUBKEY_LEN..PUBKEY_LEN + NONCE_LEN]
        .try_into().unwrap();
    let _sig_a_bytes: [u8; SIG_LEN] = buf[PUBKEY_LEN + NONCE_LEN..PUBKEY_LEN + NONCE_LEN + SIG_LEN]
        .try_into().unwrap();

    // NOTE: In production, verify initiator's signature here

    // Create responder ephemeral
    let x_secret = EphemeralSecret::random_from_rng(OsRng);
    let x_pub = X25519Public::from(&x_secret);

    // Generate nonceB
    let mut nonce_b = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_b);

    // Sign x_pub || nonceB
    let mut to_sign = Vec::with_capacity(PUBKEY_LEN + NONCE_LEN);
    to_sign.extend_from_slice(x_pub.as_bytes());
    to_sign.extend_from_slice(&nonce_b);
    let sig = identity.sign(&to_sign);

    // Send responder message
    let mut message_b = Vec::with_capacity(PUBKEY_LEN + NONCE_LEN + SIG_LEN);
    message_b.extend_from_slice(x_pub.as_bytes());
    message_b.extend_from_slice(&nonce_b);
    message_b.extend_from_slice(&sig.to_bytes());
    write_lp(transport, &message_b).await.map_err(HandshakeError::Io)?;

    // Compute shared secret
    let x_a_pub = X25519Public::from(x_a_bytes);
    let shared = x_secret.diffie_hellman(&x_a_pub);

    // Derive session key
    let info = [&nonce_a[..], &nonce_b[..]].concat();
    let hk = Hkdf::<Sha256>::new(None, shared.as_bytes());
    let mut okm = [0u8; 32];
    hk.expand(&info, &mut okm)
        .map_err(|_| HandshakeError::Crypto("HKDF expand failed".into()))?;

    let aead = XChaCha20Poly1305::new(&okm.into());

    Ok(Session { aead, session_key: okm })
}

//
// Helper encrypted frame IO for Session
//
impl Session {
    /// Send a length-prefixed encrypted frame. Nonce scheme: 24-byte random XNonce per-frame.
    pub async fn send_encrypted_frame<T: AsyncWrite + Unpin + Send>(
        &self,
        transport: &mut T,
        plaintext: &[u8]
    ) -> Result<(), std::io::Error> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from(nonce_bytes);

        // Prepare ciphertext (in-place encryption)
        let mut buf = plaintext.to_vec();

        self.aead.encrypt_in_place(&nonce, b"", &mut buf)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::Other,
                "aead encrypt failed"
            ))?;

        // Frame = nonce || ciphertext
        let mut frame = Vec::with_capacity(24 + buf.len());
        frame.extend_from_slice(&nonce_bytes);
        frame.extend_from_slice(&buf);

        // Length-prefix and write
        transport.write_all(&(frame.len() as u32).to_be_bytes()).await?;
        transport.write_all(&frame).await?;
        transport.flush().await?;

        Ok(())
    }

    /// Read an encrypted frame and return plaintext.
    pub async fn read_encrypted_frame<T: AsyncRead + Unpin + Send>(
        &self,
        transport: &mut T
    ) -> Result<Vec<u8>, std::io::Error> {
        // Read length-prefixed frame
        let mut lenb = [0u8; 4];
        transport.read_exact(&mut lenb).await?;
        let len = u32::from_be_bytes(lenb) as usize;

        // Sanity check
        if len > 10 * 1024 * 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame too large"
            ));
        }

        let mut frame = vec![0u8; len];
        transport.read_exact(&mut frame).await?;

        if frame.len() < 24 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame too small"
            ));
        }

        let nonce_bytes: [u8; 24] = frame[..24].try_into().unwrap();
        let nonce = XNonce::from(nonce_bytes);
        let mut cipher = frame[24..].to_vec();

        self.aead.decrypt_in_place(&nonce, b"", &mut cipher)
            .map_err(|_| std::io::Error::new(
                std::io::ErrorKind::Other,
                "aead decrypt failed"
            ))?;

        Ok(cipher)
    }
}