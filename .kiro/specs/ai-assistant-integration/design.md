# Design Document: AI Assistant Integration for Topology Composer

## Overview

This design document outlines a multi-layered approach to making Topology Composer (TC) accessible and usable by AI coding assistants. The solution consists of four primary components that work together to provide comprehensive TC knowledge and capabilities to AI assistants across different platforms:

1. **Steering Files** - Automatic context for Kiro users
2. **MCP Server** - Interactive TC operations for MCP-compatible tools
3. **Annotated Examples** - Pattern library with inline documentation
4. **Structured Documentation** - AI-parseable reference materials

The design prioritizes ease of maintenance, automatic synchronization with TC changes, and cross-platform compatibility.

## Architecture

### High-Level Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     AI Assistants                            │
│  (Kiro, Cursor, Copilot, Claude Code, etc.)                │
└────────┬──────────────────────────────┬────────────────────┘
         │                               │
         │ Auto-loaded                   │ MCP Protocol
         │ (Kiro only)                   │ (All MCP tools)
         │                               │
┌────────▼─────────────┐        ┌───────▼──────────────────┐
│  Steering Files      │        │   TC MCP Server          │
│  (.kiro/steering/)   │        │   (Python/Node)          │
│                      │        │                          │
│  - Core concepts     │        │  Tools:                  │
│  - Entity guide      │        │  - tc_compose            │
│  - Pattern catalog   │        │  - tc_validate           │
│  - Decision trees    │        │  - tc_query_topology     │
│  - Anti-patterns     │        │  - tc_list_examples      │
└──────────────────────┘        │  - tc_get_pattern        │
                                └──────────┬───────────────┘
                                           │
                                           │ Executes
                                           │
         ┌─────────────────────────────────▼───────────────┐
         │         TC CLI & Codebase                       │
         │                                                  │
         │  - tc binary (Rust)                             │
         │  - examples/ (annotated patterns)               │
         │  - docs/ (structured guides)                    │
         └──────────────────────────────────────────────────┘
```

### Data Flow

1. **Context Loading**: When an AI assistant starts working in a TC project
   - Kiro: Automatically loads steering files
   - Other tools: User provides context via documentation or MCP queries

2. **Query & Execution**: When an AI assistant needs TC information or operations
   - MCP tools query topology structure, examples, or execute commands
   - Results returned as structured JSON for AI parsing

3. **Learning & Adaptation**: When TC codebase changes
   - Examples updated with inline annotations
   - Documentation regenerated from source
   - MCP server reflects latest capabilities

## Components and Interfaces

### 1. Steering Files System

**Purpose**: Provide automatic TC context to Kiro users without manual setup.

**Location**: `.kiro/steering/`

**Files Structure**:
```
.kiro/steering/
├── tc-core-concepts.md          # Always included
├── tc-entity-reference.md       # Always included
├── tc-composition-patterns.md   # Always included
├── tc-topology-guide.md         # Conditional: fileMatch "topology.yml"
├── tc-function-guide.md         # Conditional: fileMatch "*/functions/*"
└── tc-testing-guide.md          # Manual: #tc-testing
```

**Content Organization**:

Each steering file follows this structure:
```markdown
---
inclusion: always | fileMatch | manual
fileMatchPattern: "pattern" (if fileMatch)
---

# [Topic]

## Quick Reference
[Concise summary for AI quick lookup]

## Concepts
[Detailed explanations]

## Examples
[Inline code examples]

## Decision Trees
[When to use what]

## Common Mistakes
[Anti-patterns to avoid]
```

**Key Design Decisions**:
- Use frontmatter for conditional inclusion rules
- Keep each file focused on a single concern
- Include inline examples rather than references
- Provide decision trees for entity selection
- Document anti-patterns explicitly

**Interface**:
- No programmatic interface - files are read by Kiro automatically
- Must be valid markdown with optional YAML frontmatter
- Support `#[[file:path]]` syntax for including other files (e.g., example topologies)

### 2. TC MCP Server

**Purpose**: Expose TC operations and queries as tools that AI assistants can invoke interactively.

**Technology**: Python (using FastMCP framework for rapid development)

**Location**: `mcp-server/` (new directory in TC repo)

**Deployment Modes**:
1. **Local Mode**: Docker container for development and local use
2. **Serverless Mode**: AWS Lambda with AgentCore integration for production use

**Architecture Decision**: Use AgentCore and AgentCore Gateway for serverless deployment to leverage:
- Managed MCP protocol handling
- Scalable serverless execution
- Built-in authentication and authorization
- Multi-tenant support
- Cost-effective pay-per-use model

**Tools Exposed**:

