# `tc` Style Guide (prescriptive)

Write code indistinguishable from the original author's. This is the acceptance
criterion for lifting the "no LLM-generated code" rule. When a general Rust habit
conflicts with a rule here, **the rule here wins**. Every rule below was verified
against the current source; file:line citations are illustrative anchors.

> **Two styles coexist — know which layer you're in.**
> - **Base style** (most of the code, e.g. `lib/kit/src/core.rs`, `io.rs`, and every
>   entity module): terse, free-function-heavy, `unwrap`-happy, fail-fast.
> - **Disciplined layer** (`lib/kit/src/memo.rs`, the caching helpers in `io.rs`,
>   resolver concurrency in `lib/resolver/`, and `lib/differ/`): heavily
>   doc-commented, gracefully degrading, capacity-budgeted, unit-tested with
>   regression rationale.
>
> Match the **base style** for ordinary code. Match the **disciplined layer** only
> when writing process-wide caches, concurrency limits, or a typed-error contract —
> and when you do, document it as heavily as those files do.

---

## 1. Naming
- Free functions and helpers are **terse, lowercase, abbreviated**. No `get_`/
  `compute_` prefixes. Examples from `kit`: `s()` (=`String::from`), `empty()`,
  `kv(k,v)`, `sw(opt)`, `triml`/`trim`, `splitf`/`splitl`, `nth`, `second`, `first`,
  `maybe_str`/`maybe_string`/`maybe_int`, `pwd`, `mkdir`, `slurp`, `sh`.
- Reuse the existing abbreviation vocabulary — do **not** invent synonyms:
  `s`/`st` (string), `h` (HashMap), `xs` (Vec), `f` (file/function/format-string by
  context), `t` (topology/value), `ctx`, `rt` (root), `langr`/`lang`, `fqn`,
  `dir`. Loop bindings are single letters: `for (k, v) in …`, `for (_, f) in fns`.
- Types are `PascalCase`, mostly single nouns (`Topology`, `Function`, `Auth`,
  `Context`, `Entity`). Closed variant sets get a **`Kind` suffix**
  (`BuildKind`, `TopologyKind`, `LangRuntime`).
- Env/config knobs are `TC_`-prefixed `SCREAMING_SNAKE` (`TC_SANDBOX`,
  `TC_RESOLVE_CONCURRENCY`, `TC_ECR_REPO`).
- Deliberately-unused-but-kept items get a leading `_` (`_remove_suffix`, `_f`).

## 2. Function design
- **Free functions over methods.** `impl` blocks are for genuine data types
  (`Topology`, `Auth`, `AsyncMemo`). Logic that would idiomatically be a method is a
  free function that *takes* `&Topology`/`&Auth`. Do not add methods to domain
  structs to hold behavior.
- State is passed explicitly: `auth: &Auth`, `ctx: &Context`, `dir: &str`,
  `sandbox: &str` threaded by hand. No ambient globals except the intentional
  process-wide caches (§8).
- Signatures take `&str`/`Vec<T>`, return **owned** `String`/`Vec`/`HashMap`. The
  author routinely takes `Vec<&str>` by value and `.clone()`s inside — do not
  "optimize" these into borrows.
- Destructure at the top: `let Topology { functions, routes, .. } = topology;`
- Keep functions short (3–15 lines typical); decompose longer ones into a sequence
  punctuated by `tracing::debug!`.
- **At most 4 parameters** per function (enforced by `clippy.toml`
  `too-many-arguments-threshold = 4`; a 5th parameter fails line-scoped clippy on the
  signature). Thread wider state through `&Context`/`&Auth` or a small `pub` struct
  rather than a long argument list.

## 3. Error handling
- **Base style unwraps freely and fails loud**: `.unwrap()`, `.expect("fail")`,
  `panic!("…")`, `std::process::exit(1)`. This is the intended CLI posture, not an
  oversight. `.expect("fail")` / `.expect("Not found")` are the stock messages.
