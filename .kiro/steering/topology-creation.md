---
inclusion: manual
---

# Creating tc Topologies

This guide helps create tc topology configurations for applicaitons. Use this when building new serverless applications with tc.

## Core Principles

- **Provider-agnostic**: Definitions work across cloud providers (AWS, GCP, Fly, local)
- **Composable**: Entities chain together naturally through graph relationships
- **Namespaced**: All entities are isolated within their topology
- **Business-focused**: Abstract away infrastructure complexity

## Available Entities

### Routes
HTTP endpoints that trigger functions or events:
```yaml
routes:
  /api/endpoint:
    method: POST|GET|PUT|DELETE|PATCH
    function: function-name    # Synchronous
    event: event-name         # Asynchronous
    cors:                     # Optional CORS config
      methods: ['POST', 'OPTIONS']
      origins: ['*']
      allowed_headers: ['Content-Type']
```

### Functions
Serverless compute units that can chain to other entities:
```yaml
functions:
  function-name:
    function: next-function   # Chain to another function
    event: event-name        # Trigger an event
    queue: queue-name        # Send to queue
    channel: channel-name    # Send to WebSocket
    mutation: mutation-name  # GraphQL mutation
    state: state-name        # Step Functions workflow
```

### Events
Asynchronous event notifications (maps to EventBridge on AWS):
```yaml
events:
  EventName:
    function: handler-function
    queue: queue-name
    channel: channel-name
    state: state-name
```

### Mutations
GraphQL API definitions (maps to AppSync on AWS):
```yaml
mutations:
  authorizer: authorizer-function  # Optional
  types:
    TypeName:
      field: String!
  resolvers:
    resolverName:
      input: InputType
      output: OutputType
      function: handler-function  # Must use Lambda (VTL not yet supported)
      subscribe: true  # Enable subscriptions
```

**Note**: tc currently requires Lambda functions for mutation resolvers. Direct VTL resolvers or other AWS resources (like DynamoDB direct access) are not yet supported. Use Lambda functions to interact with DynamoDB and other services.

### Queues
Message queues for async processing (maps to SQS on AWS):
```yaml
queues:
  queue-name:
    function: processor-function
    batch_size: 10
```

### Channels
WebSocket connections for real-time communication (maps to AppSync Events on AWS):
```yaml
channels:
  channel-name:
    type: websocket
    handler: default  # or specific function name
```

### States
Step Functions workflows for orchestration:
```yaml
states:
  workflow-name:
    definition: workflow.json  # ASL definition file
```

### Tables
DynamoDB tables for data persistence:
```yaml
tables:
  table-name:
    hash_key: "PrimaryKey"      # Partition key
    range_key: "SortKey"        # Optional sort key
```

### Pages
Static sites or SPAs:
```yaml
pages:
  app:
    dir: app              # Source directory
    dist: dist            # Build output directory
    config_template: src/config.js  # Optional config template
    skip_deploy: true     # Skip deployment (local dev only)
```

## Design Patterns

### Synchronous Request-Response
```yaml
routes:
  /api/data:
    method: GET
    function: fetch-data
```

### Async Fire-and-Forget
```yaml
routes:
  /api/process:
    method: POST
    event: ProcessStart

events:
  ProcessStart:
    function: processor
```

### Function Pipeline
```yaml
functions:
  validate:
    function: transform
  transform:
    function: save
  save:
    event: SaveComplete
```

### Real-Time Updates
```yaml
functions:
  processor:
    channel: live-updates

channels:
  live-updates:
    handler: default
```

### GraphQL with Subscriptions
```yaml
mutations:
  types:
    Note:
      id: String!
      text: String
  resolvers:
    addNote:
      input: Note
      output: Note
      function: processor  # Lambda function required
      subscribe: true  # Enables real-time subscriptions
```

### DynamoDB Tables
```yaml
tables:
  users:
    hash_key: "userId"
    range_key: "timestamp"  # Optional

functions:
  save-user:
    # Access DynamoDB through Lambda function
    # Table name available via environment variables
```

### Background Processing
```yaml
functions:
  api-handler:
    queue: processing-queue

queues:
  processing-queue:
    function: worker
    batch_size: 10
```

## Naming Conventions

- **Topology name**: kebab-case (e.g., `user-management`, `order-processing`)
- **Functions**: action-based, kebab-case (e.g., `validate-input`, `process-payment`)
- **Events**: PascalCase, past tense (e.g., `OrderCreated`, `PaymentProcessed`)
- **Queues**: purpose-based, kebab-case (e.g., `processing-queue`, `email-queue`)
- **Channels**: purpose-based, kebab-case (e.g., `live-updates`, `notifications`)
- **Routes**: REST-style paths (e.g., `/api/users`, `/api/orders/:id`)

## Topology File Structure

Standard order for clarity:
```yaml
name: topology-name

routes:
  # HTTP endpoints

functions:
  # Business logic

events:
  # Async events

mutations:
  # GraphQL API

queues:
  # Background processing

channels:
  # WebSocket connections

states:
  # Step Functions workflows

tables:
  # DynamoDB tables

pages:
  # Static sites/SPAs
```

## Common Compositions

**Route → Function → Event → Channel** (async with real-time updates)
**Route → Event → Function → Function** (async pipeline)
**Route → Function → Mutation** (GraphQL API)
**Route → Function → Queue → Function** (background processing)
**Function → State** (complex orchestration)

## Validation Checklist

- [ ] All referenced entities are defined
- [ ] Routes have valid HTTP methods
- [ ] Function chains are logical and non-circular
- [ ] CORS is configured for browser-based clients
- [ ] Mutations have proper type definitions
- [ ] Channel handlers are defined

## Current Limitations

- **Mutation resolvers**: Must use Lambda functions. VTL resolvers and direct AWS resource access (e.g., DynamoDB VTL resolvers) are not yet supported.
- **DynamoDB access**: Use Lambda functions to interact with DynamoDB tables, even for simple CRUD operations.

## Tips

- Full documentation of tc is at https://tc-functors.org and https://deepwiki.com/tc-functors/tc
- Specification for topology.yml is at https://tc-functors.org/reference/specification/
- Use `event` from routes for fire-and-forget operations
- Use `function` from routes for synchronous responses
- Chain functions for sequential processing
- Use channels for real-time browser updates
- Use mutations for GraphQL APIs with subscriptions
- Define `tables` for DynamoDB and access them via Lambda functions
- Keep function logic focused and composable
- Leverage tc's inference for permissions and infrastructure
- tc automatically grants Lambda functions permissions to access defined tables
