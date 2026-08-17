# GitHub Copilot instructions for `tc`

Canonical guidance is in `/AGENTS.md` and `/docs/agents/`. Follow them; conformance
to the existing house style is the acceptance criterion for this repo.

Core rules:
- Terse **free functions** over methods/`impl`; take `&Topology`/`&Auth`/`&str`,
  return owned `String`/`Vec`/`HashMap`.
- Use the **`kit`** crate (`use kit as u;` + `use kit::*;`): `s()`, `empty()`, `s!`,
  `v!`, `maybe_*`, `nth`, `split*`.
- Fail-fast: `.unwrap()`, `.expect("fail")`, `panic!`. **No `anyhow`/`thiserror`/
  custom error enums.** Graceful degradation only in `kit::memo`/resolver/`differ`.
- Data structs: `#[derive(Serialize, Deserialize, Clone, Debug)]`, `pub` fields,
  `HashMap<String, _>`, `Option<T>` + `#[serde(default)]`.
- Bounded concurrency (`buffer_unordered`); `TC_*` env config; vertical
  crate-granular imports (nightly `rustfmt`); `#[cfg(test)]` twin for new leaf IO.
- Do not add a provider trait (AWS is hard-coded), do not "modernize" intentional
  idioms, do not reformat unrelated files.

Build/test: `cargo build` and `cargo test --workspace` (green as of Phase 2; CI runs
it). Gate before finishing: `./scripts/agent-check.sh`. See `docs/agents/CALIBRATION.md`.
