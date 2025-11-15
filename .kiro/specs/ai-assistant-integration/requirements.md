# Requirements Document

## Introduction

This specification defines the requirements for making Topology Composer (TC) accessible and usable by AI coding assistants (LLMs and AI IDEs such as Kiro, Cursor, Copilot, Claude Code, etc.). The goal is to enable AI assistants to understand TC's graph-based architecture philosophy, use TC commands effectively, and design serverless systems following TC's composition patterns and best practices.

## Glossary

- **TC (Topology Composer)**: A graph-based, executable architecture description language and framework for cloud-native serverless systems
- **Cloud Functor**: A namespaced, sandboxed, versioned, and isomorphic topology of serverless components in TC
- **Entity**: Core cloud primitives in TC (functions, events, mutations, queues, routes, states, channels)
- **Topology**: A graph-based composition of TC entities that defines system architecture
- **AI Assistant**: LLM-powered coding tools including Kiro, Cursor, Copilot, Claude Code, and similar tools
- **Steering File**: A markdown file in Kiro that provides automatic context to AI assistants
- **MCP (Model Context Protocol)**: A protocol for exposing tools and context to AI assistants across multiple platforms
- **MCP Server**: A service that implements MCP to provide tools and resources to AI assistants
- **Composition Pattern**: A reusable way of connecting TC entities to achieve specific architectural outcomes

## Requirements

### Requirement 1

**User Story:** As a developer using an AI assistant, I want the assistant to understand TC's core philosophy and design principles, so that it can help me design systems that align with TC's graph-based, entity-composition approach.

#### Acceptance Criteria

1. WHEN a developer asks an AI assistant about TC architecture, THE AI Assistant SHALL explain the Cloud Functor concept and entity-based composition model
2. WHEN a developer requests system design help, THE AI Assistant SHALL recommend solutions using TC's seven core entities (functions, events, mutations, queues, routes, states, channels)
3. WHEN a developer describes a use case, THE AI Assistant SHALL identify appropriate TC composition patterns that match the requirements
4. THE AI Assistant SHALL distinguish between business logic concerns and infrastructure concerns when designing TC topologies
5. THE AI Assistant SHALL recommend namespace organization strategies based on domain-specific boundaries

### Requirement 2

**User Story:** As a developer using an AI assistant, I want the assistant to help me write correct topology.yml files, so that I can define my serverless architecture without syntax errors or structural mistakes.

#### Acceptance Criteria

1. WHEN a developer requests a topology definition, THE AI Assistant SHALL generate valid YAML syntax conforming to TC's topology schema
2. WHEN a developer describes entity relationships, THE AI Assistant SHALL create correct entity-to-entity connections in the topology
3. THE AI Assistant SHALL validate that referenced entities (functions, events, etc.) are properly defined before being referenced
4. WHEN a developer adds a new entity, THE AI Assistant SHALL ensure all required fields for that entity type are included
5. THE AI Assistant SHALL detect and warn about circular dependencies or invalid composition patterns

### Requirement 3

**User Story:** As a developer using an AI assistant, I want the assistant to know how to use TC CLI commands effectively, so that it can help me build, compose, deploy, and manage my topologies.

#### Acceptance Criteria

1. WHEN a developer asks how to perform a TC operation, THE AI Assistant SHALL provide the correct tc command with appropriate flags and arguments
2. THE AI Assistant SHALL recommend the correct sequence of tc commands for common workflows (build, compose, create, deploy)
3. WHEN a developer encounters a TC error, THE AI Assistant SHALL interpret the error and suggest corrective actions
4. THE AI Assistant SHALL know when to use tc emulate for local testing versus tc create for cloud deployment
5. THE AI Assistant SHALL understand the difference between tc compose (infrastructure generation) and tc create (deployment)

### Requirement 4

**User Story:** As a developer using an AI assistant, I want the assistant to access TC topology information and command capabilities interactively, so that it can query my current system state and execute TC operations on my behalf.

#### Acceptance Criteria

1. WHEN an AI assistant needs topology information, THE MCP Server SHALL provide tools to query topology structure and entity definitions
2. WHEN an AI assistant needs to validate a topology, THE MCP Server SHALL provide tools to check topology correctness without deployment
3. WHEN an AI assistant needs to inspect deployed resources, THE MCP Server SHALL provide tools to list and describe created topologies
4. THE MCP Server SHALL expose tc build, tc compose, and tc resolve commands as callable tools
5. THE MCP Server SHALL return structured data from TC commands that AI assistants can parse and interpret

