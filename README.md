# tc
Contextual, serverless workflow tool

Note: This project is in active development.

Documentation: [https://informed-labs.github.io/tc/](https://informed-labs.github.io/tc/)

Video Presentation on tc from AWS Community Day - Bay Area Sept 2024  
[Higher Order Abstraction & Tooling for Step Functions & Serverless](https://youtu.be/1gqDGulszzQ?si=dtHcUkQF2nhZ_td8)

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
