# tc (Topology Composer)

tc is a graph-based, executable architecture description language and framework for cloud-native serverless systems, with fractal composition and infrastructure generation capabilities.

tc (topology composer) provides a higher-level abstraction for serverless development, focusing on the logical relationships between abstract entities (functions, events, routes, queues, channels, mutations and pages) rather than the underlying infrastructure details.

[![Build](https://github.com/tc-functors/tc/actions/workflows/ci.yml/badge.svg)](https://github.com/tc-functors/tc/actions/workflows/ci.yml)
[![Ask DeepWiki (useful but watch  for hallucinations)](https://deepwiki.com/badge.svg)](https://deepwiki.com/tc-functors/tc)

tc's core value proposition is enabling developers to focus on business logic and component relationships rather than infrastructure management, while maintaining the ability to deploy consistently across environments and providers.

`tc` enables developers to compose cloud applications using high-level abstractions called `Cloud Functors` without getting bogged down in provider-specific infrastructure details.

The central concept in `tc` is the `Cloud Functor` - a namespaced, sandboxed, versioned, and isomorphic topology of serverless components. The term "functor" is borrowed from OCaml's parameterized modules, emphasizing first-class, composable units.

## Key features

**Entity Abstraction**

At it's core, `tc` provides 7 entities (functions, events, mutations, queues, routes, states and channels). Entities can be thought of like atoms. They are core cloud primitives which abstract away all the low-level details. Entities are defined in a cloud-agnostic way. tc provides eight entities that are sufficient to build sophisticated serverless topologies.


**Namespacing**

The above entities can be namespaced arbitrarily - typically domain-specific. Namespaces can be thought of modules in a programming language or molecules comprising of atoms.


**Composition**

tc provides a simple mechanism to define and connect these namespaced entities as a graph and thus the use of word topology. As a result of entity composition, tc can infer the infrastrucuture permissions etc and render it in arbitrary sandboxes thus enabling sophisticated workflows.


Example Topology definition:

```yaml
name: etl

routes:
  /api/etl:
    method: POST
    function: enhancer

functions:
  enhancer:
    function: transformer
  transformer:
    function: loader
  loader:
    event: Notify

events:
  Notify:
    channel: Subscription

channels:
  Subscription:
    function: default

```

`/api/etl` HTTP route calls function `enhancer` which then triggers a pipeline of functions which are either local (subdirectories) or remote (git repos). In this example, loader finally generates an event `Notify` whose target is a websocket Channel called `Subscription`. We just defined an entire ETL flow without specifying anything about infrastructure, permissions or the provider. None of the infrastructure stuff has leaked into this definition that describes the high-level flow. This definition is good enough to render it in the cloud as services, as architecture diagrams and release manifests.

`tc compose` maps these entities to the provider's serverless constructs. If the provider is AWS (default), tc maps `routes` to API Gateway, events to `Eventbridge`, `functions` to either `Lambda` or `ECS Fargate`, `channels` to `Appsync Events`, `mutations` to `Appsync Graphql` and `queues` to `SQS`

## Install

See [installation guide](https://tc-functors.org/docs/installation.html)

## Resources

Documentation: https://tc-functors.org/

Video Presentation on tc from AWS Community Day - Bay Area Sept 2024
[Higher Order Abstraction & Tooling for Step Functions & Serverless](https://youtu.be/1gqDGulszzQ?si=dtHcUkQF2nhZ_td8)


## Basic Usage


```sh
Usage: tc <COMMAND>

Commands:
  build    Build layers, extensions and pack function code
  compose  Compose a Topology
  create   Create a sandboxed topology
  delete   Delete a sandboxed topology
  emulate  Emulate Runtime environments
  inspect  Inspect via browser
  invoke   Invoke a topology synchronously or asynchronously
  list     List created entities
  resolve  Resolve a topology from functions, events, states description
  update   Update components
  upgrade  upgrade tc version
  version  display current tc version
```
## Contributing

We welcome contributions from the community! Whether you're just giving feedback, fixing bugs, improving documentation, or proposing new features, your efforts are appreciated.

### Code of Conduct

This project follows the Contributor Covenant Code of Conduct. We expect all contributors to adhere to its guidelines to maintain a welcoming and inclusive environment. Please read our [Code of Conduct](code_of_conduct.md) before participating.

### Ways to Contribute

- **Report Issues**: Found a bug or have a suggestion? Open an issue on our [tc GitHub Issues](https://github.com/tc-functors/tc/issues) page
- **Submit Pull Requests**: Have a fix or enhancement? PRs are welcome! [tc Github PRs](https://github.com/tc-functors/tc/pulls)
- **Improve Documentation**: Help make our docs better by fixing errors or adding examples
- **Join Discussions**: Participate in [GitHub Discussions](https://github.com/orgs/tc-functors/discussions) to share ideas, ask questions, and help others

### Project Structure

The codebase is organized as a Rust workspace with multiple libraries:

- **Main CLI (`src/`)**: Command-line interface for TC
- **Libraries (`lib/`)**:
  - `authorizer`: Authentication and authorization
  - `builder`: Building and packaging functions
  - `composer`: Composing topologies
  - `deployer`: Deploying to cloud providers
  - `differ`: Comparing topologies
  - `emulator`: Local emulation
  - `inspector`: Visualization and inspection
  - `invoker`: Invoking functions
  - `releaser`: Release management
  - `resolver`: Resolving template variables

### Development Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write or update tests as needed
5. Submit a pull request
6. Respond to any feedback

We aim to review all contributions promptly and look forward to collaborating with you!

## Core Authors

- Isaac Praveen
- Rob Berger (Mentor)

## Thanks & Credits

Thanks to the following engineers for contributing ideas and testing.

- [Eric Harvey](https://github.com/EricHarvey)
- [Rachel Chung](https://github.com/rachel-yujin-chung)
- [Rahul Salla](https://github.com/raaahulss)
- [Alper Vural](https://github.com/alperinformed)
- [Alexander Ngyuen](https://github.com/GalexyN)
- [Sanjeev](https://github.com/sanjeev247)
- [Abhijith Gopal](https://github.com/abhijith)

Thanks to Rich Hickey (Clojure) and Joe Armstrong (Erlang) for influencing the way we think about programs and complexity.