### Requirement 5

**User Story:** As a developer using Kiro, I want TC design patterns and best practices automatically available in my AI context, so that Kiro can help me build TC systems without me having to repeatedly explain TC concepts.

#### Acceptance Criteria

1. WHEN a developer opens a TC project in Kiro, THE Steering Files SHALL automatically provide TC philosophy and entity definitions to the AI context
2. THE Steering Files SHALL include examples of common composition patterns with explanations
3. THE Steering Files SHALL document anti-patterns and common mistakes to avoid
4. WHEN a developer works on topology files, THE Steering Files SHALL provide entity-specific guidance based on file context
5. THE Steering Files SHALL include decision trees for choosing between different TC entities and patterns

### Requirement 6

**User Story:** As a developer using an AI assistant, I want access to annotated examples of TC patterns, so that the assistant can learn from real implementations and suggest similar solutions for my use cases.

#### Acceptance Criteria

1. THE Example Repository SHALL contain at least 10 common serverless patterns implemented in TC with detailed annotations
2. WHEN an AI assistant encounters a use case, THE AI Assistant SHALL reference relevant annotated examples to guide implementation
3. THE Annotated Examples SHALL explain why specific entities were chosen and how they compose together
4. THE Annotated Examples SHALL include comments describing the data flow through the topology
5. THE Annotated Examples SHALL document deployment considerations and testing strategies

### Requirement 7

**User Story:** As a developer using an AI assistant, I want the assistant to help me test and debug TC topologies, so that I can identify and fix issues before deployment.

#### Acceptance Criteria

1. WHEN a developer requests testing help, THE AI Assistant SHALL recommend appropriate tc emulate commands for local testing
2. THE AI Assistant SHALL suggest test payloads and invocation patterns for different entity types
3. WHEN a topology fails to compose, THE AI Assistant SHALL analyze the error and suggest specific fixes
4. THE AI Assistant SHALL recommend debugging strategies using tc inspect and tc invoke commands
5. THE AI Assistant SHALL help developers write test cases that validate topology behavior

### Requirement 8

**User Story:** As a developer using an AI assistant, I want the assistant to understand TC's multi-environment deployment model, so that it can help me manage sandboxed topologies across development, staging, and production environments.

#### Acceptance Criteria

1. WHEN a developer asks about environments, THE AI Assistant SHALL explain TC's sandbox concept and namespace isolation
2. THE AI Assistant SHALL recommend environment-specific configuration strategies using TC's templating capabilities
3. WHEN a developer deploys to multiple environments, THE AI Assistant SHALL suggest appropriate naming and tagging conventions
4. THE AI Assistant SHALL understand how to use tc list and tc delete to manage sandboxed topologies
5. THE AI Assistant SHALL recommend strategies for promoting topologies between environments safely

### Requirement 9

**User Story:** As a TC maintainer, I want AI assistants to stay current with TC updates, so that they provide accurate guidance as TC evolves.

#### Acceptance Criteria

1. THE Documentation System SHALL include version information that AI assistants can reference
2. WHEN TC introduces new features, THE Documentation System SHALL clearly mark new capabilities with version numbers
3. THE Documentation System SHALL maintain a changelog that AI assistants can parse to understand feature evolution
4. THE Steering Files SHALL include the current TC version they are written for
5. THE MCP Server SHALL report its supported TC version when queried

### Requirement 10

**User Story:** As a developer using an AI assistant across different tools (Kiro, Cursor, Claude, etc.), I want consistent TC guidance regardless of which AI tool I use, so that I can switch tools without losing TC expertise.

#### Acceptance Criteria

1. THE Documentation SHALL be structured in a tool-agnostic format that any AI assistant can consume
2. THE MCP Server SHALL work with any MCP-compatible AI assistant without tool-specific customization
3. THE Example Repository SHALL use standard markdown and YAML formats accessible to all AI tools
4. THE Steering Files SHALL have equivalent documentation available as standalone markdown for non-Kiro tools
5. THE AI Integration System SHALL provide a single source of truth that all AI tools can reference
