---
name: rust-function-handler
description: Creates a new Rust Lambda-style handler following the async main and service_fn pattern. Use this skill when the user requests a new Rust AWS Lambda handler scaffold with the standard entrypoint (`src/main.rs`) and async handler function (`func`). Do NOT use for refactoring existing handlers or non-Lambda Rust projects.
---
# Rust Lambda Function Handler

## Critical

1. Always use `lambda_runtime` crate with `service_fn` and `LambdaEvent<Value>` as function signature.
2. Maintain a single async `main` function in `src/main.rs` using the `#[tokio::main]` macro.
3. The handler function must accept `LambdaEvent<Value>` and return `Result<Value, Error>`, where `Value` is from `serde_json::Value`.
4. Ensure dependencies include `lambda_runtime = "^0.5"`, `tokio = { version = "^1", features = ["macros"] }`, and `serde_json` in your `Cargo.toml`.

## Instructions

1. **Create the entry file `src/main.rs`:**
   - Use the exact async main boilerplate from the project.
   - Imports must include:
     ```rust
     use lambda_runtime::{service_fn, LambdaEvent, Error};
     use serde_json::{Value};
     ```
   - The `main` function MUST be annotated with `#[tokio::main]` and call `lambda_runtime::run` with a service function wrapping `func`.
   - Add an `async fn func(event: LambdaEvent<Value>) -> Result<Value, Error>` stub inside the same file (or imported from `handler.rs` if preferred).
   - Utilize `lambda_runtime` error type for main function error handling.
   
   _Verify_ that `cargo build` passes with no errors before proceeding.

2. **Optionally, create a `src/handler.rs` file** (if you want handler code separated):
   - Move `async fn func` there.
   - Import `lambda_runtime`, `serde_json` similarly.
   - Export the handler function, and import it in `main.rs`.
   
   _Verify_ module paths and crate imports resolve correctly.

3. **Update `Cargo.toml` dependencies:**
   - Ensure these dependencies exist:
     ```toml
     [dependencies]
lambda_runtime = "^0.5"
tokio = { version = "^1", features = ["macros"] }
serde_json = "^1"
     ```
   - If missing, add them and run `cargo build`.

4. **Add a minimal test if desired:**
   - Under `tests/` or `src/main.rs`, add a simple test to instantiate a dummy `LambdaEvent<Value>` and call `func`.
   - This validates the handler compiles and returns expected `Result<Value, Error>`.

5. **Validate by running:**
   - `cargo build --workspace`
   - `cargo test --workspace` (if test added)

6. **Document:**
   - Add a README snippet near the handler with usage instructions if appropriate.

## Examples

User says: "Generate a new Rust Lambda function handler scaffold for `src/main.rs`."

→ Actions taken:
- Created `src/main.rs` with async main and handler `func` following pattern:
```rust
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    Ok(event.payload)
}
```
- Verified dependencies in `Cargo.toml`.
- Built project successfully.

Result:
`src/main.rs` ready for AWS Lambda deployment with proper async Rust handler setup.

## Common Issues

- **Error:** `cannot find macro 'tokio::main'`
  - Fix: Ensure `tokio` dependency is in Cargo.toml with feature `macros` enabled:
    ```toml
tokio = { version = "^1", features = ["macros"] }
    ```

- **Error:** `lambda_runtime::run(func).await?;` unresolved or type mismatch
  - Fix: Verify import `use lambda_runtime::{service_fn, LambdaEvent, Error};` and async function signature matches `async fn func(event: LambdaEvent<Value>) -> Result<Value, Error>`

- **Build fails on missing `serde_json` symbols**
  - Fix: Add `serde_json = "^1"` to dependencies.

- **Function signature error: mismatched types or missing `LambdaEvent`**
  - Fix: Import `serde_json::Value` and use `LambdaEvent<Value>` exactly as in pattern.

- **Compilation error about `async` main missing `#[tokio::main]`**
  - Fix: Add `#[tokio::main]` macro above main.

- **If running build fails with missing `lambda_runtime` crate**
  - Fix: Add `lambda_runtime = "^0.5"` to dependencies.

Follow these exact patterns strictly to maintain consistency with project conventions and ensure successful builds and deploys.