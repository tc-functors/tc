# Bugbot review rules for `tc` (Topology Composer)

You are reviewing a PR against this repository's **established house style**. Here,
conformance to the existing conventions is an **acceptance criterion**, not a
nice-to-have: the project historically rejected LLM-generated code because it ignored
these conventions. Judge whether a change reads like the **original author wrote it**,
not whether it follows generic Rust "best practices." When generic Rust advice
conflicts with a rule below, the rule below wins.

Canonical references (read for depth): `/AGENTS.md` and `/docs/agents/STYLE.md`
(especially **§10, "deliberate idioms — match, don't fix"**), `/docs/agents/ARCHITECTURE.md`.

## Scope — which rules apply where
The BLOCK rules below concern the **Rust codebase** (`lib/**`, `src/**`). Auxiliary
tooling — Python/shell/YAML under `scripts/**`, `eval/**`, `.github/**` — is **not**
Rust: do **not** apply the Rust-specific rules to it (the `#[cfg(test)]` twin,
free-functions-over-methods, the derive quartet, `kit` usage, vertical `use` blocks).
Judge that tooling on its own terms — stdlib-only where practical, fail-safe, no
needless dependencies, no secrets. (For example, a Python helper that shells out to
`git`/`aws` is fine and does **not** need a Rust `#[cfg(test)]` twin.)

## BLOCK (raise a blocking Bug) when a changed **Rust** file does any of these
1. Adds `anyhow` or `thiserror` to any `Cargo.toml`, or adds `use anyhow`/`use
   thiserror`. This codebase is **deliberately free of them**. Error handling is
   fail-fast (`.unwrap()`, `.expect("fail")`, `panic!`, `std::process::exit(1)`).
2. Introduces a custom error `enum`, or a `Result`-returning API, in ordinary code.
   (Exception: the "disciplined layer" — `kit::memo`, resolver concurrency, `differ`
   — legitimately uses graceful degradation; see below.)
3. Introduces a provider trait or any cloud-provider abstraction. **AWS is hard-coded
   by design**; do not accept a "make it multi-cloud" abstraction unless the PR
   explicitly says the maintainer asked for it.
4. Adds unbounded concurrency — `join_all(` / `futures::future::join_all`. Concurrency
   must be **bounded**: `futures::stream … buffer_unordered(n)` with an env-configurable
   cap clamped to a documented `const` ceiling.
5. Adds a new leaf IO function in `lib/kit` (filesystem/process/network) **without a
   `#[cfg(test)]` twin**. Tests mock leaf IO via a `#[cfg(test)]`/`#[cfg(not(test))]`
   function pair (see `file_exists`, `slurp`, `sh` in `lib/kit/src/io.rs`), not traits.
6. Reformats or rewrites lines it did not functionally change (import reordering,
   whitespace churn, "modernizing") in files unrelated to the PR's purpose.
7. Converts an existing free function into a method, or adds a method to a domain data
   struct to hold logic. Logic is written as **free functions** taking
   `&Topology`/`&Auth`/`&str` and returning owned values.
8. Defines a serialized data struct **without** `#[derive(Serialize, Deserialize,
   Clone, Debug)]`, or with non-`pub` fields. These are transparent `pub` data records.
9. Blurs the **`TopologySpec`** (unresolved, `Option`-heavy) vs **`Topology`**
   (resolved, non-optional `HashMap`s) boundary — e.g. resolving during compile, or
   adding `Option` churn to the resolved type.
10. Adds a new third-party dependency without a clear, stated need.
11. Adds a function with **more than 4 parameters** (`clippy.toml` enforces
    `too-many-arguments-threshold = 4`). Thread wider state via `&Context`/`&Auth` or a
    small `pub` struct. Grandfathered: only flag a signature the change adds or widens,
    never a pre-existing wide one.
12. Puts new behavior/unit tests in a fresh inline `#[cfg(test)] mod tests` when they
    only exercise the crate's **public** surface. Those belong in the sibling
    `lib/<crate>/tests/<area>_test.rs` (the per-entity files already scaffolded in
    `lib/composer/tests/`). Inline `#[cfg(test)]` is correct **only** for the leaf-IO
    cfg-twin (#5) or a test that needs a crate-private item.

For new **caching / concurrency** code specifically, DO expect the disciplined-layer
style and block if it is missing: bounded concurrency, an env-configurable cap with a
`const` ceiling, poisoned-lock-as-cache-miss graceful degradation, and doc comments
justifying cache-key sufficiency (mirror `lib/kit/src/memo.rs` and
`lib/resolver/src/lib.rs`).

## DO NOT FLAG — these are intentional idioms (flagging them is noise)
Per `docs/agents/STYLE.md` §10, the following are deliberate and correct here. Do not
raise Bugs, and do not suggest "improvements," for any of them:
- Liberal `.clone()`, including `xs.clone().into_iter()` and `parts.clone().first()`.
- `if x.len() > 0` (do **not** suggest `!x.is_empty()`).
- `match opt { Some(_) => true, None => false }` (do **not** suggest `.is_some()`).
- `.unwrap()` / `.expect("fail")` / `panic!` / `process::exit(1)` in CLI and leaf paths.
- `HashMap<String, _>` used pervasively instead of newtypes / stronger typing.
- Terse, abbreviated names (`s`, `st`, `xs`, `h`, `f`, `t`, `fqn`, `dir`, `langr`) and
  tiny helpers duplicated across modules instead of shared.
- The `kit` helpers and macros: `s()`, `empty()`, `s!`, `v!`, `ln!`, `maybe_*`, `nth`,
  `split*`.
- Vertical, crate-granular `use` blocks (enforced by `rustfmt.toml`; do **not** suggest
  collapsing to one line).
- Missing doc comments on small free functions.
- Pre-existing 100-column lines, or lines the change did not functionally alter. The
  80-column target (`.editorconfig` / `.dir-locals.el`) applies to **new** code only
  until a maintainer-run repo-wide reformat lands; do not flag existing width, and do
  not suggest reflowing untouched lines.

Do **not** recommend generic refactors that contradict this style: builder patterns,
trait-based abstraction, added generics, `?`/`Result` error propagation in place of
fail-fast, removing `.clone()`s, `is_empty()`, or "reduce duplication" for trivial
helpers.

## Severity
- Reachable correctness, security, data-loss, crash/hang, or cross-OS breakage → **high / block**.
- The house-style violations in the BLOCK list above → **block** (they are the
  acceptance criterion for this repo).
- Anything else → advisory at most.

The deterministic gate (`scripts/agent-check.sh`) and CI (`.github/workflows/agent-conformance.yml`)
already enforce the mechanical subset (fmt, line-scoped clippy, `no anyhow/thiserror`,
`no unbounded join_all`, workspace tests). **Your job is the judgment a linter cannot
do: does this change read like the original author wrote it?**