```python
# Tool definitions (FastMCP syntax)

@mcp.tool()
def tc_compose(topology_path: str, sandbox: str = "dev", aws_region: str = None, role_arn: str = None) -> dict:
    """
    Compose a TC topology and return the generated infrastructure.
    
    Args:
        topology_path: Path to topology.yml file
        sandbox: Sandbox name for composition
        aws_region: AWS region (optional, uses default if not specified)
        role_arn: AWS role ARN to assume for cross-account access (optional)
    
    Returns:
        {
            "success": bool,
            "infrastructure": dict,  # Parsed infrastructure output
            "errors": list,
            "aws_account": str,  # AWS account ID used
            "aws_region": str    # AWS region used
        }
    """
    
@mcp.tool()
def tc_validate(topology_path: str) -> dict:
    """
    Validate a topology file without deploying.
    
    Returns:
        {
            "valid": bool,
            "errors": list,
            "warnings": list,
            "entities": {
                "functions": list,
                "events": list,
                ...
            }
        }
    """

@mcp.tool()
def tc_query_topology(topology_path: str) -> dict:
    """
    Parse and return structured topology information.
    
    Returns:
        {
            "name": str,
            "entities": {
                "routes": dict,
                "functions": dict,
                "events": dict,
                ...
            },
            "composition_graph": list  # Edge list
        }
    """

@mcp.tool()
def tc_list_examples(pattern_type: str = None) -> list:
    """
    List available TC example patterns.
    
    Args:
        pattern_type: Filter by type (composition, pattern, state, etc.)
    
    Returns:
        [
            {
                "name": str,
                "path": str,
                "description": str,
                "entities_used": list,
                "use_cases": list
            }
        ]
    """

@mcp.tool()
def tc_get_pattern(pattern_name: str) -> dict:
    """
    Get detailed information about a specific pattern.
    
    Returns:
        {
            "topology": str,  # YAML content
            "annotations": dict,  # Parsed annotations
            "description": str,
            "data_flow": list,
            "deployment_notes": str
        }
    """

@mcp.tool()
def tc_list_sandboxes() -> list:
    """
    List deployed TC sandboxes.
    
    Returns:
        [
            {
                "name": str,
                "topology": str,
                "created": str,
                "status": str
            }
        ]
    """

@mcp.tool()
def tc_get_recent_changes(since_days: int = 7) -> dict:
    """
    Get recent changes to examples and documentation.
    
    Returns:
        {
            "examples_added": list,
            "examples_modified": list,
            "docs_updated": list,
            "new_features": list
        }
    """

@mcp.tool()
def tc_create(topology_path: str, sandbox: str, aws_region: str = None, role_arn: str = None) -> dict:
    """
    Deploy a TC topology to AWS.
    
    Args:
        topology_path: Path to topology.yml file
        sandbox: Sandbox name for deployment
        aws_region: AWS region (optional, uses default if not specified)
        role_arn: AWS role ARN to assume for cross-account access (optional)
    
    Returns:
        {
            "success": bool,
            "deployment_id": str,
            "resources_created": list,
            "endpoints": dict,  # API endpoints, WebSocket URLs, etc.
            "errors": list,
            "aws_account": str,
            "aws_region": str
        }
    """

@mcp.tool()
def tc_delete(topology_name: str, sandbox: str, aws_region: str = None, role_arn: str = None) -> dict:
    """
    Delete a deployed TC topology from AWS.
    
    Args:
        topology_name: Name of the topology to delete
        sandbox: Sandbox name
        aws_region: AWS region (optional, uses default if not specified)
        role_arn: AWS role ARN to assume for cross-account access (optional)
    
    Returns:
        {
            "success": bool,
            "resources_deleted": list,
            "errors": list
        }
    """

@mcp.tool()
def tc_invoke(function_name: str, payload: dict, sandbox: str, aws_region: str = None, role_arn: str = None) -> dict:
    """
    Invoke a deployed TC function.
    
    Args:
        function_name: Name of the function to invoke
        payload: JSON payload to send to function
        sandbox: Sandbox name
        aws_region: AWS region (optional, uses default if not specified)
        role_arn: AWS role ARN to assume for cross-account access (optional)
    
    Returns:
        {
            "success": bool,
            "response": dict,  # Function response
            "logs": str,       # Function logs
            "duration_ms": int,
            "errors": list
        }
    """

@mcp.tool()
def tc_verify_credentials() -> dict:
    """
    Verify AWS credentials and return identity information.
    
    Returns:
        {
            "success": bool,
            "identity": {
                "account_id": str,
                "user_arn": str,
                "user_id": str
            },
            "permissions": {
                "can_compose": bool,
                "can_create": bool,
                "can_delete": bool,
                "can_invoke": bool
            },
            "errors": list
        }
    """
```

**Resources Exposed**:

```python
@mcp.resource("tc://docs/{topic}")
def get_documentation(topic: str) -> str:
    """Provide structured documentation on demand."""
    
@mcp.resource("tc://examples/{category}/{name}")
def get_example(category: str, name: str) -> str:
    """Provide example topology with annotations."""
```

**Implementation Details**:
- Use subprocess to execute tc CLI commands
- Parse YAML topologies using PyYAML
- Cache example metadata for fast queries (in-memory for Lambda, Redis for multi-instance)
- Watch examples/ directory for changes (local mode only)
- Return structured JSON for AI parsing

