# CLAUDE.md

This repository's agent guidance is canonical in **`AGENTS.md`** and **`docs/agents/`**.
Read them before writing code — they are authoritative for Claude Code too.

@AGENTS.md

Deep references:
- @docs/agents/STYLE.md — prescriptive style guide (the most important file)
- @docs/agents/ARCHITECTURE.md — pipeline, crate map, data model
- @docs/agents/DSL.md — `topology.yml` / `function.yml` schema
- @docs/agents/CALIBRATION.md — the verified build/test/lint baseline

TL;DR: terse free functions over methods; use the `kit` crate (`s()`, `empty()`,
`s!`, `maybe_*`); fail-fast (`.unwrap()`/`panic!`, **no** `anyhow`/`thiserror`);
`HashMap`-of-everything data structs with `#[derive(Serialize, Deserialize, Clone,
Debug)]`; bounded concurrency; nightly `rustfmt`; add `#[cfg(test)]` twins for leaf
IO. Run `./scripts/agent-check.sh` before finishing. Do not "modernize" the
deliberate idioms and do not add a provider trait.
