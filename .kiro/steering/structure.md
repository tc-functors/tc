# Project Structure

## Repository Layout

```
tc/
├── src/                    # Main CLI application
│   ├── main.rs            # Entry point
│   ├── lib.rs             # Library interface
│   ├── interactive.rs     # Interactive mode
│   └── remote.rs          # Remote operations
├── lib/                   # Workspace libraries (19 crates)
├── examples/              # Example topologies and functions
├── etc/                   # Utilities (e.g., transducer.py)
├── doc/                   # Documentation
├── Cargo.toml             # Workspace manifest
├── Makefile               # Build automation
└── .kiro/                 # Kiro AI assistant configuration
    └── steering/          # AI guidance documents
```

## Library Organization

The codebase follows a modular architecture with 19 specialized libraries in `lib/`:

### Core Libraries
- **kit**: Shared utilities and common functionality
- **compiler**: Compiles topology definitions
- **resolver**: Resolves template variables and references
- **composer**: Composes topologies from entity definitions

### Build & Deploy
- **builder**: Builds and packages function code, layers, extensions
- **deployer**: Deploys to cloud providers
- **configurator**: Configuration management
- **scaffolder**: Generates project scaffolding

### Runtime & Execution
- **emulator**: Local emulation of runtime environments
- **executor**: Executes functions and workflows
- **invoker**: Invokes topology functions (sync/async)
- **provider**: Cloud provider abstractions

### Testing & Quality
- **tester**: Testing framework
- **snapshotter**: Snapshot testing
- **differ**: Compares topologies and detects changes

### Operations
- **router**: Routing logic
- **notifier**: Notification system
- **tagger**: Version tagging
- **visualizer**: Visualization and inspection

## Examples Directory

Organized by composition patterns and language runtimes:

### Composition Patterns (`examples/composition/`)
- Event-driven patterns (event-channel, event-function, event-state)
- Function composition (function-function, function-mutation)
- Routing patterns (route-event, route-function, route-queue, route-state)
- Queue-based patterns (queue-state)

### Function Examples (`examples/functions/`)
Language-specific implementations:
- **Python**: basic, inline, image, layer, snap variants
- **Node.js**: basic, inline variants
- **Ruby**: basic, inline variants
- **Rust**: inline variant
- **Clojure**: inline variant
- **Janet**: inline variant
- **Mixed**: Multi-language topology example

Each function example typically includes:
- `handler.*` - Function implementation
- `function.yml` - Function configuration
- Language-specific dependency files (pyproject.toml, package.json, Gemfile, etc.)

## Topology Definition Files

- **topology.yml**: Defines entity relationships and composition
- **function.yml**: Individual function configuration
- **config.toml**: Configuration settings

## Conventions

- Library crates are path dependencies in the workspace
- Each library is self-contained with its own Cargo.toml
- Main CLI depends on all libraries
- Examples demonstrate both simple and complex composition patterns
- Cross-platform support via conditional compilation and feature flags
