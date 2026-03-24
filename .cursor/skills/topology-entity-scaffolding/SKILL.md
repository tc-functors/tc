---
name: topology-entity-scaffolding
description: Scaffolds new Topology YAML entities and updates Rust modules. Use when user says 'define new event', 'add function', 'insert mutation', or modifies entity blocks in topology.yml. Do NOT trigger for only attribute tweaks or non-entity YAML changes.
---
# Topology Entity Scaffolding

## Critical

1. Always update the `topology.yml` file under the correct entity section (`functions:`, `events:`, `mutations:`, `routes:`, etc) following the existing YAML indentation and naming conventions.
2. After modifying `topology.yml`, run `tc compose [dir]` to propagate changes into Rust provider modules.
3. Changes to entities must be reflected in the appropriate Rust source file within `lib/composer/src/aws/` or related directories — do NOT skip code migration.
4. Validate the YAML syntax with `yaml-lint` or another linter before proceeding.

## Instructions

### Step 1: Add or Update Entities in `topology.yml`
- Path: project root `topology.yml`
- Action:
  - Locate the relevant section (`functions:`, `events:`, etc).
  - Add a new entity with a unique name (snake_case), using the established indentation and attribute structure.
  - Follow examples in existing `topology.yml` entries.
- Validation:
  - Verify valid YAML parsing and no indentation errors.
- Depends on: None

### Step 2: Compose Topology to Reflect Changes in Rust Code
- Command: `tc compose [dir]` where `[dir]` is your topology directory (e.g., `.` for root).
- Outcome:
  - This regenerates Rust modules that represent topology entities.
  - Look for new or updated Rust files under `lib/composer/src/aws/` matching entity names (e.g., `event.rs`, `function.rs`).
- Validation:
  - Confirm compose log shows your new entity processed without errors.
  - Check `cargo build --workspace` passes.
- Depends on: Step 1 output

### Step 3: Implement or Update Entity Handler Code
- File Paths:
  - `lib/composer/src/aws/event.rs` for events
  - `lib/composer/src/aws/function.rs` for functions
  - And others for corresponding entity types
- Action:
  - Follow existing idiomatic Rust async patterns.
  - Add handlers, trait impls, or AWS bindings as per existing modules.
  - Use `lambda_runtime::{service_fn, LambdaEvent, Error}` pattern for functions.
- Validation:
  - Verify `cargo fmt` formatting.
  - Test entity handlers with `cargo test --workspace` or `make unit-test`.
- Depends on: Step 2 generated stubs

### Step 4: Test End-to-End With Deploy/Emulate
- Commands:
  - `tc build function --name <ENTITY_NAME>` to build function images.
  - `tc deploy entity --name <ENTITY_NAME>` to deploy.
  - `tc emulate function --name <ENTITY_NAME>` to locally emulate.
- Validation:
  - Confirm no error logs during build/deploy.
  - Emulation runs and invokes handler correctly.
- Depends on: Step 3 implementation

## Examples

**User says:** "Add new event called user_signup"

**Actions taken:**
1. Insert under `events:` in `topology.yml`:
```yaml
events:
  - name: user_signup
    source: cognito
    detail-type: "User SignUp"
```
2. Run `tc compose .` to generate Rust event module.
3. Update `lib/composer/src/aws/event.rs` to handle `user_signup` logic.
4. Test with `cargo test` and deploy using `tc deploy event --name user_signup`.

**Result:**
New event successfully added to topology and Rust codebase, ready for deployment.

## Common Issues

- **Error: YAML parse error in topology.yml**
  - Fix: Check indentation and dashes. Use `yamllint topology.yml`.

- **tc compose fails with "Entity name already exists"**
  - Fix: Ensure new entity name is unique and not conflicting.

- **cargo build fails after compose**
  - Fix: Review generated Rust code in `lib/composer/src/aws/` for missing imports or syntax errors.

- **Handler panics in lambda_runtime**
  - Fix: Wrap handler logic in `Result<Value, Error>`, ensure async main is correct per Rust function patterns.

- **`tc deploy` complains missing roles or policies**
  - Fix: Run scaffolding for roles/policies as needed with `tc create role --name <ROLE>` or check `.claude/skills/cloud-role-policy/SKILL.md`.

- **Changes not reflected after `tc compose`**
  - Fix: Confirm you edited the correct `topology.yml` file location. Re-run compose with explicit dir.

---

Run `cargo fmt` and `cargo test` frequently to keep code consistent with project conventions.