**Serverless-Specific Considerations**:
- TC binary packaged as Lambda Layer (Rust binary compiled for Amazon Linux 2)
- Examples index and documentation served from S3/CloudFront
- Stateless execution - no local file system dependencies
- Cold start optimization: pre-warm Lambda, lazy-load resources
- Timeout handling: Long-running operations return async job IDs
- AgentCore Gateway handles: Authentication, rate limiting, request routing, response caching

**AWS Credential Handling**:
- **Request Authentication**: AgentCore Gateway verifies AWS SigV4 signature on incoming requests
- **Identity Extraction**: Extract IAM identity (user/role) from verified signature
- **Credential Propagation**: Lambda assumes user's role or uses provided credentials for TC operations
- **Temporary Credentials**: For SSO users, exchange OIDC token for temporary AWS credentials
- **Credential Caching**: Short-lived credential cache (5 min) to reduce STS calls
- **Multi-Account Support**: STS AssumeRole for cross-account deployments

**Configuration** (for users):

*Local Mode (Docker):*
```json
{
  "mcpServers": {
    "tc": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "-v", "${PWD}:/workspace", "tc-mcp-server:latest"],
      "env": {
        "TC_BINARY_PATH": "tc"
      }
    }
  }
}
```

*Serverless Mode (AgentCore) - AWS SigV4 Authentication:*
```json
{
  "mcpServers": {
    "tc": {
      "transport": "http",
      "url": "https://agentcore-gateway.tc-functors.org/mcp",
      "auth": {
        "type": "aws-sigv4",
        "service": "execute-api",
        "region": "${AWS_REGION}",
        "credentials": {
          "accessKeyId": "${AWS_ACCESS_KEY_ID}",
          "secretAccessKey": "${AWS_SECRET_ACCESS_KEY}",
          "sessionToken": "${AWS_SESSION_TOKEN}"
        }
      }
    }
  }
}
```

*Serverless Mode (AgentCore) - SSO Authentication:*
```json
{
  "mcpServers": {
    "tc": {
      "transport": "http",
      "url": "https://agentcore-gateway.tc-functors.org/mcp",
      "auth": {
        "type": "aws-sso",
        "profile": "my-sso-profile",
        "region": "${AWS_REGION}"
      }
    }
  }
}
```

*Local Python Mode (Development):*
```json
{
  "mcpServers": {
    "tc": {
      "command": "uvx",
      "args": ["tc-mcp-server"],
      "env": {
        "TC_BINARY_PATH": "tc"
      }
    }
  }
}
```

### 3. Annotated Examples System

**Purpose**: Provide a rich library of patterns that AI assistants can learn from and reference.

**Location**: `examples/` (existing directory, enhanced with annotations)

**Annotation Format**:

Examples will use inline YAML comments for AI-parseable annotations:

```yaml
name: example-etl

# @pattern: Event-Driven ETL Pipeline
# @use-cases: Data transformation, Async processing, Event fanout
# @entities: routes, functions, events, channels
# @complexity: intermediate
# @description: Demonstrates chaining functions through events with WebSocket notification

routes:
  /api/etl:
    method: POST
    function: enhancer
    # @flow-step: 1
    # @description: HTTP endpoint receives ETL job request

functions:
  enhancer:
    function: transformer
    # @flow-step: 2
    # @description: Enhances data with additional context
    # @composition: function-to-function (local)
    
  transformer:
    function: loader
    # @flow-step: 3
    # @description: Transforms data format
    # @composition: function-to-function (local)
    
  loader:
    event: Notify
    # @flow-step: 4
    # @description: Loads data and triggers notification event
    # @composition: function-to-event

events:
  Notify:
    channel: Subscription
    # @flow-step: 5
    # @description: Broadcasts completion to WebSocket subscribers
    # @composition: event-to-channel

channels:
  Subscription:
    function: default
    # @flow-step: 6
    # @description: WebSocket channel for real-time updates
```

**Companion README Pattern**:

Each example directory includes a `README.md`:

```markdown
# [Pattern Name]

## Overview
[Brief description]

## Use Cases
- Use case 1
- Use case 2

## Architecture
[Explanation of the design]

## Data Flow
1. Step 1: [description]
2. Step 2: [description]
...

## Entity Composition
- **Route → Function**: [why this connection]
- **Function → Event**: [why this connection]

## Deployment
```bash
tc compose
tc create --sandbox dev
```

## Testing
```bash
tc invoke --function enhancer --payload '{"data": "test"}'
```

## Key Learnings
- Learning 1
- Learning 2

## Variations
- How to adapt for X
- How to extend for Y
```

**Metadata Index**:

Generate `examples/index.json` automatically:

```json
{
  "patterns": [
    {
      "name": "example-etl",
      "path": "examples/orchestrator",
      "category": "orchestrator",
      "pattern_type": "Event-Driven ETL Pipeline",
      "entities": ["routes", "functions", "events", "channels"],
      "complexity": "intermediate",
      "use_cases": ["Data transformation", "Async processing"],
      "description": "Demonstrates chaining functions...",
      "last_updated": "2024-11-14"
    }
  ]
}
```

**Automation**:
- Script to parse annotations from YAML comments
- Generate index.json on commit (GitHub Action)
- Validate annotation format in CI

### 4. Structured Documentation System

