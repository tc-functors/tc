# Technology Stack

## Language & Runtime

- **Primary Language**: Rust (edition 2024, minimum version 1.90)
- **Async Runtime**: Tokio with multi-threaded runtime
- **Toolchain**: RUSTUP_TOOLCHAIN=1.90

## Build System

- **Build Tool**: Cargo (Rust's package manager)
- **Workspace**: Multi-crate workspace with 19 library crates in `lib/`
- **Make**: Makefile for common build tasks and cross-compilation

## Core Dependencies

- **CLI Framework**: clap v4.1 with derive features
- **Serialization**: serde, serde_json, serde_derive
- **Logging**: tracing, tracing-subscriber with env-filter
- **TLS**: openssl with vendored feature for static linking
- **Terminal UI**: colored, tabled, inquire
- **Testing**: mockall for mocking

## Common Commands

### Development
```bash
# Build debug version (creates ./tc binary)
make build

# Run unit tests (parallel execution limited to 2 threads)
make unit-test

# Run integration tests
make integration-test

# Format code (requires nightly toolchain)
make fmt

# Clean build artifacts
make clean
```

### Direct Cargo Commands
```bash
# Build debug
cargo build

# Build release (optimized for size with LTO)
cargo build --release

# Run tests
cargo test --quiet -j 2 -- --test-threads=2
```

### Cross-Compilation Targets
```bash
# Linux x86_64 (musl, static binary with UPX compression)
make x86_64-linux

# macOS x86_64
make x86_64-apple

# macOS ARM64 (aarch64)
make aarch64-apple
```

## Release Profile

Optimized for minimal binary size:
- LTO: fat (link-time optimization)
- Optimization level: 'z' (size)
- Stripped symbols
- Single codegen unit
- Panic: abort

## Cross-Compilation Notes

- Linux builds use musl for static linking
- Requires PKG_CONFIG_ALLOW_CROSS=1 and OPENSSL_STATIC=true
- macOS cross-compilation uses osxcross toolchain
- UPX compression applied to Linux binaries
