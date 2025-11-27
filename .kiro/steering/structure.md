# Project Structure

## Workspace Organization

tc is organized as a Cargo workspace with a main CLI binary and multiple library crates.

## Top-Level Structure

```
tc/
├── src/                    # Main CLI application
│   ├── main.rs            # CLI entry point with command definitions
│   ├── lib.rs             # Core library functions
│   ├── interactive.rs     # Interactive mode functionality
│   └── remote.rs          # Remote operations (CI/CD)
├── lib/                   # Library crates (workspace members)
├── examples/              # Example topologies and patterns
├── target/                # Build artifacts (gitignored)
├── Cargo.toml            # Workspace configuration
├── Makefile              # Build automation
└── rustfmt.toml          # Code formatting rules
```

## Library Crates (lib/)

Each subdirectory in `lib/` is a separate crate with its own Cargo.toml:

- **builder**: Building and packaging functions
- **compiler**: Compiling topology specifications (YAML/Lisp)
- **composer**: Composing topologies into cloud resources
- **configurator**: Configuration management
- **deployer**: Deploying to cloud providers
- **differ**: Comparing topologies and detecting changes
- **emulator**: Local emulation of runtime environments
- **executor**: CI/CD executor integrations (CircleCI, Drone, GitHub, Rebar)
- **invoker**: Invoking functions, events, routes, and states
- **kit**: Common utilities (core, crypto, git, github, http, io, json, pprint, prompt, text, time)
- **notifier**: Notification system
- **provider**: Cloud provider implementations (aws, fly, gcp, local)
- **resolver**: Resolving template variables and dependencies
- **router**: Routing logic
- **scaffolder**: Scaffolding functions and topologies using LLM
- **snapshotter**: Creating snapshots of deployments
- **tagger**: Git tagging and changelog generation
- **tester**: Testing framework
- **visualizer**: Visualization and inspection (digraph generation)

## Examples Structure

```
examples/
├── apps/              # Full application examples (chat, notes)
├── composition/       # Entity composition patterns
├── functions/         # Function examples (various languages/runtimes)
├── orchestrator/      # Orchestration patterns
├── pages/             # Static pages and SPAs
├── patterns/          # Common patterns (auth, async, htmx, etc.)
├── states/            # State machine examples
├── tables/            # Table/database examples
└── tests/             # Test examples
```

## Key Files

- **topology.yml**: Primary specification file for defining serverless topologies
- **topology.lisp**: Alternative Lisp-based topology specification
- **function.yml**: Function-specific configuration
- **config.toml**: Configuration files

## Naming Conventions

- Library crates use lowercase with hyphens (e.g., `lib/compiler`)
- Source files use snake_case (e.g., `main.rs`, `interactive.rs`)
- Modules are organized by domain/feature within each crate
- Provider-specific code is namespaced under provider directories (e.g., `lib/composer/src/aws/`)

## Module Organization Pattern

Within library crates, common patterns include:
- `lib.rs`: Public API and re-exports
- `mod.rs`: Module definitions
- Feature-specific files (e.g., `function.rs`, `event.rs`, `state.rs`)
- Provider subdirectories for cloud-specific implementations
