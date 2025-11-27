# Examples Directory Guide

The `examples/` directory contains reference implementations demonstrating tc patterns and capabilities.

## Directory Structure

### examples/apps/
Complete application examples:
- **chat**: Real-time chat using channels (WebSocket)
- **notes**: Full-stack app with GraphQL mutations, subscriptions, and SPA

### examples/composition/
Entity composition patterns showing how different entities connect:
- **event-channel**: Event triggering channel
- **event-function**: Event triggering function
- **event-mutation**: Event triggering GraphQL mutation
- **event-state**: Event triggering Step Functions workflow
- **function-event**: Function triggering event
- **function-function**: Function chaining
- **function-mutation**: Function triggering mutation
- **mutation-function**: Mutation resolver calling function
- **queue-state**: Queue triggering state machine
- **route-***: Various route compositions

### examples/functions/
Function runtime examples:
- **clojure-inline**, **janet-inline**: Alternative language runtimes
- **node-basic**, **node-inline**: Node.js functions
- **python-basic**, **python-inline**, **python-image**, **python-layer**, **python-snap**: Python variants
- **ruby-basic**, **ruby-inline**: Ruby functions
- **rust-inline**: Rust functions
- **mixed**: Multiple languages in one topology
- **infra-basic**: Custom infrastructure configuration

### examples/patterns/
Common application patterns:
- **chat**: WebSocket-based real-time chat
- **evented**: Event-driven architecture
- **gql-progress**, **gql-proxy**: GraphQL patterns
- **htmx**: HTMX server-side rendering
- **http-upload**: File upload handling
- **rest-async**, **rest-async-progress**: Async REST APIs
- **rest-auth**: Authentication patterns

### examples/states/
Step Functions workflow patterns:
- **basic**: Simple state machine
- **continuation**: Long-running workflows
- **map-async**, **map-csv**, **map-dist**: Map state patterns
- **mapreduce**: MapReduce pattern
- **parallel**: Parallel execution
- **routing**: Conditional routing

### examples/pages/
Static site and SPA examples:
- **static**: Basic static site
- **spa-mithril**, **spa-svelte**: Single-page applications
- **pwa**: Progressive web app

### examples/orchestrator/
Complex orchestration examples with multiple functions coordinating

### examples/tables/
Database/table schema examples

### examples/tests/
Testing patterns and examples

## Using Examples

1. **Browse by use case**: Find the pattern closest to your needs
2. **Study the topology.yml**: Understand entity relationships
3. **Check function handlers**: See implementation patterns
4. **Adapt to your needs**: Copy and modify for your application

## Key Files in Examples

- **topology.yml**: Main topology specification
- **function.yml**: Function-specific configuration (optional)
- **handler.{py,js,rb,rs}**: Function implementation
- **index.html**: Frontend for full-stack examples
- **package.json**, **pyproject.toml**, **Gemfile**, **Cargo.toml**: Language-specific dependencies
