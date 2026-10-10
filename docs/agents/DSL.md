# `tc` DSL Reference — `topology.yml` & `function.yml`

Authoritative schema for the user-facing YAML, cross-checked against the docs
(tc-functors.org) and the `examples/` tree. When you change behavior that affects
this surface, update the matching `examples/` case in the same diff.

## `topology.yml` — top-level keys
| Key | Type | Notes |
|---|---|---|
| `name` | String (required) | topology / namespace name |
| `infra` | path | infra config dir (vars, roles); auto-discovered in the git repo if omitted |
| `recursive` | bool (default false) | recursively discover functions |
| `function_dirs` | [path] | logical dirs to scan/intern functions (tc is non-recursive by default) |
| `functions` | Map<Name, FunctionSpec> | see below |
| `events` | Map<Name, EventSpec> | |
| `routes` | Map<PathOrName, RouteSpec> | |
| `mutations` | MutationSpec (`authorizer`,`inputs`,`types`,`resolvers`) | AppSync GraphQL |
| `queues` | Map<Name, {function}> | SQS |
| `channels` | Map<Name, {function}> | AppSync Events |
| `pages` | Map<Name, PageSpec> | SPA/PWA/Static frontends |
| `states` / `flow` | path \| inline ASL | Step Functions; `kind: step-function` uses `flow:` |
| `stores` | Map<Name, {kind,bucket,…}> | DynamoDB/S3 (0.10.x) |
| `tests` | Map<Name, TestSpec> | topology-level tests (require `entity`) |
| `transducer` | `Function`\|`ASL`\|path | orchestration override (see ARCHITECTURE.md) |
| `mode` / `kind` | String | `mode: Express`; `kind: step-function` |

Include macros (recursive, usable in included files):
`!include ./file.yml` (inline a block), `!read ./file.yml` (splice a partial block),
`!mutations ./file.yml` (merge MutationSpec fragments), `!sexp ./topology.lisp`.

## Composition grammar
An entity references a target by name under a target-type key. Legal edges
(source → target), enumerated in `examples/composition/**`:
- **function** → function, event, mutation
- **event** → function(s), channel, mutation, state
- **route** → function, event, mutation, queue, state
- **queue** → function
- **channel** → function
- **mutation** → function (resolver), event
- **page** → route

```yaml
name: etl
routes:
  /api/etl: { method: POST, function: enhancer }
functions:
  enhancer: { function: transformer }
  transformer: { function: loader }
  loader: { event: Notify }
events:
  Notify: { channel: Subscription }
channels:
  Subscription: { function: default }
```

## `functions` (FunctionSpec)
Defined as a dir (with `handler.{py,rb,js,clj}` + optional `function.yml`), interned
in `topology.yml`, inline "nano" (`runtime.code`), or standalone. Key fields:

| Field | Type | Default | Notes |
|---|---|---|---|
| `name` | String | — | |
| `uri` | String | `file:./lambda.zip` | dir for interned, `github.com/...` for remote |
| `runtime.lang` | String | inferred | `python3.10–3.14`, `ruby3.2/3.4`, `node20/22`, `go`, `rust`, `janet`, `clojure1.10`, `java21` |
| `runtime.handler` | String | `handler.handler` | `file.func`; a shell command for MicroVm |
| `runtime.package_type` | `zip`\|`image` | `zip` | |
| `runtime.memory` | int | 128 | |
| `runtime.timeout` | int | 30 | |
| `runtime.snapstart` | bool | false | |
| `runtime.layers` | [String] | [] | pin as `name:version` |
| `runtime.extensions` | [String] | [] | ARNs or `ssm:/…` URIs |
| `runtime.environment` | Map | {} | also per-sandbox in vars file |
| `runtime.network` | bool | false | VPC (subnets/SGs in infra vars) |
| `runtime.provider` | `Lambda`\|`MicroVm`\|`AgentCore` | Lambda | |
| `runtime.arch` | `Arm64`\|… | | |
| `runtime.fs` | `{kind,bucket,mount_point}` | | attached filesystem |
| `build.kind` | enum | | `Code`,`Inline`,`Layer`,`Library`,`Slab`,`Extension`,`Image`,`MicroVmImage`,`Runtime` |
| `build.command` | String | | pack cmd, e.g. `zip -9 -q lambda.zip *.py` |
| `build.pre` / `build.post` | [String] | | shell hooks (system deps / S3 pulls) |
| `test` | Map<Name, TestSpec> \| hooks | | see tests |
| `tasks` | Map<Name, String> | | named shell tasks (`clean`, `lint`, `test`) |

