---
name: tc-aws-build-deploy
description: >-
  Build, deploy, emulate, and invoke with the tc CLI in an application repo
  (AWS-oriented workflows). Use when the user says tc build, tc deploy, emulate,
  invoke lambda, publish function, or CI deploy. Do NOT use for editing IAM
  Rust in lib/composer—that is monorepo-only unless the repo is tc itself.
---

# tc build, deploy, emulate, invoke (app repos)

## Critical

- **Application repos** use **`tc`** against **topology and handlers**. Editing **`lib/composer/src/aws/`** is for the **tc monorepo** only unless this workspace is that codebase.
- Flags below match the **`tc` CLI** as defined in the toolchain’s `main.rs`; prefer **`tc <subcommand> --help`** when in doubt.
- There is **no** `tc lint` subcommand.

## `tc build`

Build layers, extensions, and pack function code.

Common flags:

- **`-e` / `--profile`** — profile
- **`-n` / `--name`** — entity name
- **`-k` / `--kind`** — kind (e.g. function, layer—per your usage)
- **`-v` / `--version`** — version
- **`--clean`** — clean build
- **`-r` / `--recursive`**
- **`-t` / `--trace`**
- **`-p` / `--publish`**, **`--promote`**, **`--shell`**
- **`-s` / `--sync` / `--sync-to-local`**
- **`--parallel`**, **`--remote`**

Example shape (adjust profile/name/kind):

```bash
tc build -e <profile> -n <entity_name> -k <kind>
```

## `tc deploy` (CI-oriented)

Triggers deploy via CI. Non-interactive flows typically need one of **`--version` (`-v`)**, **`--branch` (`-b`)**, or **`--snapshot`**, plus topology/environment context:

- **`-t` / `--topology` / `--service`**
- **`-e` / `--env`**
- **`-s` / `--sandbox`**
- **`-d` / `--dir`**
- **`-i` / `--interactive`**, **`-f` / `--force`**

If none of version/branch/snapshot is given, the CLI may prompt you to specify one (see behavior on your version).

## `tc emulate`

Run or shell into emulation for an entity.

- **`-e` / `--profile`**
- **`-s` / `--sandbox`**
- **`-c` / `--entity`**
- **`-k` / `--kind`**
- **`-l` / `--shell`**
- **`-t` / `--trace`**

## `tc invoke`

Invoke synchronously or asynchronously.

- **`-p` / `--payload`**
- **`-e` / `--profile`**
- **`-s` / `--sandbox`**
- **`-c` / `--entity`**
- **`-d` / `--dir`**
- **`--emulator`**, **`--dumb`**
- **`-t` / `--trace`**

Note: **`invoke`** does not expose a **`-R` / `--role`** flag on the CLI; use **`test`** or other commands if you need **`-R`**.

## Operational cautions

- **AWS credentials and profiles**: use the profile (`-e`) and roles your org documents; never embed secrets in topology.
- **Sandbox names**: keep CI, local, and `tc` defaults aligned to avoid deploying to the wrong environment.
- **Idempotency**: treat **deploy** as potentially mutating cloud state; confirm **environment** and **sandbox** before suggesting **`--force`**.

## Related commands

- **`tc update`**, **`tc create`**, **`tc delete`**, **`tc resolve`**, **`tc list`**, **`tc snapshot`**, **`tc prune`**, **`tc route`**, **`tc freeze`**, **`tc unfreeze`** — use **`tc <cmd> --help`** for full flag lists.

## Validation before and after

- After topology changes: **`tc compose`** and **`tc compile`** (see **tc-topology-authoring**).
- After builds: run **`tc test`** where applicable before promoting or releasing.
