# Architecture overview

## System boundary

`tc` consumes a topology definition (normally `topology.yml`), discovers functions and nested topologies, composes an AWS-shaped `Topology`, resolves sandbox/environment-dependent values, and then invokes builder, deployer, invoker, tester, or other capability crates. The root CLI is a thin command surface; orchestration belongs primarily in [`src/lib.rs`](../../src/lib.rs).

```text
topology.yml / topology.lisp
  → compiler::TopologySpec
  → composer::Topology
  → resolver rendering / deployment decisioning
  → builder packages function artifacts (when requested)
  → deployer applies AWS resources via provider SDK clients

resolved Topology → invoker/tester/visualizer/differ/snapshotter/etc.
```

## Major layers

| Layer | Responsibility | Main source |
|---|---|---|
| CLI | Parses commands and dispatches async handlers. | [`src/main.rs`](../../src/main.rs) |
| Application façade | Coordinates compose, resolve, auth, build, lifecycle actions, tests, and validation. | [`src/lib.rs`](../../src/lib.rs) |
| Compiler | Loads and transforms YAML (or Lisp), deserializing `TopologySpec`. `TC_SPEC_SIMPLE` bypasses the custom YAML transformer. | [`lib/compiler/src/lib.rs`](../../lib/compiler/src/lib.rs), [`spec.rs`](../../lib/compiler/src/spec.rs) |
| Composer | Discovers functions/nodes and derives resource-oriented topology, naming, roles, tags, and topology kind. | [`lib/composer/src/topology.rs`](../../lib/composer/src/topology.rs) |
| Resolver/configurator | Renders environment/sandbox-specific values and provides config used during composition/deployment. | [`lib/resolver`](../../lib/resolver/), [`lib/configurator`](../../lib/configurator/) |
| Builder | Packages code, images, layers, extensions, libraries, inline code, and MicroVM variants. | [`lib/builder/src/lib.rs`](../../lib/builder/src/lib.rs) |
| Deployer/provider | Encodes AWS resource actions and centralizes AWS SDK authentication/client conventions. | [`lib/deployer/src/lib.rs`](../../lib/deployer/src/lib.rs), [`lib/provider/src/aws/`](../../lib/provider/src/aws/) |

The workspace also contains focused capabilities: `differ`, `emulator`, `executor`, `invoker`, `notifier`, `repl`, `router`, `scaffolder`, `snapshotter`, `tagger`, `tester`, `validator`, and `visualizer` (canonical membership: [`Cargo.toml`](../../Cargo.toml)).

## Composition and recursion

`TopologySpec` holds both topology metadata and declared entities. Composition establishes namespace/FQN/version, inferred kind, roles, resource maps, schedules, pages, tests, flow, hooks, and configuration in one `Topology` object ([`make`](../../lib/composer/src/topology.rs)).

When recursive composition is requested, subdirectories become `Topology.nodes`. Shared functions are promoted from descendants to the root with first-wins collision behavior, then roles are recomputed. This is a high-impact area: it changes function ownership and IAM role derivation, not merely the presented tree ([`promote_shared_to_root`](../../lib/composer/src/topology.rs)).

## Deploy order is an operational contract

`deployer::create` provisions base roles (except stable sandboxes outside the base namespace), entity roles, functions, function-role associations, then integration entities: channels, mutations, queues, events, pools, routes, schedules, pages, state flow, and finally a transducer when present ([`lib/deployer/src/lib.rs`](../../lib/deployer/src/lib.rs)). `update` follows a similar path but updates function code. Preserve ordering unless the dependency relationship is understood and tested.

## Current architectural constraints

- **AWS is the implemented backend.** The README describes cloud-agnostic definitions, but the composed model and implementation are AWS modules today. A second provider would require more than another client—it would require a provider-neutral composition/deployment boundary.
- **Validation is intentionally narrow today.** `tc validate --entity mutation` composes recursively and validates generated GraphQL text; other entity values print `nothing`. It does not gate deployment. See [source map](../source-map.md).
- **Failure handling is often fail-fast.** Several code paths use `panic!`/`unwrap`, including missing/incompatible topology inputs. Treat library-facing error model changes as a broad design decision.

For YAML semantics and AWS entity mappings, continue to [topology model](../domain/topology-model.md). For change locations, use [source map](../source-map.md).