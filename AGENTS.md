# AGENTS.md — Agent guide for `tc` (Topology Composer)

This file is the **canonical, harness-agnostic entry point** for AI coding agents
working in this repository. It is read directly by Cursor, OpenAI Codex/codex-cli,
Aider, Gemini CLI, and others; thin adapters point the remaining harnesses here
(`CLAUDE.md`, `.cursor/rules/`, `.kiro/steering/`, `.github/copilot-instructions.md`).

> **Why this exists.** Contributions of LLM-generated code were historically
> rejected because they ignored this codebase's conventions, data structures, and
> idioms. The goal of these docs + the gates in `scripts/agent-check.sh` is to make
> agent-generated code match or exceed the quality of the existing code, so that
> restriction can be lifted. **Conformance to the house style is not optional here —
> it is the acceptance criterion.**

## Read these before writing code
- `docs/agents/STYLE.md` — the prescriptive style guide. **The most important file.**
- `docs/agents/ARCHITECTURE.md` — the pipeline, crate map, and data model.
- `docs/agents/DSL.md` — the `topology.yml` / `function.yml` schema reference.
- `docs/agents/CALIBRATION.md` — what actually builds/tests/lints clean today, and why.
- `docs/agents/REVIEW.md` — the automated PR review layer (Cursor Bugbot rules).

## What this project is (one paragraph)
`tc` is a single Rust binary that turns a provider-agnostic `topology.yml` graph of
entities (functions, events, routes, mutations, queues, channels, states, pages,
stores) into deployed AWS serverless infrastructure. The pipeline is
`compile → compose → resolve → deploy`. It calls the AWS SDK directly (no
CloudFormation/CDK/Terraform) and AWS is intentionally hard-coded (no provider
trait). See `docs/agents/ARCHITECTURE.md`.

## Build / test / lint — the calibrated reality
Rust toolchain: **stable ≥ 1.95** (edition 2024) for build/clippy/test; **nightly**
is required for `fmt` (the entire `rustfmt.toml` uses unstable options).

```sh
cargo build                    # builds the `tc` binary — the primary green signal
cargo test -p composer         # the suite CI historically ran
cargo test --workspace         # full suite — green as of Phase 2 (was broken before)
cargo +nightly fmt --check     # formatting gate — REQUIRES nightly
cargo clippy --workspace       # lint (see the allow-list in CALIBRATION.md)
./scripts/agent-check.sh       # the one command to run before proposing changes
```

Known caveats (documented in `docs/agents/CALIBRATION.md`):
- `cargo test --workspace` / `make unit-test` were **broken at the tagged HEAD**
  (a missing `#[cfg(test)]` `sh` twin in `kit` plus stale `differ` fixtures). Phase 2
  **fixed** this — the full suite now passes and `.github/workflows/agent-conformance.yml`
  runs it on every PR/push.
- `cargo +nightly fmt --check` reports pre-existing drift in 14 files. Scope your
  formatting to files you changed; do not reformat unrelated files.
- `cargo clippy --workspace` is not globally clean (deliberate idioms + 2 pre-existing
  correctness lints in `compiler`); the gate is **line-scoped** so it only judges the
  lines your change adds.

## The ten non-negotiables (full detail in STYLE.md)
1. Prefer small, terse **free functions** over methods/`impl` blocks. Logic takes
   `&Topology`/`&Auth`/`&str`, returns owned `String`/`Vec`/`HashMap`.
2. Use the **`kit`** crate for everything: `use kit as u;` and `use kit::*;`.
   Reach for `s()`, `empty()`, `s!`, `v!`, `maybe_*`, `nth`, `split*` before writing
   inline logic.
3. **No `anyhow`, no `thiserror`, no custom error enums.** The CLI posture is
   fail-fast: `.unwrap()`, `.expect("fail")`, `panic!`. The *only* exception is the
   caching/concurrency "disciplined layer" (`kit::memo`, resolver concurrency,
   `differ`) — heavily documented, gracefully degrading. Match that only there.
4. Data structs: `#[derive(Serialize, Deserialize, Clone, Debug)]`, all fields
   `pub`, `HashMap<String, _>`-of-everything, `Option<T>` + `#[serde(default)]`.
5. Enums for closed sets get a `Kind` suffix and honest `todo!()` arms for
   unimplemented variants.
6. Imports: crate-granular, **vertical**, one group, braces even for one item
   (enforced by `rustfmt.toml`; needs nightly).
7. Tests: inline `#[cfg(test)] mod tests`; mock leaf IO with a
   `#[cfg(test)]`/`#[cfg(not(test))]` twin function, not a trait/mock object.
8. Concurrency is always **bounded** (`futures::stream … buffer_unordered(n)`),
   with an env-configurable cap clamped to a documented `const` ceiling.
9. Respect the `TopologySpec` (unresolved, `Option`-heavy) vs `Topology` (resolved,
   non-optional `HashMap`s) boundary. Don't blur the compile/compose/resolve stages.
10. Configuration is via `TC_*` env vars. New knobs follow the existing pattern.

## Boundaries
- Do **not** add dependencies without discussing it (especially `anyhow`/`thiserror`
  — they are deliberately absent).
- Do **not** reformat or "modernize" code that already follows the house style
  (e.g. do not rewrite `x.len() > 0` to `!x.is_empty()`, do not remove `.clone()`s,
  do not convert free functions into methods). The clippy allow-list in
  `CALIBRATION.md` lists the idioms that are intentional.
- Do **not** introduce a provider abstraction/trait unless explicitly asked — AWS is
  hard-coded by design.
- Do **not** move responsibilities between crates without discussing the design.
- Keep diffs narrowly scoped. Update `examples/` when user-facing behavior changes.

## Before you say "done"
1. `./scripts/agent-check.sh` passes (fmt on changed files, clippy allow-list,
   composer tests, convention greps).
2. `cargo build` succeeds.
3. `git diff` contains only lines that belong to the task.
4. New behavior has tests in the house style.
