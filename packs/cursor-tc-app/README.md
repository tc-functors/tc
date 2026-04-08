# Cursor skill pack: `tc` application repositories

Portable **agent skills** and a **Cursor rule** for repositories that **use** the `tc` CLI (topology YAML, handlers, CI deploy)—not for developing the `tc` toolchain itself.

## Install in another repository

1. Copy the skill directories into your app repo’s `.cursor/skills/` (merge with existing skills; do not remove unrelated skills):

   ```bash
   # From a checkout of the tc repo, with APP_REPO set to your application root:
   mkdir -p "$APP_REPO/.cursor/skills"
   cp -R packs/cursor-tc-app/skills/tc-app-repo "$APP_REPO/.cursor/skills/"
   cp -R packs/cursor-tc-app/skills/tc-topology-authoring "$APP_REPO/.cursor/skills/"
   cp -R packs/cursor-tc-app/skills/tc-aws-build-deploy "$APP_REPO/.cursor/skills/"
   ```

2. Copy the topology rule into `.cursor/rules/` (merge with existing rules):

   ```bash
   mkdir -p "$APP_REPO/.cursor/rules"
   cp packs/cursor-tc-app/rules/tc-topology-yaml.mdc "$APP_REPO/.cursor/rules/"
   ```

3. If you already have a rule or skill with the same file name, rename one side or merge the content manually.

4. Restart Cursor or reload the window so new skills are picked up.

## What you get

| Artifact | Role |
|----------|------|
| `skills/tc-app-repo` | Bootstrap and evolve an app repo: layout, installing `tc`, compose/compile/test, sandboxes and CI. |
| `skills/tc-topology-authoring` | Safe YAML edits: field order, validation via compose/compile, pointers to upstream examples. |
| `skills/tc-aws-build-deploy` | Build, deploy, emulate, invoke with flags aligned to the current `tc` CLI. |
| `rules/tc-topology-yaml.mdc` | Applies when `topology.yml` files are in context; canonical field order and trigger phrases. |

## Optional VS Code extension

If you want Command Palette / `tasks.json` shortcuts (not agent skills), see [`contrib/vscode-tc-optional/`](../../contrib/vscode-tc-optional/) in this repo.

## Developing the `tc` monorepo

If the workspace contains `lib/composer` and is the **tc** source tree, use the existing monorepo `.cursor/skills/` (e.g. topology scaffolding, cloud role/policy) instead of treating this pack as authoritative for Rust provider code.
