# Caliber Learnings

Accumulated patterns and anti-patterns from development sessions.
Auto-managed by [caliber](https://github.com/rely-ai-org/caliber) — do not edit manually.

- **[correction]** When asked for a Cursor plugin, do NOT create a VS Code extension; these serve different purposes. Cursor plugins provide skills and rules for repo evolution and operations, whereas VS Code extensions provide editor UI features.
- **[pattern]** Cursor plugins for this project should embed skills related to repo bootstrap, topology authoring, AWS build/deploy, and validation rules rather than UI commands.
- **[pattern]** Use `tc` CLI subcommands with appropriate flags: `-d` for topology directory if set, `-e` for default profile/env, `-s` for default sandbox. These flags are critical for multi-env CI/CD workflows.
- **[convention]** Inside VS Code extension commands invoking `tc`, always pass topologyDirectory as `-d` flag, profile as `-e`, and sandbox as `-s` where supported.
- **[fix]** If packaging VS Code extension with `vsce package` triggers missing LICENSE or repository warnings, add the LICENSE file and repository field in `package.json`. Use `--allow-missing-repository` flag cautiously to avoid build errors.
- **[fix]** Copy LICENSE file to the VS Code extension directory before packaging to satisfy `vsce package` license requirement.
- **[gotcha]** When moving VS Code extension from `extensions/tc-serverless` to a `contrib/` folder, watch out for `.vscode` directory permission issues during `rsync`. Retrying without `.vscode` may be needed.
- **[pattern]** For VS Code extensions, use a `tasks.json` with custom task types of `tc`, mapping to `tc` subcommands and args; allows reuse of `tc` CLI workflows within editor tasks.
- **[env]** Require that the `tc` binary is on PATH or set explicitly via VS Code extension setting `tc.serverless.executablePath`.
- **[pattern]** In Cursor skills or rules managing topology YAML, target `.mdc` rules to be portable with friendly globs for app repos.
- **[pattern]** Use clear GUIDED user input prompts for entity names, kinds, and flags before running build, deploy, or invoke commands for better UX in extensions.
