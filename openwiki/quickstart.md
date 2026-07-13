# tc: engineering quickstart

`tc` (Topology Composer) is a Rust CLI and workspace for describing serverless systems as topology graphs, converting those definitions into an AWS-oriented resource model, and building, deploying, invoking, and testing the resulting components. The public framing is a **Cloud Functor**: a namespaced, sandboxed, versioned topology.

This wiki is an engineering map of the local codebase—not a replacement for the external product documentation linked from [`README.md`](../README.md).

## Start here

1. **Read [architecture overview](architecture/overview.md)** for the compile → compose → resolve → deploy pipeline and crate boundaries.
2. **Read [topology model](domain/topology-model.md)** before changing a `topology.yml`, entity semantics, names, or recursive composition.
3. **Use [topology lifecycle](workflows/topology-lifecycle.md)** for command-level workflows and example locations.
4. **Follow [operations and testing](operations-and-testing.md)** before running cloud-affecting commands or choosing checks.
5. **Use [source map](source-map.md)** to locate the owning code for a change and understand recent evolution.

## Repository orientation

- Root binary crate: [`src/main.rs`](../src/main.rs) defines the Clap surface; [`src/lib.rs`](../src/lib.rs) coordinates cross-crate workflows.
- Workspace libraries: [`Cargo.toml`](../Cargo.toml) lists 20 crates in `lib/`, including compiler, composer, resolver, builder, deployer, provider, tester, and validator.
- Executable examples: [`examples/`](../examples/) covers composition, function runtimes, routes, states, GraphQL mutations, pages, stores, patterns, and topology tests.
- AWS behavior lives largely in [`lib/provider`](../lib/provider/), [`lib/composer`](../lib/composer/), [`lib/resolver`](../lib/resolver/), and [`lib/deployer`](../lib/deployer/).

## Safe local starting commands

The root manifest requires **Rust 1.95** (`rust-version = "1.95"`); the Makefile also pins `RUSTUP_TOOLCHAIN=1.95` for `make build`. `AGENTS.md` currently says 1.90, so treat the manifest and Makefile as the current source of truth.

```sh
cargo check --workspace
cargo clippy --workspace --all-targets
make unit-test
make build
```

`make fmt` uses nightly rustfmt. Do not run `create`, `update`, `delete`, release/tagging, or other AWS-affecting commands without an explicit operational request and suitable AWS setup. See [operations and testing](operations-and-testing.md).

## What to preserve when changing code

- A topology field is usually cross-cutting: compiler spec → composer model → resolver/deployer/provider behavior, often with examples and tests.
- Naming, FQN, sandbox, and role changes can target existing cloud resources. Start with the model and lifecycle pages; do not treat them as cosmetic changes.
- Deployment ordering in the deployer is intentional dependency ordering, not a collection of interchangeable calls.
- Generated wiki pages live here. `openwiki/INSTRUCTIONS.md` is the user-authored brief and is not generated or maintained by this run.

## Backlog

- **Example support/maturity matrix** — [`examples/composition/status.org`](../examples/composition/status.org): existing examples and the status file do not clearly establish which combinations are currently supported; deferred because it needs execution against intended environments.
- **Provider strategy** — [`lib/composer/src/aws/`](../lib/composer/src/aws/) and [`lib/provider/src/aws/`](../lib/provider/src/aws/): source is AWS-specific despite provider-agnostic language in the README; deferred because roadmap intent is not represented in repository evidence.
- **Full CLI option reference** — [`src/main.rs`](../src/main.rs): the CLI is broad and evolves quickly; this initial wiki documents workflows rather than copying every argument.