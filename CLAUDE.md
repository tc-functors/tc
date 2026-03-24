## Overview

This repo is a Rust workspace for serverless app composition with custom code generation, deploy, topology, and cloud resource flows. Core includes Topology YAML, CLI, build/compose/deploy/invoke/emulate/asset deploy flows. The monorepo contains all infra (see `.github/` for workflows, `.kiro/` for steering), backends (Rust), assets, logs (`logs/` contains: combined.log, error.log), and CI/CD glue. The `logs/` directory stores runtime application logs and errors for all major systems and is referenced for monitoring and validation in operations flows.

### Workspaces and Entry Points
- Main CLI: `src/main.rs`, `src/lib.rs`, custom commands
- Libraries in `lib/`: modular providers
  - `lib/composer`: Topology + entity graphing
  - `lib/builder`: Builds, Dockerfiles, code packing (Python, Ruby, Node, Rust)
  - `lib/deployer`: Deploy to AWS/gateway/entities
  - `lib/scaffolder`: LLM-driven topology + handler scaffolding
  - `lib/tagger`, `lib/differ`, `lib/snapshotter`: Git/versioning and diff
- Logs directory: `logs/`, for build and runtime output (combined.log) and errors (error.log)

## Build, Test, CI, and Validation Commands
```sh
cargo build --workspace
cargo test --workspace
make build  # Custom build using Makefile; outputs `bin/tc`
cargo fmt   # Rust code format
make unit-test
make integration-test
tc lint .   # Static validation of topology and codegen
```
```sh
make docker-build-layer
```

### Compose and Deploy
- Compose topologies: `tc compose [dir]`
- Build function/image/layer: `tc build [function|layer|page]`
- Deploy to AWS: `tc deploy [entity|function|route|page]`
- Emulate: `tc emulate [function|state|...]`

### Entity Patterns
- Entities declared in YAML files (functions, events, mutations, routes, states, pages, etc.)
- Each entity type handled by modules in `lib/composer/src/aws/`
- Scaffold handler: `tc create function --name NAME --runtime rust` or via `.claude/skills/rust-function-handler/SKILL.md`

### Rust Function Handler Standards
- Entrypoint: `main`/`LambdaEvent<Value>` from `lambda_runtime`
```rust
use lambda_runtime::{service_fn, LambdaEvent, Error};
#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}
async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> { ... }
```
- See `.claude/skills/rust-function-handler/SKILL.md` for new handler patterns

### Docker Build and Packaging
- Python/Ruby/Node functions/layers: auto-generate Dockerfile and .dockerignore per runtime
  - Uses Docker Buildx, AWS creds secret, zipped deps, project dockerignore patterns
  - Layer zip splitting for >60MB: see `lib/builder/src/layer/mod.rs`

### Cloud Provider Roles & Policies (AWS)
- Roles in `lib/composer/src/aws/role.rs`
- Role ARNs and policies derived from entity/project name

### Pages/SPA Asset Hosting
- Topology `pages:` triggers static asset/S3 deploy + CloudFront
- Uses modules under `lib/composer/src/aws/` and `lib/deployer/src/aws/`

### Commit Patterns, Conventions, CI
- Commit with changesets (entities, handlers, build assets, deploy workflows)
- Use Git tags/versioning flows (tagger/differ)
- Run: `make unit-test`, `make integration-test`, plus workflow automation in `.github/`

### MCP Integration
- CircleCI workflows for build/deploy/invoke via API hooks (`lib/executor/src/circleci.rs`)

## Library/Directory Reference
- Entrypoint: `src/main.rs`
- Library: `lib/`
  - Topology, build, deployer, scaffolder, provider, etc.
- Examples: composition and testing in `examples/patterns/`, `examples/composition/`, `examples/pages/`, `examples/tests/`
- CI/CD and governance: `.github/` (workflows), `.kiro/` (steering)
- Logs: `logs/` (contains: combined.log, error.log) for runtime and test output

<!-- caliber:managed:pre-commit -->
## Before Committing

Run `npx --yes @rely-ai/caliber refresh` before creating git commits to keep docs in sync with code changes.
After it completes, stage any modified doc files before committing:

```bash
npx --yes @rely-ai/caliber refresh && git add CLAUDE.md .claude/ .cursor/ .github/copilot-instructions.md AGENTS.md CALIBER_LEARNINGS.md 2>/dev/null
```
<!-- /caliber:managed:pre-commit -->

<!-- caliber:managed:learnings -->
## Session Learnings

Read `CALIBER_LEARNINGS.md` for patterns and anti-patterns learned from previous sessions.
These are auto-extracted from real tool usage — treat them as project-specific rules.
<!-- /caliber:managed:learnings -->
