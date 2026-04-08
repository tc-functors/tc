---
name: tc-app-repo
description: >-
  Bootstrap or evolve an application repo that uses the tc CLI: topology layout,
  installing and pinning tc, compose/compile/test, sandboxes, and CI hooks.
  Use when the user says serverless app repo, topology.yml, tc compose, pin tc
  version, sandbox, or CI deploy with tc. Do NOT use for the tc toolchain
  monorepo (lib/composer development).
---

# tc application repository

## Critical

- If this workspace is the **tc** source tree (e.g. contains `lib/composer/`), prefer the monorepo’s existing `.cursor/skills/` (topology scaffolding, cloud roles, Rust providers) instead of this skill.
- Teach only **`tc` subcommands that exist in the CLI** (see `tc --help` or the installed binary). There is **no** `tc lint` subcommand.
- **Compose** materializes topology into generated artifacts; **compile** validates/processes topology spec—use both for confidence after YAML changes.

## Typical app repo layout

- **`topology.yml`** (or a directory of topology files your project uses) at the repo root or a known subfolder—keep paths consistent with `-d` flags.
- **Handlers and assets** as your stack requires (e.g. per-function directories, layers, pages)—match your team’s conventions and CI.
- **CI** invokes `tc` with explicit **environment** (`-e`), **sandbox** (`-s`), and **topology** (`-t` / `--topology` where applicable) so builds are reproducible.

## Installing and pinning `tc`

- Install the `tc` binary the way your organization distributes it (release artifact, package, or `cargo install` from a tagged revision).
- **`tc version`** prints the current version; use **`tc upgrade --version <ver>`** (`-v`) when the CLI supports upgrading itself in your environment.
- Pin the version in CI (image digest, locked artifact, or explicit install step) so compose/deploy behave consistently.

## Day-to-day commands

| Goal | Command (from repo / topology root as appropriate) |
|------|------------------------------------------------------|
| Compose topology | `tc compose` with optional `-d <dir>`, `-r` / `--recursive`, `-c <entity>`, `--versions`, `--root`, `--compact`, `-f <format>`, `-t` / `--trace` |
| Compile spec | `tc compile` with optional `-d <dir>`, `-r` / `--recursive`, `--root`, `-f <file>`, `-R` / `--repl`, `-t` / `--trace` |
| Run tests | `tc test` with optional `-d <dir>`, `-e` profile, `-R` role, `-s` sandbox, `-u` unit filter, `-r` / `--recursive`, `-i` / `--interactive`, `-t` / `--trace` |
| Visualize | `tc visualize` with optional `-d <dir>` |
| Scaffold (LLM) | `tc scaffold` with optional `-e`, `-d`, `-f` / `--functions`, `--llm`, `--provider`, `--model` |

## Sandboxes and lifecycle (when you use them)

- **Create**: `tc create` — `-e`, `-s`, optional `-R`, `-T` / `--topology`, plus flags such as `-r`, `--cache`, `--notify`, `--sync`, `--remote`, `--dry-run`, `--dirty`, `-f` / `--force`, `-t` / `--trace`.
- **Update / delete**: `tc update`, `tc delete` — same style of `-e`, `-R`, `-s`, `-c` entity, `-r`, `--cache`, `-i`, `--remote`, `-t`.
- **Diff / resolve / list**: `tc diff`, `tc resolve`, `tc list` — profile/sandbox/entity flags as in `--help`.
- **Freeze / unfreeze / prune / route / snapshot / tag / changelog**: available for ops workflows; prefer **`tc <cmd> --help`** before documenting rare flags.

## CI deploy and releases

- **`tc deploy`** is oriented toward **CI**: non-interactive use typically requires one of **`--version` (`-v`)**, **`--branch` (`-b`)**, or **`--snapshot`**, along with **`--topology` (`-t`, alias `--service`)**, **`--env` (`-e`)**, **`--sandbox` (`-s`)**, and optional **`--dir` (`-d`)**, **`--interactive` (`-i`)**, **`--force` (`-f`)**.
- Some distributions expose **`tc ci-release`** for release automation; it may be omitted from **`tc --help`** because it is marked hidden in the CLI source—rely on your org’s runbooks if you use it.

## Evolution

- Use branches and pinned `tc` versions per environment; align **sandbox** names with CI and local defaults.
- After topology or handler changes: **`tc compose`**, **`tc compile`**, then **`tc test`** (and your language-specific tests).
