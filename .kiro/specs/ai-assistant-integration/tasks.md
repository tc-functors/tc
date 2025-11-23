# Implementation Plan

- [ ] 1. Set up project structure and documentation foundation
  - Create `.kiro/steering/` directory structure
  - Create `docs/ai-assistant-guide/` directory structure
  - Create `mcp-server/` directory for MCP server implementation
  - Set up Python project structure with FastMCP dependencies
  - _Requirements: 11.1, 11.2_

- [ ] 2. Create core steering files for Kiro
  - [ ] 2.1 Write tc-core-concepts.md steering file
    - Document Cloud Functor concept and TC philosophy
    - Explain entity-based composition model
    - Include decision trees for entity selection
    - Add inline examples of basic topologies
    - _Requirements: 1.1, 1.2, 1.3, 5.1, 5.2_

  - [ ] 2.2 Write tc-entity-reference.md steering file
    - Document all 7 TC entities (functions, events, mutations, queues, routes, states, channels)
    - Include entity-specific syntax and options
    - Provide examples for each entity type
    - Document entity composition patterns
    - _Requirements: 1.2, 5.2_

  - [ ] 2.3 Write tc-composition-patterns.md steering file
    - Document common composition patterns (route-function, function-event, etc.)
    - Include use cases for each pattern
    - Add decision trees for pattern selection
    - Document anti-patterns to avoid
    - _Requirements: 1.3, 5.2, 5.3_

  - [ ] 2.4 Write tc-topology-guide.md with conditional inclusion
    - Configure fileMatch for "topology.yml" files
    - Document topology file structure and syntax
    - Include validation rules and common mistakes
    - Add examples of complete topologies
    - _Requirements: 2.1, 2.2, 2.3, 5.4_

  - [ ] 2.5 Write tc-function-guide.md with conditional inclusion
    - Configure fileMatch for "*/functions/*" files
    - Document function implementation patterns
    - Include runtime-specific guidance
    - Add examples of function code
    - _Requirements: 5.4_

  - [ ] 2.6 Write tc-testing-guide.md for manual inclusion
    - Configure as manual inclusion (#tc-testing)
    - Document testing strategies for topologies
    - Include tc emulate usage examples
    - Add test payload examples
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [ ] 3. Implement local MCP server with FastMCP
  - [ ] 3.1 Set up FastMCP server project structure
    - Initialize Python project with FastMCP
    - Configure dependencies (PyYAML, boto3, etc.)
    - Set up logging and error handling
    - Create configuration management
    - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

  - [ ] 3.2 Implement tc_validate tool
    - Parse topology YAML files
    - Validate topology structure and syntax
    - Check entity references and dependencies
    - Return structured validation results
    - _Requirements: 2.5, 4.2_

  - [ ] 3.3 Implement tc_query_topology tool
    - Parse topology files into structured data
    - Extract entity definitions
    - Build composition graph (entity relationships)
    - Return JSON representation of topology
    - _Requirements: 4.1_

  - [ ] 3.4 Implement tc_compose tool
    - Execute tc compose command via subprocess
    - Parse infrastructure output
    - Handle errors and warnings
    - Return structured composition results
    - _Requirements: 3.1, 3.2, 4.4, 4.5_

  - [ ] 3.5 Implement example query tools
    - Implement tc_list_examples tool to query examples index
    - Implement tc_get_pattern tool to retrieve pattern details
    - Parse example annotations from YAML comments
    - Return structured pattern information
    - _Requirements: 4.1, 6.2_

  - [ ] 3.6 Implement tc_get_recent_changes tool
    - Query git history for recent changes
    - Track new and modified examples
    - Track documentation updates
    - Return structured change information
    - _Requirements: 12.1, 12.2, 12.3, 12.4_

  - [ ] 3.7 Create Dockerfile for local deployment
    - Create multi-stage Dockerfile
    - Include TC binary in container
    - Configure volume mounts for workspace access
    - Optimize for fast startup
    - _Requirements: 4.5_

  - [ ]* 3.8 Write unit tests for MCP tools
    - Test each tool function with mock data
    - Test error handling paths
    - Validate response schemas
    - Test YAML parsing edge cases
    - _Requirements: 4.5_

- [ ] 4. Implement AWS authentication and serverless deployment
  - [ ] 4.1 Implement AWS credential handling
    - Add AWS SigV4 signature verification
    - Implement SSO token exchange (OIDC to AWS credentials)
    - Support standard AWS credential chain
    - Add credential caching with TTL
    - _Requirements: 4.5_

  - [ ] 4.2 Implement tc_verify_credentials tool
    - Call AWS STS GetCallerIdentity
    - Extract IAM identity information
    - Check permissions for TC operations
    - Return identity and permission status
    - _Requirements: 4.5_

  - [ ] 4.3 Implement tc_create tool
    - Execute tc create command with user's AWS credentials
    - Handle cross-account access via AssumeRole
    - Parse deployment results
    - Return created resources and endpoints
    - _Requirements: 3.1, 3.2, 4.4, 4.5_

  - [ ] 4.4 Implement tc_delete tool
    - Execute tc delete command with user's AWS credentials
    - Handle cross-account access via AssumeRole
    - Track deleted resources
    - Return deletion status
    - _Requirements: 3.1, 3.2, 4.4, 4.5_

  - [ ] 4.5 Implement tc_invoke tool
    - Execute tc invoke command with user's AWS credentials
    - Handle cross-account access via AssumeRole
    - Capture function response and logs
    - Return invocation results
    - _Requirements: 3.1, 3.2, 4.4, 4.5, 7.4_

  - [ ] 4.6 Implement tc_list_sandboxes tool
    - Query AWS CloudFormation for TC stacks
    - Parse stack information
    - Return list of deployed sandboxes
    - _Requirements: 4.3, 8.4_

  - [ ] 4.7 Create Lambda deployment package
    - Package MCP server code for Lambda
    - Create TC binary Lambda Layer (compile Rust for Amazon Linux 2)
    - Configure Lambda function settings (timeout, memory, environment)
    - Set up IAM role for Lambda execution
    - _Requirements: 4.5_

  - [ ] 4.8 Integrate with AgentCore Gateway
    - Configure AgentCore Gateway routing to Lambda
    - Set up authentication middleware for AWS credentials
    - Configure rate limiting and caching
    - Set up CloudWatch monitoring
    - _Requirements: 4.5_

  - [ ]* 4.9 Write integration tests for serverless mode
    - Test Lambda cold start and warm execution
    - Test AWS credential propagation
    - Test cross-account access
    - Test AgentCore Gateway integration
    - _Requirements: 4.5_

- [ ] 5. Create annotated examples system
  - [ ] 5.1 Define annotation schema and format
    - Document annotation syntax in YAML comments
    - Define required vs optional annotations
    - Create annotation validation rules
    - Write contributor guide for annotations
    - _Requirements: 6.1, 6.3, 11.2, 11.3, 11.4_

  - [ ] 5.2 Annotate existing composition examples
    - Add annotations to examples/composition/* topologies
    - Document data flow and entity relationships
    - Add use case descriptions
    - Create README.md for each example
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ] 5.3 Annotate existing pattern examples
    - Add annotations to examples/patterns/* topologies
    - Document architectural decisions
    - Add deployment and testing notes
    - Create README.md for each pattern
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

  - [ ] 5.4 Annotate state machine examples
    - Add annotations to examples/states/* topologies
    - Document state machine workflows
    - Add orchestration patterns
    - Create README.md for each example
    - _Requirements: 6.1, 6.2, 6.3, 6.4_

  - [ ] 5.5 Create annotation parser script
    - Parse YAML comments for annotations
    - Extract metadata from annotations
    - Validate annotation completeness
    - Generate structured annotation data
    - _Requirements: 11.1, 12.1_

  - [ ] 5.6 Generate examples/index.json
    - Run annotation parser on all examples
    - Aggregate example metadata
    - Create searchable index file
    - Include last updated timestamps
    - _Requirements: 11.1, 12.1, 12.2_

  - [ ] 5.7 Set up CI validation for annotations
    - Create GitHub Action to validate annotations
    - Check for required annotations on new examples
    - Validate annotation syntax
    - Auto-generate index.json on merge
    - _Requirements: 11.1, 12.1, 12.2, 12.3_

- [ ] 6. Write comprehensive AI assistant documentation
  - [ ] 6.1 Write docs/ai-assistant-guide/README.md
    - Overview of TC for AI assistants
    - Quick start guide
    - Links to detailed guides
    - Version information
    - _Requirements: 9.1, 9.2, 10.1_

  - [ ] 6.2 Write docs/ai-assistant-guide/core-concepts.md
    - TC philosophy and design principles
    - Cloud Functor concept
    - Entity-based composition model
    - Namespace and sandbox concepts
    - _Requirements: 1.1, 1.2, 1.4, 10.1_

  - [ ] 6.3 Write docs/ai-assistant-guide/entity-reference.md
    - Detailed documentation for all 7 entities
    - Syntax and configuration options
    - Entity-specific examples
    - Entity composition rules
    - _Requirements: 1.2, 2.1, 2.2, 2.3, 2.4, 10.1_

  - [ ] 6.4 Write docs/ai-assistant-guide/composition-patterns.md
    - Catalog of common composition patterns
    - Use cases and problem-solution pairs
    - Pattern examples with explanations
    - Pattern selection guidance
    - _Requirements: 1.3, 6.1, 6.2, 10.1_

  - [ ] 6.5 Write docs/ai-assistant-guide/cli-commands.md
    - Complete TC CLI command reference
    - Command usage examples
    - Common workflows (build, compose, create, deploy)
    - Error interpretation guide
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 10.1_

  - [ ] 6.6 Write docs/ai-assistant-guide/decision-trees.md
    - Decision trees for entity selection
    - When to use which composition pattern
    - Synchronous vs asynchronous decision tree
    - Testing strategy decision tree
    - _Requirements: 1.3, 5.5, 10.1_

  - [ ] 6.7 Write docs/ai-assistant-guide/anti-patterns.md
    - Common mistakes and how to avoid them
    - Anti-patterns with explanations
    - Correct alternatives for each anti-pattern
    - Debugging tips
    - _Requirements: 2.5, 5.3, 7.3, 10.1_

  - [ ] 6.8 Write docs/ai-assistant-guide/testing-strategies.md
    - Local testing with tc emulate
    - Test payload creation
    - Integration testing approaches
    - Debugging deployed topologies
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 10.1_

  - [ ] 6.9 Write docs/ai-assistant-guide/multi-environment.md
    - Sandbox concept and usage
    - Environment-specific configuration
    - Deployment strategies (dev, staging, prod)
    - Namespace and tagging conventions
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5, 10.1_

  - [ ] 6.10 Write docs/ai-assistant-guide/changelog.md
    - Version history with feature additions
    - Breaking changes documentation
    - Migration guides for version updates
    - Deprecation notices
    - _Requirements: 9.1, 9.2, 9.3, 9.4, 12.4_

- [ ] 7. Deploy static assets for serverless mode
  - [ ] 7.1 Set up S3 bucket and CloudFront distribution
    - Create S3 bucket for static assets
    - Configure CloudFront distribution
    - Set up CORS for AI assistant access
    - Configure cache invalidation
    - _Requirements: 10.2, 10.3_

  - [ ] 7.2 Deploy examples index to CDN
    - Upload examples/index.json to S3
    - Configure versioned URLs for cache busting
    - Set up automatic updates on git push
    - Test CDN access from MCP server
    - _Requirements: 12.1, 12.2_

  - [ ] 7.3 Deploy documentation to CDN
    - Upload docs/ai-assistant-guide/* to S3
    - Configure directory structure for easy access
    - Set up automatic updates on git push
    - Test CDN access from MCP server
    - _Requirements: 12.3_

- [ ] 8. Integration testing and refinement
  - [ ] 8.1 Test with Kiro
    - Test steering file auto-loading
    - Test conditional file inclusion
    - Verify AI can generate correct topologies
    - Test file reference syntax (#[[file:path]])
    - _Requirements: 5.1, 5.2, 5.4, 10.1_

  - [ ] 8.2 Test MCP server with multiple AI tools
    - Test with Kiro (MCP client)
    - Test with Claude Desktop
    - Test with other MCP-compatible tools
    - Verify cross-tool consistency
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_

  - [ ] 8.3 Test AWS authentication flows
    - Test SigV4 authentication
    - Test SSO authentication
    - Test standard AWS credentials
    - Test cross-account access
    - _Requirements: 4.5_

  - [ ] 8.4 Test end-to-end workflows
    - Test complete workflow: validate → compose → create → invoke
    - Test multi-environment deployments
    - Test error handling and recovery
    - Test with real TC topologies
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 7.1, 7.2, 7.3, 7.4, 7.5_

  - [ ] 8.5 Gather feedback from test users
    - Recruit beta testers from TC community
    - Collect feedback on AI assistant accuracy
    - Identify gaps in documentation
    - Refine based on real usage patterns
    - _Requirements: 10.5_

  - [ ] 8.6 Performance optimization
    - Optimize Lambda cold start time
    - Implement response caching
    - Optimize example index queries
    - Reduce MCP tool response times
    - _Requirements: 4.5_

- [ ] 9. Automation and CI/CD setup
  - [ ] 9.1 Create GitHub Action for annotation validation
    - Validate annotations on PR
    - Check for required annotations
    - Validate annotation syntax
    - Report validation errors
    - _Requirements: 11.1, 11.4_

  - [ ] 9.2 Create GitHub Action for index generation
    - Run annotation parser on merge to main
    - Generate examples/index.json
    - Commit and push updated index
    - Deploy to S3/CloudFront
    - _Requirements: 11.1, 12.1, 12.2_

  - [ ] 9.3 Create GitHub Action for documentation deployment
    - Deploy docs to S3 on merge to main
    - Invalidate CloudFront cache
    - Update version markers
    - Notify on deployment success
    - _Requirements: 12.3_

  - [ ] 9.4 Create GitHub Action for MCP server deployment
    - Build Lambda deployment package
    - Build and push TC binary layer
    - Deploy to AWS Lambda
    - Update AgentCore Gateway configuration
    - _Requirements: 4.5_

  - [ ] 9.5 Set up monitoring and alerting
    - Configure CloudWatch dashboards
    - Set up error rate alerts
    - Monitor Lambda performance metrics
    - Track MCP tool usage statistics
    - _Requirements: 4.5_

- [ ] 10. Documentation and launch preparation
  - [ ] 10.1 Write contributor guide for examples
    - Document annotation template
    - Explain how to add new examples
    - Provide README.md template
    - Include validation checklist
    - _Requirements: 11.1, 11.2, 11.3, 11.4_

  - [ ] 10.2 Write user guide for MCP server setup
    - Document local mode setup (Docker, uvx)
    - Document serverless mode setup (AgentCore)
    - Explain AWS credential configuration
    - Provide troubleshooting guide
    - _Requirements: 10.1, 10.2, 10.3_

  - [ ] 10.3 Create example AI assistant prompts
    - Provide sample prompts for common tasks
    - Include prompts for learning TC
    - Add prompts for debugging
    - Document prompt best practices
    - _Requirements: 10.1_

  - [ ] 10.4 Write maintenance documentation
    - Document update procedures
    - Explain version management
    - Provide runbook for common issues
    - Document backup and recovery
    - _Requirements: 9.5, 12.1, 12.2, 12.3_

  - [ ] 10.5 Prepare launch announcement
    - Write blog post about AI assistant integration
    - Create demo video
    - Prepare documentation site updates
    - Plan community outreach
    - _Requirements: 10.5_