**Purpose**: Provide comprehensive, AI-parseable reference documentation.

**Location**: `docs/ai-assistant-guide/` (new directory)

**Structure**:
```
docs/ai-assistant-guide/
├── README.md                    # Overview and quick start
├── core-concepts.md             # Philosophy and fundamentals
├── entity-reference.md          # Detailed entity documentation
├── composition-patterns.md      # Pattern catalog
├── cli-commands.md              # Command reference
├── decision-trees.md            # When to use what
├── anti-patterns.md             # Common mistakes
├── testing-strategies.md        # How to test topologies
├── multi-environment.md         # Sandbox management
└── changelog.md                 # Version history
```

**Content Format**:

Each document follows a consistent structure optimized for AI parsing:

```markdown
# [Topic]

## Summary
[2-3 sentence overview]

## Concepts

### [Concept 1]
**Definition**: [Clear definition]

**When to Use**: [Specific scenarios]

**Example**:
```yaml
[Minimal example]
```

**Related**: [Links to related concepts]

## Patterns

### [Pattern Name]
**Problem**: [What problem does this solve]

**Solution**: [How to implement]

**Example**:
```yaml
[Complete example]
```

**Tradeoffs**: [Pros and cons]

## Decision Trees

### Choosing Between X and Y
```
Is the operation synchronous?
├─ Yes → Use routes + functions
└─ No → Is it event-driven?
    ├─ Yes → Use events
    └─ No → Use queues for guaranteed delivery
```

## Common Mistakes

### Mistake: [Description]
**Problem**: [Why it's wrong]

**Solution**: [How to fix]

**Example**:
```yaml
# Bad
[wrong way]

# Good
[right way]
```
```

**Cross-References**:
- Use consistent terminology from glossary
- Link between related concepts
- Reference example implementations

## Data Models

### Topology Structure

```yaml
# Complete topology schema
name: string                    # Required: Topology name

routes:                         # Optional: HTTP endpoints
  [path]:
    method: GET|POST|PUT|DELETE|PATCH
    function: string            # Function to invoke
    authorizer: string          # Optional: Auth function

functions:                      # Optional: Lambda functions
  [name]:
    function: string            # Optional: Call another function
    event: string               # Optional: Trigger an event
    mutation: string            # Optional: Call GraphQL mutation
    queue: string               # Optional: Send to queue
    runtime: string             # Optional: Runtime override
    timeout: number             # Optional: Timeout in seconds
    memory: number              # Optional: Memory in MB
    environment:                # Optional: Environment variables
      [key]: string

events:                         # Optional: EventBridge events
  [name]:
    function: string            # Optional: Invoke function
    functions: [string]         # Optional: Fanout to multiple
    state: string               # Optional: Trigger state machine
    mutation: string            # Optional: Call GraphQL mutation

mutations:                      # Optional: GraphQL API
  authorizer: string            # Optional: Auth function
  types:                        # GraphQL type definitions
    [TypeName]:
      [field]: string
  resolvers:                    # GraphQL resolvers
    [resolverName]:
      function: string
      input: string
      output: string
      subscribe: boolean        # Optional: Subscription support

queues:                         # Optional: SQS queues
  [name]:
    function: string            # Function to process messages
    batch_size: number          # Optional: Batch size
    visibility_timeout: number  # Optional: Timeout

states:                         # Optional: Step Functions
  Comment: string
  StartAt: string
  TimeoutSeconds: number
  States:
    [stateName]:
      Type: Task|Choice|Parallel|Map|Wait|Succeed|Fail
      # State-specific fields...

channels:                       # Optional: AppSync WebSocket
  [name]:
    function: string            # Handler function
    handler: string             # Optional: Handler name

pages:                          # Optional: Static sites
  [name]:
    dir: string                 # Source directory
    dist: string                # Build output directory
    bucket: string              # Optional: S3 bucket
    config_template: string     # Optional: Config file
    skip_deploy: boolean        # Optional: Skip deployment

tables:                         # Optional: DynamoDB tables
  [name]:
    schema: string              # Path to schema file
```

### Annotation Schema

```typescript
// Annotations in YAML comments
interface TopologyAnnotations {
  // Top-level annotations
  "@pattern": string;           // Pattern name
  "@use-cases": string;         // Comma-separated use cases
  "@entities": string;          // Comma-separated entity types
  "@complexity": "basic" | "intermediate" | "advanced";
  "@description": string;       // Brief description
  
  // Entity-level annotations
  "@flow-step": number;         // Step in data flow
  "@description": string;       // Entity purpose
  "@composition": string;       // Composition type (e.g., "function-to-event")
}
```

### MCP Tool Response Schema

