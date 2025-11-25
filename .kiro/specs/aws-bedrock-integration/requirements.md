# Requirements Document

## Introduction

This document specifies the requirements for adding AWS Bedrock support to the tc (topology composer) scaffolder module. Currently, the scaffolder only supports direct Anthropic Claude API calls via the `CLAUDE_API_KEY` environment variable. This enhancement will enable users to leverage Claude models through AWS Bedrock, providing better integration with AWS infrastructure, SSO authentication, and cost management through AWS billing.

## Glossary

- **Scaffolder**: The tc module responsible for generating topology YAML files using LLM assistance
- **AWS Bedrock**: Amazon's managed service for accessing foundation models including Claude
- **LLM Provider**: The backend service used to access Claude models (either Anthropic API or AWS Bedrock)
- **AWS SDK**: The Rust AWS SDK used for authenticating and communicating with AWS services
- **SSO**: Single Sign-On authentication mechanism for AWS
- **Credential Chain**: AWS SDK's hierarchical method for resolving credentials from multiple sources
- **Config File**: The tc configuration file (typically `config.toml`) for storing user preferences
- **CLI Parameters**: Command-line arguments passed to tc commands
- **Model Identifier**: A string specifying which Claude model to use (e.g., "claude-sonnet-4-5-20250929")

## Requirements

### Requirement 1

**User Story:** As a tc user, I want to use Claude through AWS Bedrock by default, so that I can leverage my existing AWS credentials and billing without managing separate API keys.

#### Acceptance Criteria

1. WHEN the Scaffolder initializes without explicit configuration THEN the Scaffolder SHALL use AWS Bedrock as the default LLM Provider
2. WHEN using AWS Bedrock THEN the Scaffolder SHALL authenticate using the standard AWS SDK credential chain
3. WHEN AWS credentials are available through SSO THEN the Scaffolder SHALL successfully authenticate to AWS Bedrock
4. WHEN AWS credentials are available through a named profile THEN the Scaffolder SHALL successfully authenticate to AWS Bedrock
5. WHEN AWS credentials are unavailable THEN the Scaffolder SHALL provide a clear error message indicating authentication failure

### Requirement 2

**User Story:** As a tc user, I want to configure which LLM provider to use, so that I can choose between Anthropic API and AWS Bedrock based on my infrastructure setup.

#### Acceptance Criteria

1. WHEN a user specifies a provider via CLI parameter THEN the Scaffolder SHALL use that provider regardless of other configuration sources
2. WHEN a user sets a provider via environment variable and no CLI parameter is provided THEN the Scaffolder SHALL use the environment variable value
3. WHEN a user sets a provider in the config file and no CLI parameter or environment variable is provided THEN the Scaffolder SHALL use the config file value
4. WHEN no provider is explicitly configured THEN the Scaffolder SHALL default to AWS Bedrock
5. WHEN an invalid provider name is specified THEN the Scaffolder SHALL return an error with valid provider options

### Requirement 3

**User Story:** As a tc user, I want to specify which Claude model to use, so that I can choose between different model versions and capabilities based on my needs.

#### Acceptance Criteria

1. WHEN a user specifies a model via CLI parameter THEN the Scaffolder SHALL use that Model Identifier
2. WHEN a user sets a model via environment variable and no CLI parameter is provided THEN the Scaffolder SHALL use the environment variable value
3. WHEN a user sets a model in the config file and no CLI parameter or environment variable is provided THEN the Scaffolder SHALL use the config file value
4. WHEN no model is explicitly configured THEN the Scaffolder SHALL use a sensible default model for the selected provider
5. WHEN using AWS Bedrock THEN the Scaffolder SHALL format the Model Identifier according to Bedrock's naming convention

### Requirement 4

**User Story:** As a tc user, I want to specify an AWS region for Bedrock, so that I can use Bedrock in my preferred region or where my organization has access.

#### Acceptance Criteria

