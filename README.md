# tc
A graph-based, contextual, infrastructure composer. tc is both a Rust library and a cli app.

### Abstract

`tc` composes infrastructure using 6 core serverless primitives viz. `event`, `function`, `mutation`, `state`, `route` and `queue`. These primitives are [first-class](https://en.wikipedia.org/wiki/First-class_function) and can be composed like higher-order functions in functional programming. These primitives can be thought of nodes in an acyclic graph (DAG) and are agnostic of underlying infrastructure. For example, one can define a topology as follows

```yaml
name: my-pipeline
events:
  consumes:
    EventA:
      function: my-lambda-or-container-function
    EventB:
      mutation: Mutation-0

routes:
  apiA:
    path: /api
    function: my-lambda-proxy
  apiB:
    path: /hooks
    queue: QueueA

queues:
  dlq:
    consumer:
      function: Function-A

mutations:
  resolvers:
    Mutation-0:
      function: Function-B
      input: Event
      output: SomeOutputType
      subscribe: true

states:
  States Language (ASL)

```

No infrastructure or underlying implementation has leaked into this definition. `tc` takes this topology specification, scans the current directory recursively for `functions` (Python3.X, Ruby3.2, Clojure, Janet, Go are currently supported), layers the dependencies and deploys the topology as a single unit. A tc topology can have tens or hundreds of functions, events, mutations, state transitions (Step functions), API routes etc. `tc` also sandboxes the deployment using convention. This also allows canary-style deployments and routing.

### Resources

Documentation: [https://informed-labs.github.io/tc/](https://informed-labs.github.io/tc/)

Video Presentation on tc from AWS Community Day - Bay Area Sept 2024
[Higher Order Abstraction & Tooling for Step Functions & Serverless](https://youtu.be/1gqDGulszzQ?si=dtHcUkQF2nhZ_td8)

### Basic Usage


```sh
Usage: tc <COMMAND>

Commands:
  bootstrap  Bootstrap IAM roles, extensions etc
  build      Build layers, extensions and pack function code
  compile    Compile a Topology
  create     Create a sandboxed topology
  delete     Delete a sandboxed topology
  emulate    Emulate Runtime environments
  invoke     Invoke a topology synchronously or asynchronously
  list       List created entities
  publish    Pulish layers, slabs and assets
  resolve    Resolve a topology from functions, events, states description
  scaffold   Scaffold roles and infra vars
  test       Run unit tests for functions in the topology dir
  update     Update components
  upgrade    upgrade tc version
  version    display current tc version
  help       Print this message or the help of the given subcommand(s)
```

Note: this project is still quite nascent and is being actively developed
