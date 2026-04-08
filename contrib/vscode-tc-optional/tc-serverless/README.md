# tc Serverless (VS Code / Cursor)

**Optional** editor integration for the `tc` CLI (compose, compile, build, deploy, emulate, test, invoke) from the Command Palette or reusable `tasks.json` entries. Primary Cursor guidance for app repos lives in the [`packs/cursor-tc-app/`](../../../packs/cursor-tc-app/) skill pack, not in this extension.

## Requirements

- `tc` on your `PATH`, or set **tc Serverless: Executable Path** in settings.
- A workspace folder opened in Cursor/VS Code (your application repo).

## Install in another repository

1. **From a VSIX** (after packaging in this directory):

   ```bash
   npm install && npm run compile
   npx --yes @vscode/vsce package --allow-missing-repository --no-dependencies
   ```

   Then in the app repo: **Extensions → … → Install from VSIX…** and pick the `.vsix`.

2. **Development**: open this folder (`contrib/vscode-tc-optional/tc-serverless`) and use **Run Extension** (F5) to debug against a sample workspace window.

## Settings (`tc.serverless.*`)

| Setting | Purpose |
|--------|---------|
| `executablePath` | `tc` or absolute path to the binary |
| `topologyDirectory` | Passed as `-d` for compose, compile, test, deploy, visualize |
| `defaultProfile` | Passed as `-e` where the CLI supports it |
| `defaultSandbox` | Passed as `-s` where supported |

## Commands

Open the Command Palette and run **tc:** … (Compose, Compile, Build, Deploy, Resolve, Emulate, Test, Invoke, Version, Visualize).

## Custom tasks in `tasks.json`

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "tc: compose",
      "type": "tc",
      "command": "compose",
      "args": [],
      "problemMatcher": []
    },
    {
      "label": "tc: deploy staging",
      "type": "tc",
      "command": "deploy",
      "args": ["-e", "staging"],
      "problemMatcher": []
    }
  ]
}
```

Optional `cwd` is relative to the workspace folder or absolute.
