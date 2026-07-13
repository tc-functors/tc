# Topology model and domain concepts

## The authoring unit

A `topology.yml` is the declarative input to `tc`. [`TopologySpec`](../../lib/compiler/src/spec.rs) owns its top-level shape: identity/configuration (`name`, `version`, `infra`, `kind`, `mode`, `hyphenated_names`, `concurrent`), recursive nodes, functions, events, routes, mutations, queues, channels, triggers, pages, tests, states/flow, and sequences.

The file is transformed/deserialized by compiler code, then enriched by the composer into a deployable topology. That distinction matters: adding a YAML key alone normally does not change runtime behavior.

## Topology kind and entity graph

The composer infers a topology kind when it is not specified: state flow yields `StepFunction`; otherwise mutations yield `Graphql`, routes yield `Routed`, functions yield `Function`, and the fallback is `Evented` ([`find_kind`](../../lib/composer/src/topology.rs)). The kind influences freeze checks and lifecycle behavior.

The README’s ETL example demonstrates the intended graph style: route → functions → event → channel. Current AWS mappings described in [`README.md`](../../README.md) include:

| Topology concept | AWS target described by repository docs |
|---|---|
| routes | API Gateway |
| functions | Lambda or ECS Fargate |
| events | EventBridge |
| channels | AppSync Events |
| mutations | AppSync GraphQL |
| queues | SQS |

The actual composed topology also has schedules, pools, pages, flows/state machines, roles, tags, hooks, tests, sequences, and a possible transducer. The function runtime path now includes Lambda, MicroVM, and an initial Bedrock AgentCore option (see `examples/functions/python-microvm-deps/function.yml` for a MicroVM declaration). Avoid relying on the README’s older claim of exactly seven/eight entities; the spec and composer are the accurate current shape.

## Function discovery and nested composition

A source directory can be a full topology, relative topology, standalone function, or singular function directory. [`Topology::new`](../../lib/composer/src/topology.rs) selects the case and either discovers functions or uses explicit/interned functions. Recursive mode constructs nodes below the root. Shared functions from descendant nodes are promoted upward and roles recomputed; duplicate names use first-wins behavior with a debug log on distinct FQNs.

**Change rule:** function naming, `shared`, FQN templates, sandbox names, and role construction affect resource lookup, generated names, IAM ownership, and deployment targeting. Trace all of compiler → composer → resolver → deployer/provider before changing any of them.

## GraphQL mutations

Mutations have `authorizer`, `inputs`, `types`, and `resolvers` in the compiler spec. The recent mutation example ([`examples/mutations/topology.yml`](../../examples/mutations/topology.yml)) shows named GraphQL input objects and resolver `input`/`output` fields. Composer code builds GraphQL input/type strings and resolver fields; provider code wires Lambda-backed AppSync integration.

Recent history added `inputs` specifically so a resolver can use GraphQL `input` definitions rather than only object types. `tc validate --entity mutation` is currently the only validation path and works on generated GraphQL schema text, not general topology semantics.

## Tests as topology data

`tests` are part of a topology, not Rust unit-test definitions. Each case identifies an entity (for example `functions/foo`, `state`, or `routes/ping`), payload, expected response, and condition. [`examples/tests/topology.yml`](../../examples/tests/topology.yml) shows `matches` and `includes`; the tester also interprets other conditions as JSONPath assertions. These tests invoke AWS services through resolved names, so they require the deployed target rather than being offline tests.

## Representative examples

- Composition patterns: [`examples/composition/`](../../examples/composition/)
- Function runtime/build modes: [`examples/functions/`](../../examples/functions/)
- State-machine patterns: [`examples/states/`](../../examples/states/)
- Application patterns: [`examples/patterns/`](../../examples/patterns/)
- Static/web pages: [`examples/pages/`](../../examples/pages/)
- Stores: [`examples/stores/`](../../examples/stores/)

Use examples as behavioral fixtures, but check [`examples/composition/status.org`](../../examples/composition/status.org) for stated maturity caveats before treating every combination as supported.