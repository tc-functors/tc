---
name: cloud-role-policy
description: Create or customize AWS IAM roles and policies for new entities defined in topology YAML or implemented in Rust at lib/composer/src/aws/role.rs. Use this skill when adding new AWS entity types like functions, routes, events, or when updating role definitions for existing entities. It generates role ARNs and policy ARNs programmatically as per project conventions. Do NOT use for non-AWS or unrelated role management tasks.
---
# cloud-role-policy

## Critical

1. Always derive IAM role names and policy ARNs strictly from the topology entity name plus type using the existing naming pattern in `lib/composer/src/aws/role.rs`.
2. Modify or add role/policy definitions only inside `lib/composer/src/aws/role.rs` or closely related files to keep consistency.
3. Validate that the generated role ARNs exactly match AWS resource ARN conventions and use correct region/account placeholders.
4. Do NOT create duplicate roles or policies. Check existing entries in `role.rs` before adding new ones.
5. Test role changes with `cargo test --workspace` and by running `tc deploy <entity>` to confirm deployment correctness.

## Instructions

1. **Identify the new entity requiring a role or policy**
   - Locate the entity name and type in `topology.yml` or Rust entity module under `lib/composer/src/aws/`.
   - Verify the new entity's AWS resource requirements (e.g., Lambda function, API Gateway route).
   - Verify this entity is not already covered by existing roles in `lib/composer/src/aws/role.rs`.

2. **Open `lib/composer/src/aws/role.rs` for editing**
   - Confirm import dependencies at file top align with existing pattern (e.g., `use crate::topology::EntityName;`).
   - Copy the code structure of an existing role/policy definition function for similar entity type.
   - Verify formatting with `cargo fmt` before continuing.

3. **Add or customize role and policy generation code**
   - Follow the established naming pattern: roles and policies are named using entity type + topology service name, e.g., `arn:aws:iam::${account_id}:role/${topology_name}-${entity_type}-${entity_name}`.
   - Generate the exact role ARN and policy ARN strings programmatically.
   - Include all required AWS permissions scoped to the entity's AWS resources.
   - Wrap permission statements in appropriate Rust types and serialize with `serde` if required.
   - Verify role and policy structs implement the expected traits and serialization.

4. **Integrate the new role/policy with the topology composition and deploy flow**
   - Ensure that `lib/composer/src/aws/role.rs` exposes functions to retrieve role ARNs for the new entity.
   - Verify that `tc deploy` command workflows in `lib/deployer` call these functions to assign roles correctly.
   - Run `cargo test --workspace` to validate no compilation or test failures.

5. **Validate in local deploy or emulator environment**
   - Run `tc deploy entity --name <new_entity_name>` to deploy with updated role.
   - Confirm AWS IAM roles are created with expected policies.
   - Use AWS CLI or console to spot-check role permissions.
   - Fix errors related to ARN formatting or missing permissions in `role.rs`.

6. **Commit changes with standardized commit message**
   - Use message prefix `feat(role): add IAM role/policy for <entity_type> <entity_name>`.
   - Push and open PR for review ensuring coding style matches existing files.

## Examples

**User says:**
> Add a new Lambda function entity called "payment_processor" needing custom IAM role

**Actions taken:**
- Locate `payment_processor` entity in `topology.yml`
- Edit `lib/composer/src/aws/role.rs`
- Copy structure from existing function role generator
- Create new function generating role ARN `arn:aws:iam::${account_id}:role/${topology_name}-function-payment_processor`
- Add permission policy including Lambda invoke and DynamoDB access
- Test with `cargo build` and `tc deploy function --name payment_processor`

**Result:**
- New IAM role and policy created in AWS matching project conventions
- Deployment succeeded with no role permission errors

## Common Issues

- **Error: "Role ARN malformed or missing ${account_id}"**
  - Fix: Check that the role ARN string in `role.rs` properly interpolates the AWS account ID variable.
  - Verify code uses correct variables from topology context.

- **Error: "Permission denied for Lambda invocation"**
  - Fix: Add `lambda:InvokeFunction` permission in the IAM policy statements for the function.

- **Duplicate role creation errors on deploy**
  - Fix: Confirm no duplicate role generation code exists for the same entity name in `role.rs`.

- **Cargo test failures after role.rs changes**
  - Fix: Run `cargo fmt` and ensure all role structs implement expected traits (`Serialize`, etc.)

- **`tc deploy` fails with missing role ARN**
  - Fix: Confirm new role generator function is imported and used in deploy workflow in `lib/deployer/src/aws/` files.


Use this skill template to maintain consistent AWS IAM roles and policies aligned with project topology and Rust codebase conventions.