# Optional VS Code / Cursor editor helper

This folder is **not** the primary Cursor integration for `tc`. Cursor agents are guided by **skills** (markdown under `.cursor/skills/`) and **rules** (`.cursor/rules/`). For application repositories, copy the portable pack from [`packs/cursor-tc-app/`](../packs/cursor-tc-app/) into your app repo.

## Contents

- [`tc-serverless/`](tc-serverless/) — VS Code extension source: Command Palette commands and `tasks.json` provider for running `tc` (compose, compile, build, deploy, emulate, test, invoke, etc.). Use this if you want editor shortcuts in **VS Code** or **Cursor** without relying on the agent.

## Development

From `tc-serverless/`, run `npm install && npm run compile`. Package a VSIX with `npx --yes @vscode/vsce package` when you need to install the extension in another machine or repo. See that directory’s README for settings and task examples.
