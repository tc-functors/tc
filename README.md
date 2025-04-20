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

We've been working on tc for quite a while in a private repo, but much of that time was focused on the internal needs of Informed. Since the creation of this public repo we have started work on making it suitable for broader use cases beyond our own.

Here is a snapshot of the history and future plans. Please let us know how we could make it useful for your use cases as well.

![Roadmap](doc/images/tc-roadmap.png)

## Contributing

Though significant work has been done previous to this public repo for internal use at Informed, this project is still quite nascent and is being actively developed to be suitable for use outside of Informed.

We welcome contributions from the community! Whether you're just giving feedback, fixing bugs, improving documentation, or proposing new features, your efforts are appreciated.

### Code of Conduct

This project follows the Contributor Covenant Code of Conduct. We expect all contributors to adhere to its guidelines to maintain a welcoming and inclusive environment. Please read our [Code of Conduct](code_of_conduct.md) before participating.

### Getting Started

1. Try out tc by following the [installation guide](https://informed-labs.github.io/tc/installation.html)
2. Build from source:
   - Follow the [build instructions](https://informed-labs.github.io/tc/installation.html#building-your-own) to compile tc locally
   - This is a great way to understand the codebase and start contributing

### Ways to Contribute

- **Report Issues**: Found a bug or have a suggestion? Open an issue on our [tc GitHub Issues](https://github.com/informed-labs/tc/issues) page
- **Submit Pull Requests**: Have a fix or enhancement? PRs are welcome! [tc Github PRs](https://github.com/informed-labs/tc/pulls)
- **Improve Documentation**: Help make our docs better by fixing errors or adding examples
- **Join Discussions**: Participate in [GitHub Discussions](https://github.com/orgs/informed-labs/discussions) to share ideas, ask questions, and help others

### Development Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write or update tests as needed
5. Submit a pull request
6. Respond to any feedback

We aim to review all contributions promptly and look forward to collaborating with you!