- **`Option` is the primary "maybe" type**, matched with `match`, not combinators.
  Prefer the `kit` coalescers: `unwrap`, `sw`, `maybe_str`, `maybe_string`,
  `maybe_int`, `maybe_hashmap`, `maybe_vec_string`, `safe_unwrap`, `opt_as_bool`.
- **`Result` is rare** in signatures. There are **no custom error enums, no
  `anyhow`, no `thiserror`** — do not introduce them.
- **Disciplined-layer exception**: caches/concurrency degrade gracefully —
  `if let Ok(g) = cache.lock() { … }` treats a poisoned lock as a miss rather than
  panicking. Use this *only* in that class of code.

## 4. Strings
- Use `s()` for `String::from` and `empty()` for `""`/`String::new()`.
- The `s!` macro is the everyday string builder — it concatenates any `Display`
  args: `h.insert(s!("API_GATEWAY_URL"), endpoint)`, `s!(k)`, `s!(a, "-", b)`.
  Reach for `s!` where you'd otherwise `.to_string()` a key or concatenate.
- `format!` is for multi-part path/ARN templating:
  `format!("arn:aws:lambda:{}:{}:layer:{}", auth.region, auth.account, name)`.
- Case conversion via `kit::text` (`kebab_case`, `snake_case`, `pascal_case`);
  coloring via `red`/`blue`/`green` (note the real fn name `mangenta`); templating
  via `stencil(s, table)`.

## 5. Serde & data structures
- Data struct derive quartet: `#[derive(Serialize, Deserialize, Clone, Debug)]`
  (import from `serde_derive`). All fields `pub`.
- `HashMap<String, _>` is the dominant collection — `Topology` is ~15 of them.
  Model new collections the same way; do not reach for newtype keys or
  `Vec<(String, T)>`.
- `Option<T>` marks genuinely-absent fields; add `#[serde(default)]` to fields that
  may be missing from serialized input.
- Enums for closed sets, matched exhaustively, `FromStr` + `to_str()`, `todo!()`
  arms for unimplemented variants (honest, not `unreachable!()`).
- Templating round-trips through JSON strings deliberately: serialize → `render` →
  deserialize. Use `kit::json` helpers (`json_value`, `merge_json`, `pretty_json`).

## 6. Imports & formatting
`rustfmt.toml` is authoritative (needs **nightly**): crate-granular, **vertical**,
one group, braces even for a single item, comments wrapped at 100.

```rust
use std::{
    collections::HashMap,
    str::FromStr,
};
use serde_derive::{
    Deserialize,
    Serialize,
};
```
- `kit` is imported two ways at once: `use kit as u;` (then `u::pwd()`) and
  `use kit::*;` (macros + free fns). `kit::sh` is also called fully-qualified.
