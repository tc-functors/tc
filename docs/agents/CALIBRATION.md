# `tc` Calibration — the verified green baseline

This is the empirical ground truth an agent's changes are measured against,
captured by actually running the toolchain against `HEAD` (commit `2f54221f`,
`tc` 0.9.159) on **stable rustc 1.97.1** + **nightly 1.99.0**. A quality gate that
rejects the *existing* code trains agents to fight the style, so the gates in
`scripts/agent-check.sh` are calibrated to exactly this baseline. Re-run and update
this file when the baseline moves.

## Toolchain
- `Cargo.toml` pins `rust-version = "1.95"`, `edition = "2024"`. Build/clippy/test
  need stable **≥ 1.95**.
- **`fmt` requires nightly.** Every option in `rustfmt.toml` (`imports_granularity`,
  `imports_layout = Vertical`, `group_imports`, `wrap_comments`, `comment_width`,
  `format_code_in_doc_comments`) is *unstable*. On stable, rustfmt **ignores** them
  and reports the *opposite* formatting (it collapses the vertical imports). Always
  `cargo +nightly fmt`.

## What passes / fails at HEAD (verified)
| Command | Result | Notes |
|---|---|---|
| `cargo build` | ✅ passes | builds the `tc` binary in ~60s from clean |
| `cargo test -p composer` | ✅ passes | the suite CI actually runs (`.github/workflows/ci.yml`) |
| `cargo +nightly fmt --check` | ❌ 55 hunks / 14 files | pre-existing drift, not agent-introduced |
| `cargo clippy --workspace` | ❌ 2 errors + 153 warns | correctness lints in `compiler`; idioms elsewhere — see below |
| `cargo clippy --workspace --all-targets` | ❌ compile error | `kit` **test** target — see bug below |
| `cargo test --workspace` / `make unit-test` | ❌→✅ fixed (Phase 2) | did not compile at HEAD; **now passes** on this branch — see below |

> Update (this branch): the `cargo test --workspace` row was ❌ at the tagged HEAD
> baseline; Phase 2 fixed it (84 tests pass). The green baseline for CI is now
> `agent-check.sh` + `cargo test --workspace`.

**Therefore the green baseline is: `cargo build` + `cargo test -p composer`.** Do
not claim "workspace tests pass" — they do not compile yet.

## `kit` test target — FIXED on this branch (Phase 2)
Originally `lib/kit/src/io.rs:202` defined `pub fn sh(..)` under `#[cfg(not(test))]`
with **no `#[cfg(test)]` twin**, while `lib/kit/src/git.rs` and other `io.rs`
functions called `sh` unconditionally — so `kit`'s lib-test target failed
(`E0432`/`E0425`) and `cargo test --workspace` never compiled. CI never caught it
because it only ran `cargo test` in `lib/composer`.

Phase 2 fixed this:
- Added the `#[cfg(test)]` twin for `sh` (mirrors `file_exists`/`slurp`).
- Repaired the stale `differ` topology fixture (`fixture_topology`), which had
  drifted behind the resolved structs. Added the now-required fields: `Runtime`
  (`arch`, `enable_network`, `microvm`, `port`), `Build` (`dirs`, `include_deps`,
  `image_name`, `base_image_arn`, `build_role_arn`, `bucket`, `package_manager`),
  `Function` (`tasks`, `shared`), `Topology` (`root`, `concurrency`, `hooks`).

Result: **`cargo test --workspace` now compiles and passes (84 tests, 0 failures)**,
and `.github/workflows/agent-conformance.yml` runs it on every PR/push. This is a
worked example of the framework surfacing real pre-existing rot: fixing one masked
compile error exposed three stale-fixture test failures underneath it.

## clippy: HEAD is NOT clippy-clean → the gate is diff-scoped
`cargo clippy --workspace` **fails on HEAD**: the `compiler` crate has **2
error-level (correctness) lints** and the workspace emits **153 warnings across 33
lints**. The error-level findings (clippy's correctness category is deny-by-default)
are:

```
clippy::derived_hash_with_manual_eq        # lib/compiler/src/spec/... derives Hash + manual PartialEq
clippy::derive_ord_xor_partial_ord         # derives Ord + manual PartialOrd
clippy::non_canonical_partial_ord_impl     # (same site)
```

The 33 warning-level lints are the deliberate house idioms (see `STYLE.md` §10),
including: `needless_borrow`, `needless_borrows_for_generic_args`,
`redundant_field_names`, `redundant_pattern_matching`, `manual_find`, `manual_map`,
`manual_unwrap_or`, `manual_unwrap_or_default`, `manual_pattern_char_comparison`,
`iter_nth_zero`, `collapsible_if`, `single_match`, `if_same_then_else`,
`let_and_return`, `needless_late_init`, `needless_return`, `redundant_closure`,
`op_ref`, `ptr_eq`, `get_first`, `comparison_to_empty`, `println_empty_string`,
`useless_conversion`, `useless_borrows_in_formatting`, `useless_format`,
`unwrap_or_default`, `unnecessary_to_owned`, `unnecessary_mut_passed`,
`single_char_add_str`, `write_with_newline`, `new_without_default`.

**Because HEAD is dirty, a static allow-list is the wrong tool** — it would force us
to permanently *allow* real correctness lints. Instead `scripts/agent-check.sh`
runs clippy with `--cap-lints=warn` (so it analyzes the whole workspace despite the
2 errors) and then **fails only on findings whose primary span is in a file the
change touched**. New code is held to a clean bar; pre-existing debt is not
attributed to the agent. The `single_match`/idiom findings a change happens to touch
are the one soft spot — the later adversarial-review layer judges positive
style-match, which a deterministic lint cannot.

Recommended maintainer cleanup (not agent work inside an unrelated task): fix the 2
correctness lints in `compiler`, then the gate can drop `--cap-lints=warn` and
enforce zero *new* correctness findings hard.

## fmt drift at HEAD (14 files)
`cargo +nightly fmt --check` flags 55 hunks in: `lib/compiler/src/{lib,spec}.rs`,
`lib/composer/src/{lib,topology}.rs`, `lib/composer/src/aws/function/{build.rs,
runtime/lambda.rs}`, `lib/deployer/src/aws/mutation.rs`,
`lib/inspector/src/{color,detail,lib,tree}.rs`, `lib/provider/src/aws/iam.rs`,
`src/{lib,main}.rs`. Because HEAD is not fmt-clean, the gate checks **only the files
an agent changed**, not the whole tree — do not reformat unrelated files. (A
one-time `cargo +nightly fmt` normalization of the tree is a candidate cleanup, but
that is a maintainer decision, not something an agent should do inside an unrelated
task.)

## How to re-calibrate
```sh
rustup toolchain install nightly --component rustfmt   # once
cargo build
cargo test -p composer
cargo +nightly fmt --check              # expect the 14-file drift above
cargo clippy --workspace -- --cap-lints=warn 2>&1 | grep -oE 'clippy::[a-z_]+' | sort -u
```
Update the tables above if results change, and keep `scripts/agent-check.sh` in sync
with the allow-list.
