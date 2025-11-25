# Implementation Plan

- [x] 1. Set up project structure and dependencies
  - Add AWS SDK dependencies to `lib/scaffolder/Cargo.toml` (aws-config, aws-sdk-bedrockruntime)
  - Add configuration dependencies (dotenv, toml, async-trait)
  - Add testing dependencies (proptest)
  - Create `lib/scaffolder/src/provider/` directory structure
  - _Requirements: 6.1, 6.2_

- [x] 2. Implement configuration module
- [x] 2.1 Create configuration types and enums
  - Define `LlmConfig` struct with provider, model, region, and profile fields
  - Define `LlmProvider` enum with Bedrock and Anthropic variants
  - Implement string parsing for provider enum with validation
  - _Requirements: 2.1, 2.5_

- [x] 2.2 Write property test for invalid provider rejection
  - **Property 2: Invalid provider rejection**
  - **Validates: Requirements 2.5**

- [x] 2.3 Implement configuration loading from environment
  - Implement `LlmConfig::from_env()` to read from environment variables
  - Support TC_LLM_PROVIDER, TC_LLM_MODEL, AWS_REGION, AWS_PROFILE
  - _Requirements: 2.2, 3.2, 4.2, 5.2_

- [x] 2.4 Implement configuration loading from file
  - Implement `LlmConfig::from_file()` to parse config.toml
  - Support [llm] section with provider and model fields
  - Support [llm.aws] section with region and profile fields
  - _Requirements: 2.3, 3.3, 4.3, 5.3_

- [x] 2.5 Implement configuration merging with precedence
  - Implement `LlmConfig::merge()` to combine CLI, env, and file configs
  - Ensure CLI > Env > File > Default precedence for all fields
  - Implement `LlmConfig::default()` with Bedrock as default provider
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3_

- [x] 2.6 Write property test for configuration precedence
  - **Property 1: Configuration precedence hierarchy**
  - **Validates: Requirements 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3**

- [x] 2.7 Write unit tests for configuration module
  - Test default provider is Bedrock
  - Test invalid provider returns error with valid options
  - Test config file parsing with various formats
  - _Requirements: 1.1, 2.4, 2.5_

- [x] 3. Implement .env file loading
- [x] 3.1 Add dotenv loading to configuration resolution
  - Call `dotenv::dotenv()` before loading environment configuration
  - Handle missing .env file gracefully (no error)
  - Ensure shell environment variables take precedence over .env
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5_

- [x] 3.2 Write property test for shell environment precedence
  - **Property 8: Shell environment precedence over .env**
  - **Validates: Requirements 9.3**

- [x] 3.3 Write property test for .env variable loading
  - **Property 7: Environment variable loading from .env**
  - **Validates: Requirements 9.2**

- [x] 4. Define provider trait and error types
- [x] 4.1 Create provider trait definition
  - Define `LlmProvider` trait with send() and extract_code() methods
  - Add name() method for logging/debugging
  - Use async_trait for async methods
  - _Requirements: 6.1, 6.4_

- [x] 4.2 Define common error types
  - Create `LlmError` enum with AuthenticationError, NetworkError, ModelNotAvailable, InvalidResponse, ConfigurationError
  - Implement Display trait with helpful error messages and hints
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

- [x] 4.3 Write property test for provider error mapping
  - **Property 4: Provider error mapping**
  - **Validates: Requirements 6.5**

- [x] 5. Implement Anthropic provider
- [x] 5.1 Create AnthropicProvider struct and implementation
  - Move existing Anthropic API logic from llm.rs to provider/anthropic.rs
  - Implement LlmProvider trait for AnthropicProvider
  - Use CLAUDE_API_KEY from environment
  - Maintain existing request format and behavior
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 5.2 Implement error handling for Anthropic provider
  - Map HTTP status codes to LlmError variants
  - Provide clear error message when CLAUDE_API_KEY is missing
  - _Requirements: 7.5, 8.5_

- [x] 5.3 Write property test for Anthropic ignoring AWS config
  - **Property 6: Anthropic ignores AWS configuration**
  - **Validates: Requirements 4.5, 5.5**

- [x] 5.4 Write unit tests for Anthropic provider
  - Test API key validation
  - Test error mapping from HTTP status codes
  - Test extract_code functionality
  - _Requirements: 8.1, 8.2, 8.3, 8.5_

