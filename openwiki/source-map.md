# Source map and change impact

## Start from the change type

| If you need to change… | Start here | Then trace into… |
|---|---|---|
| CLI command or arguments | [`src/main.rs`](../src/main.rs) | Handler and orchestration in [`src/lib.rs`](../src/lib.rs) |
| Top-level YAML schema/entity spec | [`lib/compiler/src/spec.rs`](../lib/compiler/src/spec.rs), `lib/compiler/src/spec/` | Composer construction, resolver/deployer behavior, example YAML |
| Topology identity, recursive nodes, shared functions, roles | [`lib/composer/src/topology.rs`](../lib/composer/src/topology.rs) | AWS function/role templates, resolver, deployment lifecycle |
| Function runtime or build kind | `lib/compiler/src/spec/function.rs`, `lib/composer/src/aws/function/` | [`lib/builder/src/`](../lib/builder/src/), deployer function implementation |
| AWS service/API call | [`lib/provider/src/aws/`](../lib/provider/src/aws/) | Composer model and matching `lib/deployer/src/aws/` action |
| Provisioning/update/delete ordering | [`lib/deployer/src/lib.rs`](../lib/deployer/src/lib.rs) | Entity-specific `lib/deployer/src/aws/` and provider modules |
| Sandbox/config resolution | [`lib/resolver/`](../lib/resolver/), [`lib/configurator/`](../lib/configurator/) | Composer templates and CLI lifecycle calls |
| Invocation or topology testing | [`lib/invoker/`](../lib/invoker/), [`lib/tester/src/lib.rs`](../lib/tester/src/lib.rs) | `src/lib.rs`, YAML tests in examples |
| GraphQL mutation support | `lib/compiler/src/spec/mutation.rs`, [`lib/composer/src/aws/mutation.rs`](../lib/composer/src/aws/mutation.rs) | `lib/provider/src/aws/appsync/`, validator, mutation example |

## Workspace map

The canonical workspace list is in [`Cargo.toml`](../Cargo.toml). Its main groups are:

- **Core representation:** `kit`, `compiler`, `composer`, `resolver`, `configurator`.
- **Artifact and cloud lifecycle:** `builder`, `deployer`, `provider`, `differ`, `snapshotter`, `tagger`, `notifier`, `router`.
- **Execution/support:** `invoker`, `tester`, `emulator`, `executor`, `visualizer`, `scaffolder`, `repl`, `validator`.

`README.md` contains an older, smaller library list (including names not present in the current workspace). Use the manifest and source tree for ownership decisions.

## High-risk change zones

### Naming and identity

Namespace, FQN formatting, sandbox, version, hyphenated names, tags, and role ARN composition can alter what existing cloud resource is selected or created. Follow the full pipeline and use non-stable sandbox/diff review. Relevant code begins in composer templates/topology and provider ARN helpers.

### Recursive/shared functions

`Topology::new` can build nested nodes; recursive composition promotes descendant shared functions to root and recomputes roles. Changing discovery, collision behavior, or ownership can break deployment associations even when the YAML tree still renders.

### Lifecycle ordering

Create/update contain explicit sequencing for roles, functions, integrations, pages, flows, and transducers. Do not reorder based only on apparent alphabetical entity grouping. Verify both create and update; inspect delete coverage as a separate behavior.

### Validation and tests

The new validator has mutation-only scope and is diagnostic-print based. The topology tester reaches live AWS resources. A change that relies on either mechanism needs an explicit test strategy instead of assuming they provide exhaustive local validation.

## Recent repository evolution (high-signal)

The recent history at the initialized revision shows active expansion of deployment capabilities rather than a static CLI:

- **Mutation inputs:** a recent change added `mutations.inputs`, composer generation of GraphQL `input` objects, and matching AppSync/Lambda wiring. Use [`examples/mutations/topology.yml`](../examples/mutations/topology.yml) as the current shape.
- **Validation command:** `validator` crate and `tc validate` were added with stated mutation-only support. Preserve this limitation in UX/docs until broader validation is implemented.
- **Schedules:** a follow-up corrected schedule role ARN handling, reinforcing that composer-generated ARN values must already be correctly shaped before deployment.
- **AgentCore and MicroVM work:** recent commits added AgentCore function examples/provider support and evolved MicroVM build behavior (Dockerfile, dependency/pre/post build handling). Function build changes span compiler, composer, builder, deployer, provider, and examples—not one crate.
- **AWS SDK/S3 maintenance:** recent dependency bumps and S3 timeout fixes indicate provider operations have active reliability concerns.

These points explain current seams but are not an exhaustive commit log. Use targeted `git log`/`git show` around the code you change.

## Documentation sources and caveats

- [`README.md`](../README.md) provides product intent and an illustrative topology, but its CLI and crate inventory lag source.
- [`AGENTS.md`](../AGENTS.md) provides local contribution/verification constraints, but its Rust version line conflicts with `Cargo.toml`/`Makefile`.
- `examples/` is useful executable evidence, with uneven maturity documented in `examples/composition/status.org`.

When sources disagree, favor current manifests and implementation; record any user-visible incompatibility in the change description.