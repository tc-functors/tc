# AGENTS.md

## Cursor Cloud specific instructions

### Overview

tc (Topology Composer) is a Rust CLI tool for defining and deploying serverless topologies. It is a single binary built from a Cargo workspace with 19 library crates under `lib/`.

### Build & Run

- **Rust 1.90+** is required (edition 2024). The default VM toolchain is too old; the update script installs 1.90 automatically.
- Build: `RUSTUP_TOOLCHAIN=1.90 cargo build` (or `make build`, though the Makefile's `cp` step assumes `TARGET_DIR=$(HOME)/.cargo/target` which may not match `cargo`'s actual target directory — the binary is at `target/debug/tc`).
- The binary is at `./target/debug/tc` after a debug build.
- OpenSSL is compiled from source via the `vendored` feature; no system `libssl-dev` is needed, but `perl` must be available.

### Testing

- Unit tests: `cargo test --quiet -j 2 -- --test-threads=2` (see `make unit-test`).
- Integration tests: `cargo test --test integration_test --quiet` (requires AWS credentials).
- Format check: `cargo +nightly fmt --check` (requires nightly toolchain installed).

### Offline commands (no AWS needed)

These tc subcommands work without AWS credentials and are useful for smoke-testing:
- `tc compile` — compile a topology spec from a `topology.yml`
- `tc compose` — compose a topology into provider-specific output
- `tc visualize` — generate an HTML flow diagram
- `tc version` — print version

Run from a directory containing a `topology.yml`, e.g.: `cd examples/composition/route-function && ./target/debug/tc compile`

### AWS-dependent commands

`create`, `update`, `delete`, `invoke`, `list`, `resolve`, `deploy`, `snapshot`, `route`, `freeze`, `unfreeze`, `prune` all require valid AWS credentials configured in the environment.

### Known issues on main

- The `Makefile` `build` target copies the binary from `$(HOME)/.cargo/target/debug/tc`, but Cargo's actual target directory defaults to `./target/`. The `cp` step may fail; use `cargo build` directly and reference `./target/debug/tc`.
- `cargo clippy` reports pre-existing errors in `compiler` crate; the project does not gate on clippy in CI.