Deploy-time overrides (env, memory, network) live in
`infrastructure/tc/<ns>/vars/<fn>.json`, not in `function.yml`; roles in
`.../roles/<fn>.json`. `ssm://…` URIs in vars resolve at create/update.

## `events` (EventSpec)
`producer`/`producers` (`default` bus, or a trigger like `S3/PUT_OBJECT`,
`Cognito/PRE_SIGNUP`, `DYNAMODB/PUT_ITEM`), `filter` (JSON-path pattern) or
`pattern` (raw JSON), and one target: `function`/`functions`, `mutation`, `state`,
`channel`. Optional `rule_name`, `doc_only`. Schedules live in
`{INFRA_DIR}/schedules.json` (`cron`, `target`, `payload`).

## `routes` (RouteSpec)
`method` (`GET|POST|PUT|DELETE`), `path` (when key is a name), `authorizer`
(function name or `cognito`), `async` (default false), one target
(`function`/`state`/`queue`/`event`), `request_template`/`response_template`,
`request_params`/`response_params`, `stage`/`stage_variables`,
`CORS: {methods, origins, headers}`, `gateway`, `vertical`. A `default:` key sets
inherited defaults. Domains/throttling live in `{INFRA_DIR}/routes.json`.

## `mutations` (MutationSpec)
`authorizer` (function name), `inputs` (Map<Name, {field: GqlType}>), `types`
(same shape; directives inferred), `resolvers` (Map<Name, {function, input, output,
subscribe}>). GraphQL scalars: `String`, `String!`, `[T!]!`, `AWSJSON`. Implicit
event-input types: `Event` (`$.detail`), `EventData` (`$.detail.data`),
`EventDataJSON` (AWSJSON), `EventMetadata`.

```yaml
mutations:
  authorizer: authorizer-fn
  types: { Status: { id: String!, message: String } }
  resolvers:
    updateStatus: { function: updater, input: Input, output: Status, subscribe: true }
```

## `queues` / `channels`
```yaml
queues:   { my-queue: { function: consumer } }   # SQS; DLQ via a function's `queue:` field
channels: { my-room:  { function: path/to/handler.js } }   # AppSync Events, inline JS handler
```

## `pages` (PageSpec)
`kind` (`SPA|PWA|Static`), `dir` (source, required), `dist`, `build` ([cmds]),
`functions.request`/`functions.response` (edge JS), `config_template` (dotenv,
rendered per sandbox), `domains` (env→host), `bucket` (prefer `TC_PAGES_BUCKET`).
Template vars in `config_template`: `GRAPHQL_ENDPOINT/API_KEY/WSS_ENDPOINT`,
`REST_ENDPOINT`, `CHANNEL_URL`, `REGION`, `ACCOUNT`.

## `states` / `flow`
Standard Amazon States Language under `states:` (or `flow:` with
`kind: step-function`); `mode: Express` selects Express workflows. Tasks invoke
Lambda by templated name (`FunctionName: '{{namespace}}_foo_{{sandbox}}'`).
Implicit flow: a `functions:` chain with `function:`/`event:`/`root: true` keys
generates the ASL automatically. Supports `Task` (+`.waitForTaskToken`),
`Parallel`, `Map` (inline + DISTRIBUTED with CSV `ItemReader`), `Choice`, `Pass`,
`Succeed`, and nested `states:startExecution.sync:2`.

## Tests
Topology-level (`tests:` — `entity` mandatory) and function-level (`test:`):
```yaml
tests:
  case1:
    entity: functions/foo        # functions/<n> | routes/<n> | state
    payload: '{"foo":"bar"}'     # inline JSON | file path | s3:// URI
    condition: matches           # matches (deep eq) | includes (subset) | a JSONPath expr
    expect: '{"foo":"bar"}'
```
Implemented targets: `functions`, `state`, `routes` (events/mutations pending).

## Conventions
- `.tcignore` — newline-delimited dirs excluded from the topology scan (like
  `.gitignore`).
- `infra` overlays are raw JSON: `hooks.json` (`pre`/`post`), `routes.json`,
  `vars/<fn>.json`, `roles/<fn>.json`, `schedules.json`, `tags.json`.
- Template vars everywhere: `{{namespace}}`, `{{sandbox}}`, `{{region}}`,
  `{{account}}`, `{{env}}`.
