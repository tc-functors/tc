# Product Overview

tc (Topology Composer) is a graph-based, executable architecture description language and framework for cloud-native serverless systems with fractal composition and infrastructure generation capabilities.

## Core Concept

The central concept is the "Cloud Functor" - a namespaced, sandboxed, versioned, and isomorphic topology of serverless components. The term "functor" is borrowed from OCaml's parameterized modules, emphasizing first-class, composable units.

## Value Proposition

tc enables developers to focus on business logic and component relationships rather than infrastructure management, while maintaining the ability to deploy consistently across environments and providers.

## Key Capabilities

- **Entity Abstraction**: 7 cloud-agnostic entities (functions, events, mutations, queues, routes, states, channels) that abstract low-level cloud primitives
- **Namespacing**: Domain-specific organization of entities (like modules in programming languages)
- **Composition**: Define and connect namespaced entities as a graph to infer infrastructure permissions and render in arbitrary sandboxes
- **Provider Mapping**: Automatically maps entities to provider-specific constructs (AWS by default: API Gateway, EventBridge, Lambda/ECS Fargate, AppSync, SQS)

## Target Users

Developers building cloud-native serverless systems who want to define high-level architecture without dealing with infrastructure details.
