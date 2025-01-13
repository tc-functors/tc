

Cloud Functors are toplogies `composed` of nano functions, states, events and routes. These nano functions (Lambdas) capture enough business logic and offload IO and state management to an orchestrator (typically Stepfns).


`tc` is a cli tool, written in Rust, that implements the `cloud functors` pattern and enables sophisticated dev, release and debugging workflows.

## Features

![Features](/images/features.png)


### Commands overview

```sh
tc
Usage: tc <COMMAND>

Commands:
  build      Build layers and pack function code
  clean      Clean current topology directory (remove layers, zip etc)
  compose    Compose a topology from functions, events, states description
  create     Create a sandboxed topology or functor
  delete     Delete a sandboxed topology or functor
  deploy     Deploy a sandboxed topology via CircleCI
  diff       Diff sandbox versions, changelogs scoped by topology
  freeze     Freeze a sandbox and make it immutable
  invoke     Invoke a topology synchronously or asynchronously
  lint       Lint functions in the topology dir
  release    Release a topology and generate a minor version via CircleCI
  route      Route events to functors
  tag        Create semver tags scoped by a topology
  test       Run unit tests for functions in the topology dir
  unfreeze   Unfreeze a sandbox and make it mutable
  upgrade    upgrade tc version
  version    display current tc version
  help       Print this message or the help of the given subcommand(s)
```
