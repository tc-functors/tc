# `tc` Architecture

`tc` (Topology Composer) is a single Rust binary that turns a provider-agnostic
`topology.yml` graph into deployed AWS serverless infrastructure. It calls the AWS
SDK for Rust **directly** — no CloudFormation/CDK/Terraform — and **AWS is
hard-coded** (there is no provider trait; do not add one unless asked).

Mental model (from the docs): a complete, namespaced, versioned topology is a
**"Cloud Functor"**. You describe how entities *connect*; `tc` infers the
infrastructure, permissions, and orchestration. Relationships are primary,
implementation is derived.

## The pipeline — a three-stage narrowing
```
compile  →  compose  →  resolve  →  deploy
```
Each stage is a crate and a data-structure transformation. Every workflow command
(`create`, `invoke`, `test`, `update`, …) implicitly runs compile→compose→resolve
first (`tc compile | tc compose | tc resolve` is the explicit pipe form).

1. **compile** (`lib/compiler`) — walk the filesystem, validate, **intern**
   local/remote functions, infer missing fields. Cloud-agnostic. Isomorphic:
   `compile(&TopologySpec) -> TopologySpec` (same type in and out; missing fields
   inferred). Also has an embedded LISP front-end (WIP) and a bincode `.tc`
   intermediate.
2. **compose** (`lib/composer`) — build the entity **DAG**, choose connectors/shims
   for the configured provider, generate ASL for step-function flows. Produces the
   resolved `Topology`. Provider-bound but account-agnostic (a templated topology).
3. **resolve** (`lib/resolver`) — two layers: (a) mustache `{{account}}/{{region}}/
   {{namespace}}/{{sandbox}}` stencil substitution done by serialize→render→
   deserialize through JSON; (b) live AWS lookups (ARNs, SSM, layer versions, API-GW
   URLs). Bounded concurrency (`buffer_unordered`), `AsyncMemo` caches, `cacache` on
   disk at `/tmp/tc-resolver-cache`. Account-bound, self-contained.
4. **deploy** (`lib/deployer` + `lib/provider`) — idempotent `find_or_create_*` /
   `create_or_update_*` calls against the AWS SDK, chunked-concurrent via
   `tokio::spawn` + `FuturesUnordered`.

## The core data-model boundary (internalize this)
Two distinct top-level structs, one per side of the compose boundary:
- **`TopologySpec`** (`lib/compiler`) — unresolved. Almost every field is
  `Option<…>`; collections are `Option<HashMap<String, …Spec>>`. This is the
  compiler's input *and* output.
- **`Topology`** (`lib/composer`) — resolved. Fields are non-optional
  `HashMap<String, …>`; recursive via `nodes: HashMap<String, Topology>`.

Every `EntitySpec` collapses into its resolved twin across compose
(`FunctionSpec→Function`, `EventSpec→Event`, `RouteSpec→Route`, …). **Do not blur
these stages**: pre-resolution code deals in `…Spec`/`Option`; post-resolution code
deals in the concrete `HashMap`s.

## Entities → AWS constructs
| Entity | AWS construct | Deployer module | Provider module |
|---|---|---|---|
| routes | API Gateway v2 (HTTP API) | `deployer/src/aws/route.rs` | `provider/src/aws/gateway*` |
| events | EventBridge rules/targets | `deployer/src/aws/event.rs` | `provider/src/aws/eventbridge.rs` |
| functions | Lambda / Fargate MicroVM / AgentCore | `deployer/src/aws/function*` | `provider/src/aws/{lambda,microvm,agentcore}.rs` |
| channels | AppSync Events (WebSocket) | `deployer/src/aws/channel.rs` | `provider/src/aws/appsync/events.rs` |
| mutations | AppSync GraphQL | `deployer/src/aws/mutation.rs` | `provider/src/aws/appsync.rs` |
| queues | SQS (+ Lambda event-source) | `deployer/src/aws/queue.rs` | `provider/src/aws/sqs.rs` |
| schedules | EventBridge Scheduler → SFN | `deployer/src/aws/schedule.rs` | `provider/src/aws/scheduler.rs` |
| pools | Cognito user pools | `deployer/src/aws/pool.rs` | `provider/src/aws/cognito.rs` |
| flow/states | Step Functions | `deployer/src/aws/state.rs` | `provider/src/aws/sfn.rs` |
| pages | CloudFront + S3 | `deployer/src/aws/page.rs` | `provider/src/aws/{cloudfront,s3}.rs` |

The compute backend is selected by the `Provider` enum in
`lib/compiler/src/spec/function.rs` (`Lambda | MicroVm | AgentCore`) — a
compute-backend switch, **not** a cloud abstraction. `Auth`
(`lib/provider/src/aws/mod.rs`) is an AWS-specific struct with ~30 ARN-builder
methods.

## The transducer (entity transduction)
The same topology can act as an **orchestrator**. A `transducer:` key selects how:
- `Function` (default) — generates a Lambda ("misleadingly called the transducer")
  that drives data flow between entities.
- `ASL` — generates an Amazon States Language state machine (Step Functions).
- custom (e.g. `transducer: ./my-transducer.py`) — emits for an external
  orchestrator (Airflow, Flink).
Inspect with `tc compose -c transducer`.

## Permissions model
Layered IAM, plain JSON under `infrastructure/tc/…`, override precedence
**specific > namespaced > base**, plus a dynamic ABAC default (auto-generated
`tc-base-<entity>-<sandbox>` roles) and a global mode via `TC_LEGACY_ROLES`. Infra
never leaks into `topology.yml`.

## Workspace map (22 crates under `lib/` + the CLI in `src/`)
Pipeline core: `compiler`, `composer`, `resolver`, `deployer`, `provider`.
Build/runtime: `builder` (per-language Docker packing: python/ruby/node/go/rust;
janet via `provided.al2023`), `emulator` (Lambda RIE + SFN Local),
`invoker`/`tester`, `executor` (CircleCI trigger).
Support: `differ` (git-diff + dependency-closure change detection — the most
defensively engineered crate, with a typed `DiffError`), `inspector` (ratatui TUI),
`reflector`/`validator` (GraphQL introspection/validation), `scaffolder` (LLM
topology gen), `tagger`/`snapshotter`/`notifier` (release, version-tracking, Slack),
`router`, `repl`, `configurator` (shared config model), and **`kit`** — the
universal utility crate everything imports as `use kit as u`.

The CLI (`src/`) is ~35 `clap` subcommands (`Cmd` enum in `src/main.rs`) → a thin
`tc::*` facade in `src/lib.rs` → the lib crates. `src/mcp.rs` exposes 9 tools over
an `rmcp` stdio MCP server (`compose`, `build`, `create`, `update`, `delete`,
`invoke`, `test`, `resolve`, `changelog`).

## Release / audit model
Sandboxes + per-topology semver tags (`tagger`) + `freeze`/`snapshot --save` to S3 +
`ci-deploy` promotion. Manifests are the redeployable release unit. No external
state store; `tc prune` reconciles stale resources.

## Where things live (quick index)
- Data model: `lib/compiler/src/spec*` (`TopologySpec` + `…Spec`),
  `lib/composer/src/…` (`Topology` + resolved entities).
- Entity → AWS: `lib/deployer/src/aws/<entity>.rs` and
  `lib/provider/src/aws/<service>.rs`.
- Templating/resolution: `lib/resolver/src/{context,topology,function}.rs`.
- Utilities: `lib/kit/src/{core,io,json,memo,text,http,git}.rs`.
- CLI surface: `src/main.rs` (`Cmd`), `src/lib.rs` (`tc::*`), `src/mcp.rs`.