1. WHEN a user specifies a region via CLI parameter THEN the Scaffolder SHALL use that region for AWS Bedrock requests
2. WHEN a user sets a region via environment variable and no CLI parameter is provided THEN the Scaffolder SHALL use the environment variable value
3. WHEN a user sets a region in the config file and no CLI parameter or environment variable is provided THEN the Scaffolder SHALL use the config file value
4. WHEN no region is explicitly configured THEN the Scaffolder SHALL use the AWS SDK default region resolution
5. WHEN using Anthropic API as the provider THEN the Scaffolder SHALL ignore region configuration

### Requirement 5

**User Story:** As a tc user, I want to specify an AWS profile for Bedrock authentication, so that I can use different AWS accounts or roles for different projects.

#### Acceptance Criteria

1. WHEN a user specifies a profile via CLI parameter THEN the Scaffolder SHALL use that profile for AWS authentication
2. WHEN a user sets a profile via environment variable and no CLI parameter is provided THEN the Scaffolder SHALL use the environment variable value
3. WHEN a user sets a profile in the config file and no CLI parameter or environment variable is provided THEN the Scaffolder SHALL use the config file value
4. WHEN no profile is explicitly configured THEN the Scaffolder SHALL use the AWS SDK default profile resolution
5. WHEN using Anthropic API as the provider THEN the Scaffolder SHALL ignore profile configuration

### Requirement 6

**User Story:** As a developer integrating AWS Bedrock, I want the LLM client to have a clean abstraction, so that adding future providers is straightforward and the codebase remains maintainable.

#### Acceptance Criteria

1. WHEN implementing provider support THEN the system SHALL define a common trait for LLM providers
2. WHEN a provider is selected THEN the system SHALL instantiate the appropriate provider implementation
3. WHEN adding a new provider THEN the developer SHALL only need to implement the common trait
4. WHEN making LLM requests THEN the calling code SHALL not need to know which provider is being used
5. WHEN handling provider-specific errors THEN the system SHALL map them to common error types

### Requirement 7

**User Story:** As a tc user, I want clear error messages when LLM operations fail, so that I can quickly diagnose and fix configuration issues.

#### Acceptance Criteria

1. WHEN AWS Bedrock authentication fails THEN the system SHALL provide an error message indicating credential issues and suggesting resolution steps
2. WHEN an AWS Bedrock API call fails THEN the system SHALL provide an error message including the AWS error code and message
3. WHEN a specified model is not available in Bedrock THEN the system SHALL provide an error message listing available models
4. WHEN network connectivity fails THEN the system SHALL provide an error message indicating connection issues
5. WHEN the Anthropic API key is missing for Anthropic provider THEN the system SHALL provide an error message indicating the missing environment variable

### Requirement 8

**User Story:** As a tc user, I want the existing Anthropic API integration to continue working, so that I can maintain backward compatibility with my current workflows.

#### Acceptance Criteria

1. WHEN a user sets the provider to Anthropic THEN the Scaffolder SHALL use the direct Anthropic API
2. WHEN using Anthropic provider THEN the Scaffolder SHALL authenticate using the CLAUDE_API_KEY environment variable from the environment or .env file
3. WHEN using Anthropic provider THEN the Scaffolder SHALL use the same request format as the current implementation
4. WHEN no provider is configured and CLAUDE_API_KEY is set THEN the Scaffolder SHALL still default to AWS Bedrock
5. WHEN using Anthropic provider and CLAUDE_API_KEY is not set THEN the Scaffolder SHALL provide a clear error message

### Requirement 9

**User Story:** As a tc user, I want to load API keys from a .env file, so that I can manage sensitive credentials securely without hardcoding them in my shell configuration.

#### Acceptance Criteria

1. WHEN the Scaffolder initializes THEN the system SHALL attempt to load environment variables from a .env file in the current directory
2. WHEN a .env file exists THEN the system SHALL parse and load all key-value pairs as environment variables
3. WHEN an environment variable is set both in the shell and .env file THEN the shell environment variable SHALL take precedence
4. WHEN using non-Bedrock providers THEN the system SHALL read API keys from environment variables loaded from .env or shell
5. WHEN a .env file does not exist THEN the system SHALL continue without error and use only shell environment variables
