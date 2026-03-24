---
name: topology-entity-scaffolding
description: Scaffolds new entities by adding entries in topology.yml and generating corresponding Rust modules and asset code. Use when user says: 'add function', 'add event', 'define channel', or creates new keys in topology YAML under routes, functions, events, mutations, states, pages, etc. Do NOT use for editing existing YAML keys or values. Key capabilities: updates topology.yml with new entity block, creates stub Rust files in lib/composer/src/aws/, generates scaffolding code using lib/scaffolder, and integrates new entities into build and deploy workflows.
---
# Topology Entity Scaffolding

## Critical

1. Always update the correct entity section in a valid YAML (e.g., add under a section like `functions:`), following established naming and indentation conventions in project configuration files. Never reference removed or invalid file paths or source code modules directly — always select the right insertion point using currently supported conventions.
2. Changes must be reflected in the appropriate project Rust module (for example, the entity-specific handler in the codebase under `lib/composer/src/aws/` or appropriate infrastructure directory).
3. Validate YAML edits with `yamllint` or a similar tool and ensure generated code builds and passes tests.

## Instructions

### Step 1: Add or Update Entities
- Edit the YAML project configuration (such as the main file listing topology entities).
- Insert the new entity under the correct section. Example YAML:
```yaml
functions:
  user_signup:
    runtime: rust
    handler: src/main.rs
    events:
      - http:
          path: /signup
          method: post
```
- Validate YAML syntax:
```sh
yamllint path/to/project-entities.yml
```

### Step 2: Compose to Rust
- Run the topology composition tool:
```sh
tc compose .
```
- This triggers Rust code module generation (see output in `lib/composer/src/aws/`).
- Validate:
```sh
cargo build --workspace
```

### Step 3: Implement Handler Logic
- Open the relevant Rust entity source file generated in `lib/composer/src/aws/` (for example, `function.rs` for functions, `event.rs` for events, etc).
- Add or complete the entity handler:
```rust
use lambda_runtime::{service_fn, LambdaEvent, Error};
#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}
async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // entity-specific logic
    Ok(json!("user_signup processed"))
}
```
- Format code:
```sh
cargo fmt
```
- Test your handler:
```sh
cargo test --workspace
```

### Step 4: Build and Deploy
- Build artifact for specific entity:
```sh
tc build function --name user_signup
```
- Deploy entity:
```sh
tc deploy function --name user_signup
```
- Emulate locally:
```sh
tc emulate function --name user_signup
```

## Examples

User: "Add event user_signup supported by a Rust Lambda"
→ YAML edit as above, compose, update handler in correct Rust module, validate with `cargo test` and `tc deploy ...`

## Common Issues
- YAML parse error? Check `yamllint` output.
- Entity exists already? Pick new unique name.
- Rust compile fails? Compare handler style with examples in the repo (`lib/composer/src/aws/`).
- Deploy/role error? Review `.claude/skills/cloud-role-policy/SKILL.md` for the relevant steps.

---
After every step, run `cargo fmt` and tests to keep integration tight and codebase healthy.
