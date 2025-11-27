# Product Overview

tc (Topology Composer) is a graph-based, executable architecture description language and framework for cloud-native serverless systems with fractal composition and infrastructure generation capabilities.

## Core Concept

The central concept is the "Cloud Functor" - a namespaced, sandboxed, versioned, and isomorphic topology of serverless components. The term "functor" is borrowed from OCaml's parameterized modules, emphasizing first-class, composable units.

## Key Features

- **Entity Abstraction**: Provides 7 core entities (functions, events, mutations, queues, routes, states, channels) that abstract cloud primitives in a cloud-agnostic way
- **Namespacing**: Entities can be namespaced arbitrarily (typically domain-specific) like modules in a programming language
- **Composition**: Define and connect namespaced entities as a graph, allowing tc to infer infrastructure permissions and render in arbitrary sandboxes
- **Provider Mapping**: Maps entities to provider-specific constructs (e.g., AWS: routes → API Gateway, events → EventBridge, functions → Lambda/ECS Fargate, channels → AppSync Events, mutations → AppSync GraphQL, queues → SQS)

## Value Proposition

Enables developers to focus on business logic and component relationships rather than infrastructure management, while maintaining the ability to deploy consistently across environments and providers.