```typescript
// Standard response format for all MCP tools
interface MCPResponse<T> {
  success: boolean;
  data?: T;
  errors?: Array<{
    code: string;
    message: string;
    details?: any;
  }>;
  warnings?: string[];
  metadata?: {
    timestamp: string;
    tc_version: string;
  };
}

// Specific response types
interface TopologyInfo {
  name: string;
  entities: {
    routes?: Record<string, RouteEntity>;
    functions?: Record<string, FunctionEntity>;
    events?: Record<string, EventEntity>;
    mutations?: MutationEntity;
    queues?: Record<string, QueueEntity>;
    states?: StateEntity;
    channels?: Record<string, ChannelEntity>;
    pages?: Record<string, PageEntity>;
    tables?: Record<string, TableEntity>;
  };
  composition_graph: Array<{
    from: { type: string; name: string };
    to: { type: string; name: string };
    relationship: string;
  }>;
}

interface PatternInfo {
  name: string;
  path: string;
  category: string;
  pattern_type: string;
  topology: string;              // YAML content
  annotations: TopologyAnnotations;
  readme: string;                // Markdown content
  use_cases: string[];
  entities_used: string[];
  complexity: string;
  data_flow: Array<{
    step: number;
    entity_type: string;
    entity_name: string;
    description: string;
  }>;
}
```

## Error Handling

### Steering Files
- **Missing files**: Kiro continues without error, just missing context
- **Invalid frontmatter**: File ignored, warning in Kiro logs
- **Broken file references**: Reference ignored, warning shown

### MCP Server
- **TC binary not found**: Return error with installation instructions
- **Invalid topology**: Return validation errors with line numbers
- **Command execution failure**: Return stderr output and exit code
- **Timeout**: Return partial results with timeout warning

**Error Response Format**:
```json
{
  "success": false,
  "errors": [
    {
      "code": "INVALID_TOPOLOGY",
      "message": "Missing required field 'name'",
      "details": {
        "file": "topology.yml",
        "line": 1
      }
    }
  ]
}
```

### Annotation Parsing
- **Invalid annotation format**: Skip annotation, continue parsing
- **Missing required annotations**: Pattern still usable, marked incomplete
- **Conflicting annotations**: Use first occurrence, warn in CI

## Testing Strategy

### Steering Files Testing
1. **Content Validation**
   - Verify markdown syntax
   - Validate frontmatter YAML
   - Check file references exist
   - Ensure examples are valid TC topologies

2. **Integration Testing**
   - Test in Kiro with sample projects
   - Verify conditional inclusion works
   - Confirm file references resolve correctly

### MCP Server Testing
1. **Unit Tests**
   - Test each tool function independently
   - Mock tc CLI execution
   - Validate response schemas
   - Test error handling paths

2. **Integration Tests**
   - Test with real tc binary
   - Validate against example topologies
   - Test with various MCP clients
   - Verify resource loading

3. **Deployment Mode Tests**
   - **Local Docker**: Test container build and execution
   - **Local Python**: Test uvx installation and execution
   - **Serverless**: Test Lambda cold start and warm execution
   - **AgentCore**: Test gateway integration and authentication

4. **End-to-End Tests**
   - Test complete workflows (compose, validate, deploy)
   - Test with AI assistant (automated prompts)
   - Verify cross-tool compatibility
   - Test local vs serverless mode parity

### Annotation System Testing
1. **Parser Tests**
   - Parse valid annotations correctly
   - Handle malformed annotations gracefully
   - Generate correct index.json

2. **CI Validation**
   - Verify all examples have required annotations
   - Check annotation consistency
   - Validate generated index.json

### Documentation Testing
1. **Content Validation**
   - Check markdown syntax
   - Verify code examples are valid
   - Test all internal links
   - Ensure consistent terminology

2. **AI Comprehension Testing**
   - Use AI to answer questions from docs
   - Verify AI can generate correct topologies
   - Test decision tree effectiveness

## Deployment Architecture

### Local Development Mode

```
┌─────────────────┐
│   AI Assistant  │
│   (Kiro/etc)    │
└────────┬────────┘
         │ stdio/HTTP
         │
┌────────▼────────────────┐
│  Docker Container       │
│  ┌──────────────────┐   │
│  │ FastMCP Server   │   │
│  │ (Python)         │   │
│  └────────┬─────────┘   │
│           │             │
│  ┌────────▼─────────┐   │
│  │  TC Binary       │   │
│  │  (mounted)       │   │
│  └──────────────────┘   │
└─────────────────────────┘
```

### Serverless Production Mode

```
┌─────────────────┐
│   AI Assistant  │
│   (Kiro/etc)    │
└────────┬────────┘
         │ HTTPS
         │
┌────────▼──────────────────┐
│  AgentCore Gateway        │
│  - Auth & Rate Limiting   │
│  - MCP Protocol Handler   │
└────────┬──────────────────┘
         │ Invoke
         │
┌────────▼──────────────────┐
│  Lambda Function          │
│  ┌──────────────────┐     │
│  │ FastMCP Server   │     │
│  │ (Python)         │     │
│  └────────┬─────────┘     │
│           │               │
│  ┌────────▼─────────┐     │
│  │  TC Binary       │     │
│  │  (Lambda Layer)  │     │
│  └──────────────────┘     │
└───────────────────────────┘
```

### Hybrid Mode (Optional)

For users who want local execution with remote examples/docs:
- MCP server runs locally (Docker/uvx)
- Queries remote AgentCore for examples index and documentation
- Executes TC commands locally for security/performance

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- Create steering files structure
- Write core concepts and entity reference
- Set up documentation directory structure
- Create annotation schema

