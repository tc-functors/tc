---
name: rust-function-handler
description: Creates a new Rust Lambda function handler using the project's standard async main with LambdaEvent pattern. Use this skill when the user requests 'new Rust handler', 'create function handler', or creates 'src/main.rs' or 'handler.rs'. Do NOT use this skill for editing existing handlers or non-Lambda Rust code. It scaffolds the full async entrypoint with error handling consistent with project conventions.
---
# Rust Lambda Function Handler

## Critical

1. Always use `lambda_runtime::{service_fn, LambdaEvent, Error}` crates as in project code.
2. Entry point must be an async `main` function with `#[tokio::main]` macro.
3. The handler function signature must be `async fn func(event: LambdaEvent<Value>) -> Result<Value, Error>` using `serde_json::Value`.
4. Return `Result<Value, Error>` with proper error propagation.
5. Do not scaffold code that deviates from this async + Error returning pattern.

## Instructions

1. **Create or overwrite `src/main.rs` as entrypoint**
   - Add the following imports exactly:
     ```rust
     use lambda_runtime::{service_fn, LambdaEvent, Error};
     use serde_json::Value;
     #[tokio::main]
     async fn main() -> Result<(), Error> {
         let func = service_fn(func);
         lambda_runtime::run(func).await?;
         Ok(())
     }
     ```
   - Verify that the `Cargo.toml` includes dependencies `lambda_runtime` and `serde_json` before proceeding.

2. **Add the handler function `func` in `src/main.rs`**
   - Signature:
     ```rust
     async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
         // Your handler logic here
         Ok(event.payload) // placeholder returns input event payload
     }
     ```
   - This function must match exactly for type signatures and async usage.
   - Ensure you have `use serde_json::Value;` at top.
   - Verify compilation with `cargo build` before next step.

3. **Confirm dependencies and features in `Cargo.toml`**
   - Verify dependency entries:
     ```toml
     [dependencies]
     lambda_runtime = "^0.5"
     serde_json = "^1.0"
     tokio = { version = "^1.0", features = ["macros"] }
     ```
   - Check `cargo build` passes with these dependencies.

4. **Test handler runs locally with `cargo test` or integration tests if applicable**
   - Run `cargo test --workspace` or relevant test commands to validate handler builds and integrates
   - Confirm no runtime errors due to async or type mismatches

5. **Commit scaffolded files following project formatting**
   - Run `cargo fmt` to apply Rust formatting standards
   - Ensure no errors or warnings from `cargo clippy` if used

## Examples

User says:
> Create a new Rust Lambda function handler for my project

Agent actions:
- Creates `src/main.rs` with async `main` and `func` using `lambda_runtime` and `serde_json::Value`
- Ensures dependencies in `Cargo.toml` include `lambda_runtime`, `serde_json`, and `tokio` with macros
- Runs `cargo build` to verify

Result: `src/main.rs` contains a fully functional async Lambda handler entrypoint matching project conventions.

```rust
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    // TODO: Add your handler logic here
    Ok(event.payload) // echo input
}
```

## Common Issues

- **Error:** "cannot find macro `tokio::main` in this scope"
  - Add `tokio` crate with features to `Cargo.toml`:
    ```toml
    tokio = { version = "^1.0", features = ["macros"] }
    ```
  - Run `cargo update` and rebuild.

- **Error:** "unresolved import `lambda_runtime`"
  - Verify `lambda_runtime` in `Cargo.toml` dependencies, add if missing:
    ```toml
    lambda_runtime = "^0.5"
    ```
  - Run `cargo build` after adding.

- **Error:** "mismatched types, expected `Result<Value, Error>`"
  - Confirm handler function signature exactly matches:
    ```rust
    async fn func(event: LambdaEvent<Value>) -> Result<Value, Error>
    ```
  - Wrap errors with `?` operator or return as `Err(Error)`.

- **Error:** "the trait bound `serde_json::Value: Serialize` is not satisfied"
  - Add `serde` with derive features to `Cargo.toml` if returning custom structs.
  - For simple `Value`, no extra steps needed.

- **Runtime panic or crash when running handler locally**
  - Ensure the function logic handles input events safely.
  - Check for awaited futures on async code.

Following these instructions will produce a Rust Lambda function handler consistent with your project's patterns and ready for build, test, and deploy.