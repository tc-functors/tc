---
name: tc-topology-authoring
description: >-
  Edit topology YAML safely: canonical field order, validation mindset with tc
  compose and compile, and pointers to examples. Use when changing topology.yml,
  adding routes/functions/events, or authoring entities. Do NOT use for tc
  monorepo Rust provider work unless also editing app topology.
---

# Topology YAML authoring (app repos)

## Critical

- If this workspace is the **tc** monorepo (`lib/composer/`), prefer **`topology-entity-scaffolding`** and related skills that update Rust modules under `lib/composer/src/aws/`.
- **Field order** for top-level sections should follow: **`name`**, **`routes`**, **`events`**, **`functions`**, **`mutations`**, **`queues`**, **`channels`**, **`states`**, **`pages`** (then any other keys your spec allows—keep consistency with existing files).
- **Validate** by running **`tc compose`** and **`tc compile`** after substantive edits; fix errors before suggesting deploy.
- Do **not** instruct authors to run **`tc lint`**—that subcommand does not exist on the CLI.

## Trigger phrases

When the user says things like **add function**, **add event**, **new mutation**, **define page**, **add channel**, **entity**, or **component**, treat the change as topology-affecting: preserve indentation, naming, and existing patterns in their `topology.yml`.

## Structure and style

- Match the **existing** `topology.yml` in the repo: indentation, key names, and entity shape.
- Prefer **small, reviewable** diffs; avoid renaming entities unless the user asked for a migration plan.
- **Warn** on unknown attributes relative to your project’s conventions; when unsure, compare to examples in the tc repo (paths below).

## Examples in the upstream `tc` repository

Use these as **reference patterns** (clone or browse the tc repo; paths are from its root):

| Area | Example paths |
|------|----------------|
| Composition (routes, events, functions, …) | `examples/composition/` |
| REST / GraphQL patterns | `examples/patterns/` |
| Functions | `examples/functions/` |
| Pages / SPA | `examples/pages/` |
| States | `examples/states/` |
| Tables / tests | `examples/tables/`, `examples/tests/` |

## Validation workflow

1. Edit **`topology.yml`** (or the topology file your project uses).
2. **`tc compose`** — optional `-d <dir>`, `-r` / `--recursive`, `-c <entity>`, `--versions`, `--root`, `--compact`, `-f <format>`, `-t` / `--trace`.
3. **`tc compile`** — optional `-d <dir>`, `-r` / `--recursive`, `--root`, `-t` / `--trace`.
4. Run **`tc test`** and project unit tests as appropriate.

## Agent behavior

- If glob or rule **`tc-topology-yaml`** is active, align with its field-order and trigger guidance.
- Do not assume every optional topology key exists in every project—**read the file first**.
