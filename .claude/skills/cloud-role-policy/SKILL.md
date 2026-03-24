---
name: cloud-role-policy
description: Creates or overrides AWS IAM role and policy Rust modules for new topology entities that require unique cloud permissions. Use when a user adds a new AWS resource or entity requiring specific IAM roles/policies, or mentions 'role', 'policy', or 'new entity' related to cloud deployment. Do NOT use for non-AWS resources or permissions.
---
# Cloud Role Policy

## Critical
- Only update or add IAM role/policy logic within `lib/composer/src/aws/role.rs` using the project's Rust conventions. Never reference or construct ARNs or documented role strings directly in documentation or skills text. Do not describe full ARN patterns; always direct work to the correct Rust codebase location.
- Role and policy names are always derived programmatically from entity/context in Rust. Never duplicate role logic for the same entity.

## Instructions

1. **Locate or Add Role Logic**
   - Open project file `lib/composer/src/aws/role.rs`.
   - Copy the structure used for existing functions, e.g.,
```rust
pub fn payment_processor_role_name(svc: &str) -> String {
    format!("{}-payment-processor-role", svc)
}
pub fn payment_processor_policy() -> serde_json::Value {
    serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [
            {"Effect": "Allow", "Action": ["lambda:InvokeFunction", "dynamodb:*"], "Resource": "*"}
        ]
    })
}
```

2. **Reference and Integrate Role/Policy in Entity Code**
   - Update handler code in files like `lib/composer/src/aws/function.rs` to assign the new role by calling the relevant function from `role.rs`.

3. **Test and Deploy**
```sh
cargo test --workspace
tc deploy function --name payment_processor
```

4. **Commit**
- Use commit message:
```
feat(role): add IAM role/policy for function payment_processor
```

## Examples
- Add new role/policy in `lib/composer/src/aws/role.rs`, reference from related entity Rust code, test, and deploy as above.

## Common Issues
- Compile/test failures? Run `cargo fmt && cargo test`.
- Deploy issues? Confirm entity code calls new role/policy code and uses correct function from `role.rs`.

---
For complete pipeline integration, run
```sh
make integration-test
```
