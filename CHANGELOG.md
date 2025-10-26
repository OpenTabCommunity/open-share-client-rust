# Changelog

All notable changes to OpenShare will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-10-26

### 🎉 Initial Release

First public release of OpenShare - a secure, local-first P2P file transfer application.

### Added

#### Core Features
- **Cryptographic Handshake**: Implemented secure handshake protocol using ephemeral X25519 keys and Ed25519 signatures
- **End-to-End Encryption**: XChaCha20-Poly1305 AEAD for all file transfers
- **Identity Management**: Ed25519 keypair generation and storage
- **Content-Addressed Storage**: SHA-256 based chunk storage with automatic deduplication
- **File Manifests**: Cryptographically signed file manifests for integrity verification
- **Chunk-Based Transfers**: 256 KiB default chunk size with configurable options

#### Discovery & Networking
- **mDNS Discovery**: Zero-configuration peer discovery on local networks
- **Service Announcement**: Broadcast device availability with metadata
- **TCP Transport**: Reliable TCP-based transfer protocol
- **Account-Based Filtering**: Discover only devices from the same account

#### CLI Commands
- `init` - Initialize device identity and configuration
- `info` - Display device information and fingerprint
- `announce` - Announce device on local network
- `discover` - Find available peers
- `send` - Send files to remote peers
- `listen` - Accept incoming file transfers
- `create-manifest` - Generate signed file manifest
- `verify-manifest` - Verify manifest signatures

#### Security
- HKDF-SHA256 for session key derivation
- Per-chunk integrity verification
- Signature verification for all manifests
- Forward secrecy through ephemeral keys
- Protection against replay attacks via nonces

#### Developer Tools
- Comprehensive test suite
- Integration test scripts
- Detailed logging with `tracing`
- Modular crate architecture

### Architecture

#### Crates
- `openshare-core` - Core cryptography and transfer logic
- `storage` - Content-addressed chunk storage
- `mdns-core` - mDNS service discovery
- `openshare-cli` - Command-line interface
- `transport-quic` - Placeholder for future QUIC support

### Known Limitations

- No GUI yet (CLI only)
- No resume capability for interrupted transfers
- No QUIC transport (TCP only)
- No certificate authority integration
- No CRL (Certificate Revocation List) support
- Large files may require significant memory
- mDNS may not work on all networks

### Technical Details

- Minimum Rust version: 1.70
- Supported platforms: Linux, macOS, Windows
- Default chunk size: 256 KiB
- Default port: 9876
- Maximum frame size: 10 MB

### Dependencies

- `tokio` 1.x - Async runtime
- `ed25519-dalek` 2.x - Ed25519 signatures
- `x25519-dalek` 2.x - X25519 key exchange
- `chacha20poly1305` 0.10 - AEAD encryption
- `mdns-sd` 0.11 - mDNS implementation
- `clap` 4.x - CLI parsing
- `serde` 1.x - Serialization
- `bincode` 1.x - Binary serialization

### Performance

Initial benchmarks (Intel i7-9750H):
- Handshake latency: < 10ms
- LAN throughput: ~800 Mbps
- Chunk processing: ~50k chunks/sec
- Discovery time: < 3 seconds

---

## [Unreleased]

## [0.1.0] - 2025-10-26

## [0.1.0] - 2025-10-26

### Planned Features

- QUIC transport support
- Resume interrupted transfers
- GUI application
- Mobile apps (iOS/Android)
- Certificate authority integration
- CRL support
- NAT traversal / relay
- Progress indicators
- Bandwidth throttling
- Multiple concurrent transfers
- Directory synchronization

---
[0.1.0]: https://github.com/OpenTabCommunity/open-share-client-rust/releases/tag/v0.1.0
