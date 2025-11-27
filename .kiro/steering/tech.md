# Technology Stack

## Language & Build System

- **Language**: Rust (edition 2024, minimum version 1.90)
- **Build System**: Cargo workspace with multiple library crates
- **Package Manager**: Cargo
- **Toolchain**: RUSTUP_TOOLCHAIN=1.90

## Core Dependencies

- **CLI**: clap v4.1 (with derive features)
- **Async Runtime**: tokio v1 (multi-threaded)
- **Serialization**: serde v1.0, serde_json v1.0
- **Logging/Tracing**: tracing v0.1, tracing-subscriber v0.3, log v0.4
- **Security**: openssl v0.10 (with vendored features)
- **UI/Display**: colored v2.0, tabled v0.10, inquire v0.7.5

## Common Commands

### Development
```bash
# Build debug version
make build
# or
cargo build

# Run with specific toolchain
RUSTUP_TOOLCHAIN=1.90 cargo build

# Format code (uses nightly for rustfmt)
make fmt
# or
cargo +nightly fmt
```

### Testing
```bash
# Run unit tests
make unit-test
# or
cargo test --quiet -j 2 -- --test-threads=2

# Run integration tests
make integration-test
# or
cargo test --test integration_test --quiet
```

### Release Builds
```bash
# Build optimized release
cargo build --release

# Cross-compile for x86_64 Linux (musl)
make x86_64-linux

# Build for Apple Silicon
make aarch64-apple

# Build for x86_64 macOS
make x86_64-apple
```

### Project Commands
```bash
# Clean build artifacts
make clean

# Generate documentation
make docs
```

## Code Formatting

Uses rustfmt with custom configuration (rustfmt.toml):
- Comment width: 100 characters
- Vertical imports layout with crate-level granularity
- One group for imports
- Format code in doc comments
- Wrap comments enabled

## Release Profile

Optimized for size and performance:
- opt-level: 'z' (optimize for size)
- LTO: fat (full link-time optimization)
- codegen-units: 1 (maximum optimization)
- strip: true (remove debug symbols)
- panic: abort (smaller binary)
