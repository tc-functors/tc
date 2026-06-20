# AGENTS.md

## Project overview
- This is `tc` (Topology Composer), a Rust 2024 workspace for describing, composing, and deploying cloud-native serverless systems.
- Use Rust 1.90, as specified by the root `Cargo.toml`.
- The CLI lives in `src/`; reusable workspace crates live in `lib/`.
- AWS integrations and infrastructure behavior live primarily under `lib/provider`, `lib/composer`, `lib/resolver`, and `lib/deployer`.
- Examples under `examples/` are executable documentation; update them when behavior or supported configuration changes.

## Commands that matter (build/test/lint)
- Build with `make build`.
- Run the full unit suite with `make unit-test`.
- Check the workspace with `cargo check --workspace`.
- Lint with `cargo clippy --workspace --all-targets`.
- Format Rust with `make fmt`; this requires nightly rustfmt.
- During iteration, run the narrowest relevant crate or test first, then run the full required checks before completion.
- Do not run deployment, release, tagging, AWS, or cross-compilation commands unless the user explicitly requests them.

## Conventions
- Present a short implementation plan and wait for approval before editing files.
- Preserve the surrounding code style and keep diffs narrowly scoped.
- Do not reformat unrelated code, even when a formatter would change it.
- Add or update tests for behavior changes.
- Prefer unit tests beside the implementation using `#[cfg(test)]`.
- Follow existing module boundaries; do not move responsibilities between crates without discussing the design first.
- Keep public behavior and configuration backward-compatible unless the requested change explicitly permits a break.
- Update examples or README text when user-facing commands, configuration, or behavior changes.
- Report assumptions when AWS behavior cannot be verified locally.

## Boundaries (never touch)
- Do not modify `Cargo.lock` unless dependencies or Cargo configuration change.
- Do not edit generated artifacts such as `target/`, `bin/`, or the root `tc` binary.
- Do not change package versions, tags, release assets, or release workflows unless explicitly requested.
- Do not access live AWS resources or use credentials unless explicitly requested.
- Do not stage or commit changes without asking immediately before doing so.
- Never discard or overwrite unrelated user changes.

## How to verify your work before telling me it's done
- Inspect `git diff` and confirm every changed line belongs to the approved task.
- Run the narrowest relevant tests while developing.
- Run `cargo check --workspace`.
- Run `cargo clippy --workspace --all-targets`.
- Run `make unit-test`.
- Run `make build`.
- Run `make fmt`, then inspect the diff and revert unrelated formatting changes.
- Report every command run, its result, and anything not run with the reason.
