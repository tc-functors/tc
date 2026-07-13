# Operations and testing

## Local development baseline

The root crate declares Rust edition 2024 and `rust-version = "1.95"` ([`Cargo.toml`](../Cargo.toml)); `make build` explicitly uses toolchain 1.95. Use that over the stale 1.90 reference in `AGENTS.md`.

| Intent | Command | Notes |
|---|---|---|
| Type-check workspace | `cargo check --workspace` | Broad, no test execution. |
| Lint | `cargo clippy --workspace --all-targets` | Recommended by `AGENTS.md`. |
| Rust tests | `make unit-test` | Runs `cargo test --quiet -j 2 -- --test-threads=2`. |
| Build debug binary | `make build` | Builds then copies the binary to the repository root. Do not commit generated output. |
| Format | `make fmt` | Uses `cargo +nightly fmt`; inspect/revert unrelated formatting changes. |

For a code change, run the narrowest relevant crate/test first, then the workspace checks appropriate to the requested confidence level. Repository guidance says to run all of the above before declaring a normal code change complete, but this documentation-only run did not execute them because no source behavior changed.

## Testing layers

### Rust tests

Unit and integration coverage is Cargo-based. Locate local tests next to code with `#[cfg(test)]`, then use a focused Cargo test before broad verification. The Makefile exposes `integration-test` as `cargo test --test integration_test --quiet`, but use it only after confirming the target is present and applicable.

### Topology tests (`tc test`)

Topology tests are deployed-system checks declared inside `topology.yml`. The tester resolves a topology and invokes AWS Lambda, Step Functions, EventBridge, or a route; it evaluates exact JSON (`matches`/`=`), JSON inclusion (`includes`/`contains`), or JSONPath conditions. See [`lib/tester/src/lib.rs`](../lib/tester/src/lib.rs) and [`examples/tests/topology.yml`](../examples/tests/topology.yml).

They are valuable behavioral checks but not substitutes for local Rust tests. They require credentials, valid deployed resources, and sandbox-aware names; do not run them merely as a local smoke test.

### Validation

`tc validate --entity mutation` is a narrowly scoped schema diagnostic: it builds generated GraphQL text from a composed topology and prints non-AWS diagnostics. It does not return a structured failure or validate general topology fields, and the current implementation assumes a mutation exists ([`lib/validator/src/lib.rs`](../lib/validator/src/lib.rs)). Use it as an aid while editing mutations, not as a deployment gate.

## Deployment/runbook notes

1. **Know the target.** Sandbox, profile/environment, namespace/FQN, and role derivation determine the AWS resource names touched. Inspect compose/resolve/diff output first.
2. **Prefer non-stable sandboxes for iteration.** Stable updates are guarded by deployer code. Bypass controls exist, but are policy exceptions rather than normal workflow.
3. **Keep build and deployment separate mentally.** Some `tc build` modes initialize AWS auth and can publish/sync artifacts. A successful Rust build does not demonstrate deployability.
4. **Treat delete/prune/freeze/route/tag/snapshot as operations.** Read their handlers before use; do not run them in broad loops or against examples without explicit scope.
5. **Never put credentials in topology/docs.** This wiki does not inspect secrets or local environment files. Configure AWS using the project’s expected profile/role mechanisms.

## CI and release behavior

[`ci.yml`](../.github/workflows/ci.yml) runs on pushed tags and builds release artifacts for Linux musl and macOS ARM, then uploads them. It is release-oriented; it does **not** run the local check/clippy/test set described above. The Makefile contains related release/cross-compilation targets; these can need target toolchains, static OpenSSL, and UPX.

[`openwiki-update.yml`](../.github/workflows/openwiki-update.yml) is an uncommitted local workflow in this checkout that schedules an OpenWiki update and opens a documentation PR. It uses repository secrets in CI; do not reproduce or inspect their values.

## When changing operational code

- Deployment sequence: test create and update separately, and consider deletion behavior. Notably, `create` provisions schedules, while the whole-topology `update_topology` path currently does not destructure or create schedules; use the explicit schedule entity update path when that is the intended operation and confirm the expected behavior before changing it.
- IAM roles/schedules: validate fully rendered ARNs and dependency order. Recent history fixed schedule role ARN construction, showing this area is sensitive.
- AWS SDK timeouts/auth: use focused provider-level tests or controlled environments where possible; do not make live-account assumptions from compilation alone.
- CLI flags: update examples/README when public behavior changes; README’s command list currently lags implementation.