# Contributing to OpenShare

Thank you for considering contributing to OpenShare! We welcome contributions from everyone.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Testing](#testing)
- [Documentation](#documentation)

## 📜 Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code.

### Our Standards

- Be respectful and inclusive
- Welcome newcomers and help them get started
- Focus on what is best for the community
- Show empathy towards other community members

## 🤝 How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

When filing a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the issue
- **Expected vs actual behavior**
- **Environment details** (OS, Rust version, etc.)
- **Logs** (with `RUST_LOG=debug`)
- **Screenshots** if applicable

### Suggesting Features

Feature requests are welcome! Please:

- Use a clear and descriptive title
- Provide detailed description of the feature
- Explain why this feature would be useful
- Consider implementation complexity

### Code Contributions

We accept pull requests for:

- Bug fixes
- New features
- Performance improvements
- Documentation improvements
- Test coverage improvements

## 🛠️ Development Setup

### Prerequisites

```bash
# Install Rust (1.70 or higher)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
rustup component add rustfmt clippy
```

### Clone and Build

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/openshare.git
cd openshare/mdns-tool

# Add upstream remote
git remote add upstream https://github.com/ORIGINAL_OWNER/openshare.git

# Build
cargo build

# Run tests
cargo test --workspace
```

### Development Tools

```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Run with logging
RUST_LOG=debug cargo run -- <command>

# Build documentation
cargo doc --open --no-deps
```

## 📝 Coding Standards

### Rust Style Guide

We follow the official [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/).

Key points:

- Use `rustfmt` for formatting (runs automatically)
- Use `clippy` for linting
- Write idiomatic Rust code
- Prefer explicit over implicit
- Use descriptive variable names

### Code Organization

```rust
// Module order
mod config;      // Configuration
mod types;       // Type definitions
mod error;       // Error types
mod core;        // Core logic
mod utils;       // Utilities

// Import order
use std::...;               // Standard library
use external_crate::...;    // External crates
use crate::...;            // Internal modules
```

### Documentation

All public APIs must be documented:

```rust
/// Brief description of function.
///
/// More detailed description with usage examples.
///
/// # Arguments
///
/// * `param1` - Description of param1
/// * `param2` - Description of param2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function returns an error
///
/// # Examples
///
/// ```
/// use openshare_core::function;
/// let result = function(arg1, arg2)?;
/// ```
pub fn function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // implementation
}
```

### Error Handling

- Use `anyhow::Result` for application errors
- Use `thiserror` for library errors
- Provide context with `.context()`
- Don't use `.unwrap()` in library code

```rust
use anyhow::{Context, Result};

pub fn read_config(path: &Path) -> Result<Config> {
    let data = fs::read_to_string(path)
        .context("Failed to read config file")?;
    
    serde_json::from_str(&data)
        .context("Failed to parse config JSON")
}
```

### Security Considerations

- Never log sensitive data (keys, passwords)
- Use zeroize for sensitive memory
- Validate all inputs
- Use constant-time comparisons for secrets
- Document security assumptions

```rust
use zeroize::Zeroize;

let mut secret = get_secret();
// use secret
secret.zeroize(); // Clear from memory
```

## 💬 Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/).

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### Examples

```bash
feat(cli): add progress bar for file transfers

- Added progress indicator for large files
- Shows percentage and ETA
- Configurable with --no-progress flag

Closes #123

fix(handshake): handle connection timeout

Previously, handshake would hang indefinitely on timeout.
Now properly returns error after 30 seconds.

Fixes #456

docs(readme): update installation instructions

Added section for Windows-specific setup
```

## 🔄 Pull Request Process

### Before Submitting

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write clear, concise code
   - Add tests for new functionality
   - Update documentation

3. **Test thoroughly**
   ```bash
   cargo test --workspace
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

4. **Update CHANGELOG.md**
   - Add entry under `[Unreleased]`
   - Follow existing format

5. **Commit your changes**
   ```bash
   git add .
   git commit -m "feat: add amazing feature"
   ```

### Submitting PR

1. **Push to your fork**
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create Pull Request**
   - Use descriptive title
   - Reference related issues
   - Describe changes in detail
   - Add screenshots if UI changes

3. **PR Template**
   ```markdown
   ## Description
   Brief description of changes

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Comments added for complex code
   - [ ] Documentation updated
   - [ ] No new warnings
   - [ ] Tests added for new features
   - [ ] CHANGELOG.md updated
   ```

### Review Process

- Maintainers will review your PR
- Address feedback promptly
- Keep PR focused and small
- Be patient and respectful

### After Approval

- Squash commits if requested
- PR will be merged by maintainers
- Delete your feature branch

## 🧪 Testing

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_value);
    }

    #[tokio::test]
    async fn test_async_feature() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p openshare-core

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests
./test_transfer.sh
```

## 📚 Documentation

### Code Documentation

- Document all public APIs
- Include examples in doc comments
- Run `cargo doc` to verify

### User Documentation

Update relevant docs in `docs/`:

- `USER_GUIDE.md` - User-facing features
- `API.md` - API documentation
- `SECURITY.md` - Security considerations

### README Updates

Update main README.md for:

- New features
- Changed APIs
- Updated requirements

## 🎯 Areas Needing Help

We especially welcome contributions in:

- 🧪 **Testing**: Add more test coverage
- 📝 **Documentation**: Improve guides and examples
- 🐛 **Bug Fixes**: Fix open issues
- 🚀 **Performance**: Optimize hot paths
- 🔒 **Security**: Security reviews and hardening
- 🌐 **Accessibility**: Improve usability
- 🎨 **UI/UX**: GUI development

## 💡 Good First Issues

Look for issues labeled:
- `good-first-issue`
- `help-wanted`
- `documentation`

## 📞 Getting Help

- 📧 Email: dev@openshare.example.com
- 🐛 [GitHub Issues](https://github.com/OpenTabCommunity/open-share-client-rust/issues)

## 🙏 Thank You!

Your contributions make OpenShare better for everyone. We appreciate your time and effort!