### Phase 2: MCP Server - Local (Week 3-4)
- Implement basic MCP server with FastMCP
- Add tc_validate and tc_query_topology tools
- Add tc_list_examples and tc_get_pattern tools
- Create Dockerfile for local deployment
- Write unit tests

### Phase 3: MCP Server - Serverless (Week 5-6)
- Adapt MCP server for Lambda execution
- Create Lambda deployment package with TC binary layer
- Integrate with AgentCore Gateway
- Set up authentication and API key management
- Test serverless deployment

### Phase 4: Annotations (Week 7-8)
- Annotate existing examples
- Create annotation parser script
- Generate examples/index.json
- Add CI validation
- Deploy examples index to S3/CloudFront for serverless access

### Phase 5: Documentation (Week 9-10)
- Write comprehensive AI assistant guide
- Create decision trees
- Document anti-patterns
- Add testing strategies guide
- Deploy docs to S3/CloudFront for serverless access

### Phase 6: Integration & Testing (Week 11-12)
- End-to-end testing with multiple AI tools (local and serverless)
- Test AgentCore Gateway integration
- Gather feedback from test users
- Refine based on real usage
- Create contributor guide

### Phase 7: Automation & Production (Week 13-14)
- Set up GitHub Actions for index generation
- Implement change detection for tc_get_recent_changes
- Add automatic steering file updates
- Set up CI/CD for serverless deployment
- Create maintenance documentation
- Production launch

## Maintenance and Evolution

### Keeping Content Current

**Automated Updates**:
- GitHub Action runs on push to main
- Regenerates examples/index.json
- Updates changelog.md with new examples
- Validates all annotations

**Manual Review Triggers**:
- New TC version released → Review all docs
- New entity type added → Update entity reference
- Breaking changes → Update anti-patterns

### Adding New Examples

**Contributor Workflow**:
1. Create example in appropriate directory
2. Add inline annotations to topology.yml
3. Create README.md following template
4. Submit PR - CI validates annotations
5. On merge, index.json auto-updates

**Annotation Template** (in examples/CONTRIBUTING.md):
```yaml
# Required annotations at top of topology.yml:
# @pattern: [Pattern Name]
# @use-cases: [Use case 1], [Use case 2]
# @entities: [entity1], [entity2]
# @complexity: basic|intermediate|advanced
# @description: [One sentence description]

# Required annotations on each entity:
# @flow-step: [number]
# @description: [What this entity does]
# @composition: [entity-type]-to-[entity-type]
```

### Version Management

**Steering Files**:
- Include TC version in frontmatter
- Mark version-specific features
- Maintain compatibility notes

**MCP Server**:
- Report supported TC version range
- Gracefully handle version mismatches
- Provide upgrade guidance

**Documentation**:
- Maintain changelog with version markers
- Use badges for version-specific features
- Archive old versions in separate directory

## Design Rationale

### Why Multiple Layers?

**Steering Files**: Automatic, zero-config experience for Kiro users
**MCP Server**: Interactive capabilities for all MCP-compatible tools
**Annotations**: In-context learning from real examples
**Documentation**: Comprehensive reference for any AI tool

This layered approach ensures:
- Kiro users get the best experience automatically
- Other tools can access TC knowledge via MCP
- All tools can reference examples and docs
- Easy to maintain and extend

### Why Inline Annotations?

Keeping annotations in YAML comments (rather than separate files) ensures:
- Single source of truth
- Annotations stay synchronized with code
- Easy for contributors to add/update
- No separate file management overhead
- AI assistants see annotations in context

### Why FastMCP for Server?

FastMCP provides:
- Rapid development with Python decorators
- Automatic MCP protocol handling
- Easy deployment via uvx
- Good documentation and examples
- Active community support

### Why Not Custom DSL?

Using standard formats (YAML, Markdown, JSON) ensures:
- No learning curve for contributors
- Works with existing tools
- Easy for AI assistants to parse
- Future-proof and maintainable
- No custom tooling required

## AWS Authentication Architecture

### Authentication Flow

**1. AWS SigV4 Authentication (Recommended)**:
```
User → AI Assistant → MCP Client
  ↓
  Signs request with AWS credentials (SigV4)
  ↓
AgentCore Gateway
  ↓
  Verifies signature using AWS STS GetCallerIdentity
  ↓
  Extracts IAM identity (account, user/role ARN)
  ↓
Lambda Function
  ↓
  Assumes user's role OR uses provided credentials
  ↓
  Executes TC command with user's permissions
  ↓
  Returns result to AI Assistant
```

**2. AWS SSO Authentication**:
```
User → SSO Portal → OIDC Token
  ↓
AI Assistant → MCP Client
  ↓
  Includes OIDC token in request
  ↓
AgentCore Gateway
  ↓
  Exchanges OIDC token for AWS credentials (STS AssumeRoleWithWebIdentity)
  ↓
  Caches temporary credentials (15 min - 1 hour)
  ↓
Lambda Function
  ↓
  Uses temporary credentials for TC operations
```

