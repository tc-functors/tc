# Topology lifecycle workflows

Run these commands from the relevant topology or function directory unless an option supplies another directory. The implemented CLI surface is in [`src/main.rs`](../../src/main.rs); README command output is incomplete relative to current code.

## 1. Inspect before mutating

```sh
tc compile
tc compose
tc resolve
tc diff
```

- `compile` loads the source specification and prints it.
- `compose` constructs the topology/resource model; use recursive/format/entity options as supported by the CLI.
- `resolve` renders values for an environment/sandbox and is used before lifecycle mutation.
- `diff` is the appropriate preflight comparison when working with existing sandbox state.

Use inspection commands first when changing definitions, especially where names, roles, or nested/shared functions are involved. The exact option grammar is source-driven from Clap structs in [`src/main.rs`](../../src/main.rs).

## 2. Build artifacts

```sh
tc build
# or, when intentionally building a topology tree:
tc build --recursive
```

The application façade determines whether this is cleanup, image sync, shell, promotion, one-function build, or a recursive build ([`src/lib.rs`](../../src/lib.rs)). Builder dispatch supports code, image, inline, layer, library, extension, and MicroVM build kinds. Recursive building currently processes root-level functions serially even though it accepts a `parallel` parameter; do not assume parallel execution.

Build/publish/sync paths initialize centralized AWS auth, and may need a configured profile/layer configuration. They are not purely local compilation steps.

## 3. Deploy lifecycle

```sh
# Cloud-affecting; run only with explicit intent and appropriate AWS profile/sandbox.
tc create
tc update
tc delete
```

The typical lifecycle is compose → authenticate → resolve → deploy. Create/update provision AWS resources in a prescribed order; see [architecture overview](../architecture/overview.md). Start with `tc diff` and inspect resolved output where possible.

### Stable sandbox safety

The deployer blocks stable-sandbox creation/update outside CI by default. In CI, `TC_FREEZE` enables the block; outside CI, `TC_FORCE_DEPLOY` disables it. The code calls this safety behavior `prevent_stable_updates` ([`lib/deployer/src/guard.rs`](../../lib/deployer/src/guard.rs)). Do not set bypass variables casually: their purpose is to prevent accidental stable deployment.

Other operational commands include `freeze`, `unfreeze`, `prune`, `route`, `snapshot`, `list`, `tag`, and `changelog`. Treat all as potentially environment- or deployment-affecting until their implementation is reviewed.

## 4. Invoke and test a deployed topology

```sh
tc invoke
tc test
# current validation scope:
tc validate --entity mutation
```

`tc test` composes and resolves first, then uses AWS Lambda, Step Functions, EventBridge, or HTTP route invocation depending on the test entity ([`lib/tester/src/lib.rs`](../../lib/tester/src/lib.rs)). Test cases live in topology YAML; see [topology model](../domain/topology-model.md).

`validate` currently supports only mutation/GraphQL schema validation. It prints diagnostics rather than establishing a general validation gate, so it supplements rather than replaces composition review and tests.

## 5. Examples as workflow starting points

Use the smallest targeted example directory rather than starting from the entire examples tree:

- State features: [`examples/states/`](../../examples/states/), including a documented distributed-map flow under `map-dist/`.
- Mutation schema inputs: [`examples/mutations/`](../../examples/mutations/).
- Topology test syntax: [`examples/tests/`](../../examples/tests/).
- Runtime/build variations: [`examples/functions/`](../../examples/functions/).

Avoid bulk create/update/delete across examples. They can create cloud resources and their support status is uneven.