- Section dividers are lowercase line comments: `// aws`, `// option`, `// arn`.
- **Column width target is 80** (emacs `rust-mode`'s default `fill-column`), set in
  `.editorconfig` + `.dir-locals.el` so standard editors wrap new code there. Write
  new/changed code at ≤80. `rustfmt.toml` still carries the historical `max_width`/
  `comment_width = 100` ceiling; a deliberate, maintainer-run repo-wide
  `cargo +nightly fmt` will lower it to 80 and make it the hard gate. Until then, do
  **not** reflow unrelated 100-column lines (§10 / BUGBOT #6 — no whitespace churn).

## 7. Testing
- **Default: put behavior/unit tests in a sibling file under
  `lib/<crate>/tests/<area>_test.rs`** (per entity, e.g.
  `lib/composer/tests/route_test.rs`), not inline. Keeping tests and source in
  separate files (the Clojure/Ruby split) makes both easier to maintain. The
  per-entity files already scaffolded in `lib/composer/tests/` are the canonical
  home — fill the matching one instead of adding an inline `mod tests`.
- Inline `#[cfg(test)] mod tests { use super::*; … }` is reserved for the two cases a
  `tests/` file cannot cover: the **leaf-IO cfg-twin** (next bullet), and a test that
  genuinely needs a crate-private item. If the test only touches the crate's public
  surface, it belongs in the sibling `tests/` file.
- **Mock leaf IO with a cfg twin**, not a trait/mock object. The same `pub fn` has
  two bodies selected by cfg:
  ```rust
  #[cfg(not(test))]
  pub fn file_exists(path: &str) -> bool { Path::new(path).exists() }
  #[cfg(test)]
  pub fn file_exists(path: &str) -> bool { path.contains("true") }
  ```
  **When you add a leaf IO primitive, add its `#[cfg(test)]` twin** — omitting it is
  exactly the bug that currently breaks `kit`'s test target (see CALIBRATION.md).
- `mockall` is available for trait-level mocking where the cfg-twin doesn't fit.
- Test names are descriptive snake_case sentences (`distinct_keys_each_run_once`).
- Regression tests carry a doc comment explaining the bug they pin.
- Async tests use `#[tokio::test]` (`flavor = "multi_thread"` for concurrency).

## 8. Async / concurrency
- `async fn` returning owned values for anything touching AWS/IO.
- **Bounded concurrency only**: `futures::stream::iter(..).buffer_unordered(n)` —
  never unbounded `join_all`. The cap is env-configurable, clamped to a documented
  `const` ceiling, with a doc comment justifying the numbers (see
  `resolve_concurrency()` in `lib/resolver/src/lib.rs`).
- Process-wide memoization goes through `kit::AsyncMemo`; document why each cache
  key is sufficient (the existing keys carry such comments).

## 9. Macros
Three tiny `macro_rules!` in `kit` (`s!`, `v!`, `ln!`) are the only macro layer.
Do **not** reach for proc-macros, `lazy_static`, or `once_cell` — the author
hand-rolls with `std::sync::OnceLock`.

## 10. Deliberate idioms — match, don't "fix"
These trip default clippy/linters but are intentional (the clippy gate allow-lists
them; see CALIBRATION.md). Do not change them, and write new code the same way:
- Liberal `.clone()`, including `xs.clone().into_iter()`, `parts.clone().first()`.
- `if xs.len() > 0` (not `!is_empty()`); `match opt { Some(_) => true, None => false }`
  (not `.is_some()`).
- Duplicated trivial helpers across modules instead of a shared dependency.
- `.unwrap()`/`.expect("fail")` in leaf helpers; `panic!` on missing config.
- `HashMap`-of-everything over stronger typing.

The author deliberately **avoids**: custom error enums / `anyhow` / `thiserror`;
trait-heavy abstraction and generics; builder patterns / fluent chains on domain
types; unbounded concurrency; long functions.

---

## Golden examples (imitate these)
- **A `kit` helper** — terse, owned return, `match` over `Option`:
  `lib/kit/src/core.rs` → `unwrap`, `maybe_string`, `nth`, `kv`.
- **A cfg-twin mock** — `lib/kit/src/io.rs` → `file_exists`, `slurp`.
- **The disciplined layer** — `lib/kit/src/memo.rs` (`AsyncMemo` + regression tests)
  and `resolve_concurrency()` in `lib/resolver/src/lib.rs` (clamped, documented cap).
- **Struct-destructuring dispatch** — `lib/deployer/src/lib.rs` `create` /
  `update_entity` (`let Topology { .. } = …;` then per-entity fan-out; exhaustive
  `match Entity`).
- **Data model** — `lib/composer/src/…` `Topology` (the `HashMap`-of-everything
  resolved struct) vs the `Option`-heavy `TopologySpec` in `lib/compiler`.
- **Idempotent provider verb** — `lib/provider/src/aws/*` `find_or_create_*` /
  `create_or_update_*`.

## Quick self-check before committing
- Did I add `anyhow`/`thiserror`/a custom error enum? → remove it.
- Did I write a method where a free function fits? → make it free.
- Did I reinvent a `kit` helper (`"".to_string()`, manual split)? → use `kit`.
- Did I add an unbounded `join_all`? → `buffer_unordered(n)` with a clamped cap.
- Did I add a leaf IO fn without a `#[cfg(test)]` twin? → add it.
- Did I "modernize" an intentional idiom (§10)? → revert it.
- Are my imports vertical/crate-granular (nightly fmt)? → `cargo +nightly fmt`.