**3. Standard AWS Credentials**:
```
User → Provides Access Key + Secret Key
  ↓
AI Assistant → MCP Client
  ↓
  Signs request with credentials
  ↓
AgentCore Gateway
  ↓
  Verifies credentials
  ↓
Lambda Function
  ↓
  Uses credentials for TC operations
```

### IAM Permission Requirements

**Minimum IAM Policy for TC MCP Server**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "TCReadOperations",
      "Effect": "Allow",
      "Action": [
        "cloudformation:DescribeStacks",
        "cloudformation:GetTemplate",
        "lambda:GetFunction",
        "lambda:ListFunctions",
        "apigateway:GET",
        "events:DescribeRule",
        "states:DescribeStateMachine",
        "appsync:GetGraphqlApi",
        "sqs:GetQueueAttributes"
      ],
      "Resource": "*"
    },
    {
      "Sid": "TCWriteOperations",
      "Effect": "Allow",
      "Action": [
        "cloudformation:CreateStack",
        "cloudformation:UpdateStack",
        "cloudformation:DeleteStack",
        "lambda:CreateFunction",
        "lambda:UpdateFunctionCode",
        "lambda:DeleteFunction",
        "lambda:InvokeFunction",
        "iam:CreateRole",
        "iam:AttachRolePolicy",
        "iam:PassRole",
        "s3:CreateBucket",
        "s3:PutObject",
        "apigateway:*",
        "events:PutRule",
        "events:PutTargets",
        "states:CreateStateMachine",
        "appsync:CreateGraphqlApi",
        "sqs:CreateQueue"
      ],
      "Resource": "*"
    },
    {
      "Sid": "TCLogging",
      "Effect": "Allow",
      "Action": [
        "logs:CreateLogGroup",
        "logs:CreateLogStream",
        "logs:PutLogEvents"
      ],
      "Resource": "arn:aws:logs:*:*:log-group:/aws/lambda/tc-*"
    }
  ]
}
```

**Recommended: Scoped Policy with Resource Constraints**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "TCOperationsScoped",
      "Effect": "Allow",
      "Action": [
        "cloudformation:*",
        "lambda:*",
        "apigateway:*",
        "events:*",
        "states:*",
        "appsync:*",
        "sqs:*"
      ],
      "Resource": [
        "arn:aws:cloudformation:*:*:stack/tc-*",
        "arn:aws:lambda:*:*:function:tc-*",
        "arn:aws:apigateway:*::/restapis/tc-*",
        "arn:aws:events:*:*:rule/tc-*",
        "arn:aws:states:*:*:stateMachine:tc-*",
        "arn:aws:appsync:*:*:apis/tc-*",
        "arn:aws:sqs:*:*:tc-*"
      ]
    },
    {
      "Sid": "TCIAMRoles",
      "Effect": "Allow",
      "Action": [
        "iam:CreateRole",
        "iam:AttachRolePolicy",
        "iam:PassRole"
      ],
      "Resource": "arn:aws:iam::*:role/tc-*"
    }
  ]
}
```

### Cross-Account Access

**Use Case**: Deploy TC topologies to different AWS accounts from a single MCP session.

**Implementation**:
1. User creates IAM role in target account with trust relationship to source account
2. User provides role ARN in MCP tool call: `role_arn="arn:aws:iam::123456789012:role/tc-deployer"`
3. Lambda uses STS AssumeRole to obtain temporary credentials for target account
4. TC operations execute in target account context

**Trust Policy (Target Account)**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Principal": {
        "AWS": "arn:aws:iam::SOURCE_ACCOUNT:root"
      },
      "Action": "sts:AssumeRole",
      "Condition": {
        "StringEquals": {
          "sts:ExternalId": "tc-mcp-server"
        }
      }
    }
  ]
}
```

## AgentCore Integration Details

### AgentCore Gateway Configuration

The MCP server will integrate with AgentCore Gateway for production serverless deployment:

**Gateway Features Used**:
- **Authentication**: API key-based auth for AI assistant access
- **Rate Limiting**: Per-user/per-organization limits to prevent abuse
- **Request Routing**: Route MCP tool calls to appropriate Lambda functions
- **Response Caching**: Cache expensive operations (example queries, topology validation)
- **Monitoring**: CloudWatch metrics for usage tracking and debugging

**Lambda Function Configuration**:
```yaml
# serverless.yml or SAM template
TCMCPServer:
  Type: AWS::Serverless::Function
  Properties:
    Runtime: python3.11
    Handler: mcp_server.handler
    Timeout: 30
    MemorySize: 512
    Layers:
      - !Ref TCBinaryLayer  # TC Rust binary
    Environment:
      TC_BINARY_PATH: /opt/bin/tc
      EXAMPLES_INDEX_URL: https://cdn.tc-functors.org/examples/index.json
      DOCS_BASE_URL: https://cdn.tc-functors.org/docs/
    Events:
      AgentCoreGateway:
        Type: HttpApi
        Properties:
          Path: /mcp/{proxy+}
          Method: ANY
