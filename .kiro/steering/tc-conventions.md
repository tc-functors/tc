---
inclusion: always
---

# tc conventions (Kiro steering)

Canonical agent guidance lives in `#[[file:AGENTS.md]]` and the `docs/agents/`
set. They are authoritative; this steering file just ensures they are always in
context. Read the style guide before writing code.

Deep references:
- `#[[file:docs/agents/STYLE.md]]` — prescriptive style guide (most important)
- `#[[file:docs/agents/ARCHITECTURE.md]]` — pipeline, crate map, data model
- `#[[file:docs/agents/DSL.md]]` — `topology.yml` / `function.yml` schema
- `#[[file:docs/agents/CALIBRATION.md]]` — verified build/test/lint baseline

Non-negotiables (full detail in STYLE.md):
1. Terse **free functions** over methods; `&Topology`/`&Auth`/`&str` in, owned out;
   **≤4 parameters** (thread wider state via `&Context`/a struct).
2. Use the **`kit`** crate (`s()`, `empty()`, `s!`, `v!`, `maybe_*`, `nth`, `split*`).
3. Fail-fast (`.unwrap()`/`.expect("fail")`/`panic!`); **no `anyhow`/`thiserror`**.
   Graceful degradation only in `kit::memo`/resolver concurrency/`differ`.
4. `#[derive(Serialize, Deserialize, Clone, Debug)]`, `pub` fields, `HashMap`
   -of-everything, `Option<T>` + `#[serde(default)]`.
5. `Kind`-suffixed enums with `todo!()` arms; bounded concurrency; `TC_*` config.
6. Vertical crate-granular imports (nightly `rustfmt`), **80-col target for new code**
   (`.editorconfig`/`.dir-locals.el`); respect `TopologySpec` vs `Topology`.
7. Tests in a sibling `lib/<crate>/tests/<area>_test.rs`; inline `#[cfg(test)]` only
   for the leaf-IO cfg-twin or a private-access test.

Do not add a provider trait, do not "modernize" intentional idioms, do not reformat
unrelated files. Build with `cargo build`; test with `cargo test --workspace`
(green as of Phase 2) or `cargo test -p composer`. Gate: `./scripts/agent-check.sh`.
