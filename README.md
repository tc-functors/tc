# tc
A graph-based, contextual, application & infrastructure composer. tc is both a Rust library and a cli app.


[![Build](https://github.com/informed-labs/tc/actions/workflows/ci.yml/badge.svg)](https://github.com/informed-labs/tc/actions/workflows/ci.yml)

## Resources

Documentation: [https://informed-labs.github.io/tc/](https://informed-labs.github.io/tc/)

Video Presentation on tc from AWS Community Day - Bay Area Sept 2024
[Higher Order Abstraction & Tooling for Step Functions & Serverless](https://youtu.be/1gqDGulszzQ?si=dtHcUkQF2nhZ_td8)

## Basic Usage


```sh
Usage: tc <COMMAND>

Usage: tc <COMMAND>

Commands:
  bootstrap  Bootstrap IAM roles, extensions etc
  build      Build layers, extensions and pack function code
  cache      List or clear resolver cache
  compile    Compile a Topology
  config     Show config
  create     Create a sandboxed topology
  delete     Delete a sandboxed topology
  freeze     Freeze a sandbox and make it immutable
  emulate    Emulate Runtime environments
  inspect    Inspect via browser
  invoke     Invoke a topology synchronously or asynchronously
  list       List created entities
  publish    Publish layers
  resolve    Resolve a topology from functions, events, states description
  route      Route events to functors
  scaffold   Scaffold roles and infra vars
  test       Run unit tests for functions in the topology dir
  tag        Create semver tags scoped by a topology
  unfreeze   Unfreeze a sandbox and make it mutable
  update     Update components
  upgrade    upgrade tc version
  version    display current tc version
  doc        Generate documentation
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
## History and Roadmap

![Roadmap](doc/images/tc-roadmap.png)


Note: this project is still quite nascent and is being actively developed