```

**TC Binary Layer**:
- Rust binary compiled for Amazon Linux 2 (x86_64 or arm64)
- Packaged as Lambda Layer for reuse across functions
- Updated automatically via CI/CD when TC releases new version

**Static Assets (S3 + CloudFront)**:
- `examples/index.json` - Searchable examples catalog
- `docs/ai-assistant-guide/` - Documentation files
- Versioned URLs for cache busting
- CORS enabled for direct AI assistant access

### Cost Optimization

**Lambda Execution**:
- Most MCP tool calls complete in < 1 second
- Estimated cost: $0.0000002 per request (512MB, 1s execution)
- Free tier: 1M requests/month

**AgentCore Gateway**:
- Pricing based on AgentCore's model (TBD)
- Caching reduces Lambda invocations by ~70%

**S3 + CloudFront**:
- Static assets: ~10MB total
- Estimated cost: < $1/month for moderate usage

### Security Considerations

**AWS Credential-Based Authentication**:
The MCP server will support full read/write operations using the user's AWS credentials, ensuring that all TC operations respect AWS IAM permissions.

**Authentication Methods Supported**:
1. **AWS IAM Identity Center (SSO)**
   - Users authenticate via SSO portal
   - Short-lived credentials obtained via OIDC token exchange
   - Automatic credential refresh

2. **AWS Credentials (Access Key/Secret Key)**
   - Standard AWS credential chain
   - Support for credential profiles
   - Environment variable configuration

3. **AWS STS Assume Role**
   - Cross-account access support
   - Role-based permissions
   - Session token management

**Credential Flow (Serverless Mode)**:
```
┌─────────────────┐
│   AI Assistant  │
│   + User Creds  │
└────────┬────────┘
         │ HTTPS + AWS SigV4
         │
┌────────▼──────────────────┐
│  AgentCore Gateway        │
│  - Verify AWS signature   │
│  - Extract IAM identity   │
└────────┬──────────────────┘
         │ Invoke with IAM context
         │
┌────────▼──────────────────┐
│  Lambda Function          │
│  - Assumes user's role    │
│  - Executes TC with       │
│    user's permissions     │
└───────────────────────────┘
```

**Authorization Model**:
- **Principle of Least Privilege**: MCP server only has permissions granted to the user's AWS credentials
- **No Credential Storage**: Credentials never stored, only used for request signing
- **Scoped Access**: Each TC operation (compose, create, delete) requires appropriate AWS IAM permissions
- **Audit Trail**: All operations logged to CloudTrail with user identity

**Data Protection**:
- **Code Privacy**: User topology files and function code never stored on server
- **Temporary Execution**: All data in Lambda memory, cleared after execution
- **Encrypted Transit**: All communication over TLS 1.3
- **Log Sanitization**: Sensitive data (credentials, secrets) automatically redacted from logs

**Resource Isolation**:
- **No Cross-User Access**: Users can only access their own AWS resources
- **Namespace Isolation**: TC sandboxes scoped to user's AWS account
- **Network Isolation**: Lambda functions run in isolated execution environments

**Compliance**:
- **AWS Shared Responsibility Model**: User controls IAM policies, we secure the MCP server
- **Data Residency**: Lambda executes in user's chosen AWS region
- **Audit Logging**: CloudTrail integration for compliance requirements

## Open Questions

1. **Should we support multiple MCP server implementations?**
   - Python (FastMCP) for rapid development
   - Rust for performance and integration with TC binary
   - Decision: Start with Python, add Rust if needed for performance

2. **How detailed should annotations be?**
   - Risk: Too verbose clutters examples
   - Risk: Too sparse limits AI understanding
   - Decision: Required minimum + optional details

3. **Should steering files include full examples or references?**
   - Tradeoff: Inline examples vs file references
   - Decision: Use `#[[file:path]]` for large examples, inline for snippets

4. **How to handle TC version compatibility?**
   - Different TC versions may have different features
   - Decision: Document version requirements, MCP server checks compatibility

5. **Should we auto-generate steering files from docs?**
   - Pro: Single source of truth
   - Con: Less control over AI-specific formatting
   - Decision: Manual curation for now, evaluate automation later

6. **Should serverless mode support tc create/deploy operations?**
   - Decision: **YES** - Full read/write support using user's AWS credentials
   - Security: User's IAM permissions control what operations are allowed
   - All TC operations (compose, create, delete, invoke) available in serverless mode

7. **How to handle AgentCore Gateway pricing and access?**
   - Free tier for open source users?
   - Paid tier for commercial use?
   - Self-hosted option?
   - Decision: TBD based on AgentCore's pricing model and community feedback

8. **How to handle AWS credential passing in MCP protocol?**
   - Option A: AWS SigV4 signing of MCP requests (most secure)
   - Option B: Temporary STS tokens in request headers
   - Option C: OIDC token exchange for SSO users
   - Decision: Support all three methods, with SigV4 as recommended approach

9. **Should we support cross-account TC operations?**
   - Use case: Deploy to different AWS accounts from single MCP session
   - Implementation: STS AssumeRole with user-provided role ARNs
   - Decision: Yes, support via explicit role ARN parameter in MCP tools
