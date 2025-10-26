# OpenShare

> Secure, local-first P2P file transfer with end-to-end encryption

OpenShare is a high-performance file transfer application that prioritizes security and privacy. Files are transferred directly between devices on your local network with strong cryptographic guarantees, requiring only a minimal server for device registration.

## ✨ Features

- 🔒 **End-to-End Encryption** - All transfers use XChaCha20-Poly1305 AEAD
- 🤝 **Mutual Authentication** - Ed25519 signatures prevent man-in-the-middle attacks
- 📦 **Content-Addressed Storage** - Automatic deduplication via SHA-256 chunk hashing
- 🔍 **Zero-Configuration Discovery** - Find peers automatically via mDNS
- 📝 **Cryptographically Signed Manifests** - Verify file integrity and authenticity
- ⚡ **Fast Chunked Transfers** - 256 KiB chunks with parallel processing
- 🔄 **Resumable Transfers** - Continue interrupted transfers seamlessly
- 🌐 **LAN-First Architecture** - Keep your data on your network

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone git@github.com:OpenTabCommunity/open-share-client-rust.git
cd openshare/mdns-tool

# Build release binary
cargo build --release

# Optional: Install system-wide
sudo cp target/release/openshare /usr/local/bin/
```

### Basic Usage

```bash
# Initialize your device
openshare init --device-id laptop --account alice@example.com

# Start listening for transfers
openshare listen --port 9876

# Send a file (from another terminal/device)
openshare send --file document.pdf --peer 192.168.1.100:9876
```

## 📖 Documentation

- [Installation Guide](docs/INSTALLATION.md)
- [User Guide](docs/USER_GUIDE.md)
- [Architecture](ARCHITECTURE.md)
- [API Documentation](docs/API.md)
- [Security Model](docs/SECURITY.md)

## 🏗️ Architecture

OpenShare uses a modular architecture with separate concerns:

```
┌─────────────────────────────────────────────────────────┐
│                     Application Layer                    │
│                    (CLI / GUI / API)                     │
├─────────────────────────────────────────────────────────┤
│                      Core Library                        │
│  ┌──────────────┬──────────────┬──────────────────────┐ │
│  │   Identity   │   Manifest   │      Handshake       │ │
│  │  (Ed25519)   │  (Signing)   │  (X25519 + HKDF)     │ │
│  └──────────────┴──────────────┴──────────────────────┘ │
├─────────────────────────────────────────────────────────┤
│               Transport & Storage Layer                  │
│  ┌──────────────┬──────────────┬──────────────────────┐ │
│  │     TCP      │   Storage    │        mDNS          │ │
│  │  (QUIC soon) │  (Chunks)    │     (Discovery)      │ │
│  └──────────────┴──────────────┴──────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Security

- **Handshake Protocol**: Ephemeral X25519 keys + Ed25519 signatures
- **Encryption**: XChaCha20-Poly1305 authenticated encryption
- **Key Derivation**: HKDF-SHA256 for session keys
- **Integrity**: Per-chunk SHA-256 verification


## 📦 Project Structure

```
mdns-tool/
├── crates/
│   ├── openshare-core/     # Core cryptography and transfer logic
│   ├── storage/            # Content-addressed storage
│   ├── mdns-core/          # mDNS service discovery
│   ├── openshare-cli/      # Command-line interface
│   └── transport-quic/     # QUIC transport (future)
├── docs/                   # Documentation
├── examples/               # Usage examples
└── tests/                  # Integration tests
```

## 🔧 Development

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=debug cargo run -- --help
```

### Running Tests

```bash
# Unit tests
cargo test --workspace

# Integration tests
./test_transfer.sh

# Specific crate tests
cargo test -p openshare-core
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) first.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test --workspace`)
6. Run formatter (`cargo fmt`)
7. Run linter (`cargo clippy`)
8. Commit your changes (`git commit -m 'Add amazing feature'`)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

## 📋 Roadmap

- [x] Core cryptographic handshake
- [x] TCP transport layer
- [x] mDNS discovery
- [x] Content-addressed storage
- [x] CLI interface
- [ ] QUIC transport
- [ ] GUI application (desktop)
- [ ] Mobile apps (iOS/Android)
- [ ] Certificate authority integration
- [ ] CRL (Certificate Revocation List)
- [ ] NAT traversal / relay support
- [ ] Progress indicators
- [ ] Bandwidth throttling
- [ ] Multiple file transfers
- [ ] Directory sync

## 📊 Performance

Benchmarks on Intel i7-9750H @ 2.60GHz:

| Operation | Performance |
|-----------|-------------|
| Handshake Latency | < 10ms |
| Throughput (LAN) | ~800 Mbps |
| Chunk Processing | ~50k chunks/sec |
| Discovery Time | < 3 seconds |

## 🐛 Known Issues

- mDNS discovery may not work on some enterprise networks with multicast filtering
- Large files (>10GB) require significant memory for chunk tracking
- Windows firewall may block incoming connections by default

See [Issues](https://github.com/yourusername/openshare/issues) for full list.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [ed25519-dalek](https://github.com/dalek-cryptography/ed25519-dalek) for signatures
- Uses [x25519-dalek](https://github.com/dalek-cryptography/x25519-dalek) for key exchange
- Uses [chacha20poly1305](https://github.com/RustCrypto/AEADs) for encryption
- mDNS via [mdns-sd](https://github.com/keepsimple1/mdns-sd)

## 📞 Support

- 📧 Email: support@openshare.example.com
- 📖 Documentation: [docs.openshare.example.com](https://docs.openshare.example.com)

## ⚠️ Disclaimer

OpenShare is currently in **beta**. While the cryptographic primitives are well-tested, the overall system should be audited before production use. For production deployments:

- Use hardware security modules (HSM) for key storage
- Implement proper certificate authority
- Add comprehensive audit logging
- Conduct professional security audit
- Implement rate limiting and abuse prevention

---

Made with ❤️ by the OpenShare Team