- [x] 6. Implement Bedrock provider
- [x] 6.1 Create BedrockProvider struct with AWS SDK client
  - Initialize AWS SDK client with optional region and profile
  - Use aws-config for credential resolution
  - Support AWS SSO and profile-based authentication
  - _Requirements: 1.2, 1.3, 1.4, 4.1, 4.2, 4.3, 4.4, 5.1, 5.2, 5.3, 5.4_

- [x] 6.2 Implement LlmProvider trait for BedrockProvider
  - Implement send() using Bedrock Converse API
  - Build Message with user role and text content
  - Configure inference parameters (max_tokens, temperature)
  - Extract text response from Converse API output
  - _Requirements: 1.1, 1.2_

- [x] 6.3 Implement error handling for Bedrock provider
  - Map Bedrock SDK errors to LlmError variants
  - Handle AccessDeniedException, ModelTimeoutException, ModelNotReadyException
  - Provide clear authentication error messages with resolution hints
  - _Requirements: 1.5, 7.1, 7.2, 7.3, 7.4_

- [x] 6.4 Write property test for AWS error propagation
  - **Property 5: AWS error propagation**
  - **Validates: Requirements 7.2**

- [x] 6.5 Write unit tests for Bedrock provider
  - Test credential resolution with different sources
  - Test region and profile configuration
  - Test error mapping from Bedrock SDK errors
  - Test model ID formatting
  - _Requirements: 1.2, 1.3, 1.4, 4.4, 5.4_

- [x] 7. Refactor LLM module to use provider abstraction
- [x] 7.1 Update llm.rs to use provider trait
  - Implement resolve_config() function to load and merge configuration
  - Implement create_provider() function to instantiate correct provider
  - Update scaffold() function to use provider abstraction
  - Update send() function to use provider abstraction
  - Keep extract_code() as standalone utility function
  - _Requirements: 6.2, 6.3, 6.4_

- [x] 7.2 Write property test for provider selection
  - **Property 3: Provider selection correctness**
  - **Validates: Requirements 6.2**

- [x] 7.3 Write unit tests for provider factory
  - Test create_provider returns correct type for each enum value
  - Test error handling when credentials are missing
  - _Requirements: 6.2, 1.5, 8.5_

- [x] 8. Add CLI parameter support
- [x] 8.1 Update CLI argument parsing
  - Add --llm-provider, --llm-model, --aws-region, --aws-profile arguments
  - Wire CLI arguments into configuration resolution
  - Ensure CLI arguments take highest precedence
  - _Requirements: 2.1, 3.1, 4.1, 5.1_

- [x] 8.2 Write unit tests for CLI argument handling
  - Test CLI arguments override environment and file config
  - Test each CLI argument independently
  - _Requirements: 2.1, 3.1, 4.1, 5.1_

- [x] 9. Update model identifier handling
- [x] 9.1 Implement model ID defaults per provider
  - Set Bedrock default to anthropic.claude-3-5-sonnet-20241022-v2:0
  - Set Anthropic default to claude-sonnet-4-5-20250929
  - Apply defaults when no model is configured
  - _Requirements: 3.4, 3.5_

- [x] 9.2 Write unit tests for model defaults
  - Test correct default model for each provider
  - Test model override via configuration
  - _Requirements: 3.4, 3.5_

- [x] 10. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 11. Integration testing preparation
- [x] 11.1 Create integration test for Bedrock provider
  - Test actual Bedrock API call (marked with #[ignore])
  - Test AWS SSO authentication flow
  - Test profile-based authentication
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 11.2 Create integration test for Anthropic provider
  - Test actual Anthropic API call (marked with #[ignore])
  - Test backward compatibility with existing implementation
  - _Requirements: 8.1, 8.2, 8.3_

- [x] 11.3 Create end-to-end scaffolding test
  - Test complete scaffold workflow with Bedrock
  - Test complete scaffold workflow with Anthropic
  - Verify generated topology.yml files
  - _Requirements: 1.1, 8.1_

- [x] 12. Documentation and examples
- [x] 12.1 Update README with Bedrock configuration examples
  - Document environment variables
  - Document config.toml format
  - Document CLI parameters
  - Provide AWS SSO setup instructions
  - Provide migration guide from Anthropic to Bedrock
  - _Requirements: 1.1, 2.1, 3.1, 4.1, 5.1_

- [x] 12.2 Create example config.toml files
  - Example for Bedrock with SSO
  - Example for Bedrock with profile
  - Example for Anthropic API
  - _Requirements: 2.3, 3.3, 4.3, 5.3_

- [x] 13. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
