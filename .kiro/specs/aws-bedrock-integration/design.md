# Design Document: AWS Bedrock Integration

## Overview

This design adds AWS Bedrock support to the tc scaffolder module, enabling users to access Claude models through AWS infrastructure. The implementation maintains backward compatibility with the existing Anthropic API integration while defaulting to AWS Bedrock for better AWS ecosystem integration.

The design introduces a provider abstraction layer that allows seamless switching between Anthropic's direct API and AWS Bedrock, with configuration through CLI parameters, environment variables, and config files following a clear precedence hierarchy.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Scaffolder Module                        │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           Configuration Resolution Layer                │ │
│  │  (CLI Args → Env Vars → Config File → Defaults)       │ │
│  └────────────────────────────────────────────────────────┘ │
│                           │                                  │
│                           ▼                                  │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              LLM Provider Trait                        │ │
│  │  - send(prompt) -> Result<String>                     │ │
│  │  - extract_code(response) -> String                   │ │
│  └────────────────────────────────────────────────────────┘ │
│           │                              │                   │
│           ▼                              ▼                   │
│  ┌──────────────────┐         ┌──────────────────────┐     │
│  │ AnthropicProvider│         │  BedrockProvider     │     │
│  │                  │         │                      │     │
│  │ - API Key Auth   │         │ - AWS SDK Auth       │     │
│  │ - Direct HTTP    │         │ - Converse API       │     │
│  └──────────────────┘         └──────────────────────┘     │
│           │                              │                   │
└───────────┼──────────────────────────────┼───────────────────┘
            │                              │
            ▼                              ▼
   ┌────────────────┐           ┌──────────────────────┐
   │ Anthropic API  │           │   AWS Bedrock        │
   │ api.anthropic  │           │   bedrock-runtime    │
   │    .com        │           │   us-east-1          │
   └────────────────┘           └──────────────────────┘
```

### Configuration Precedence

Configuration follows this precedence (highest to lowest):
1. CLI parameters (`--llm-provider`, `--llm-model`, `--aws-region`, `--aws-profile`)
2. Environment variables (`TC_LLM_PROVIDER`, `TC_LLM_MODEL`, `AWS_REGION`, `AWS_PROFILE`)
3. Config file (`config.toml`)
4. Defaults (Bedrock provider, region from AWS SDK, default model)

### Module Structure

```
lib/scaffolder/
├── src/
│   ├── lib.rs              # Public API, unchanged
│   ├── llm.rs              # Refactored to use provider abstraction
│   ├── config.rs           # NEW: Configuration resolution
│   ├── provider/
│   │   ├── mod.rs          # NEW: Provider trait definition
│   │   ├── anthropic.rs    # NEW: Anthropic provider implementation
│   │   └── bedrock.rs      # NEW: AWS Bedrock provider implementation
│   └── function.rs         # Unchanged
```

## Components and Interfaces

### 1. Configuration Module (`config.rs`)

**Purpose**: Resolve LLM configuration from multiple sources with proper precedence.

```rust
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub aws_region: Option<String>,
    pub aws_profile: Option<String>,
}

pub enum LlmProvider {
    Bedrock,
    Anthropic,
}

impl LlmConfig {
    /// Load configuration from environment, including .env file
    pub fn from_env() -> Result<Self, ConfigError>;
    
    /// Load configuration from config file
    pub fn from_file(path: &str) -> Result<Self, ConfigError>;
    
    /// Merge configurations with CLI taking precedence
    pub fn merge(cli: Option<Self>, env: Self, file: Option<Self>) -> Self;
    
    /// Get default configuration
    pub fn default() -> Self;
}
```

**Configuration File Format** (`config.toml`):
```toml
[llm]
provider = "bedrock"  # or "anthropic"
model = "anthropic.claude-3-5-sonnet-20241022-v2:0"

[llm.aws]
region = "us-east-1"
profile = "my-profile"
```

### 2. Provider Trait (`provider/mod.rs`)

**Purpose**: Define common interface for all LLM providers.

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a prompt to the LLM and receive a response
    async fn send(&self, prompt: &str) -> Result<String, LlmError>;
    
    /// Extract YAML code from the response
    fn extract_code(&self, response: &str) -> Result<String, LlmError>;
    
    /// Get the provider name for logging/debugging
    fn name(&self) -> &str;
}

pub enum LlmError {
    AuthenticationError(String),
    NetworkError(String),
    ModelNotAvailable(String),
    InvalidResponse(String),
    ConfigurationError(String),
}
```

### 3. Anthropic Provider (`provider/anthropic.rs`)

**Purpose**: Implement direct Anthropic API integration (existing functionality).

```rust
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, model: String) -> Result<Self, LlmError>;
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn send(&self, prompt: &str) -> Result<String, LlmError> {
        // Existing implementation from llm.rs
        // POST to https://api.anthropic.com/v1/messages
    }
    
    fn extract_code(&self, response: &str) -> Result<String, LlmError> {
        // Use llm-toolkit to extract YAML
    }
    
    fn name(&self) -> &str {
        "Anthropic"
    }
}
```

### 4. Bedrock Provider (`provider/bedrock.rs`)

**Purpose**: Implement AWS Bedrock integration using the Converse API.

```rust
use aws_sdk_bedrockruntime::{Client, types::{ContentBlock, ConversationRole, Message}};
use aws_config::BehaviorVersion;

pub struct BedrockProvider {
    client: Client,
    model_id: String,
}

impl BedrockProvider {
    pub async fn new(
        model_id: String,
        region: Option<String>,
        profile: Option<String>,
    ) -> Result<Self, LlmError> {
        // Build AWS config with optional region and profile
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest());
        
        if let Some(region) = region {
            config_loader = config_loader.region(
                aws_config::Region::new(region)
            );
        }
        
        if let Some(profile) = profile {
            config_loader = config_loader.profile_name(profile);
        }
        
        let sdk_config = config_loader.load().await;
        let client = Client::new(&sdk_config);
        
        Ok(Self { client, model_id })
    }
}

#[async_trait]
impl LlmProvider for BedrockProvider {
    async fn send(&self, prompt: &str) -> Result<String, LlmError> {
        let message = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(prompt.to_string()))
            .build()
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;
        
        let response = self.client
            .converse()
            .model_id(&self.model_id)
            .messages(message)
            .inference_config(|config| config
                .max_tokens(20000)
                .temperature(0.7)
            )
            .send()
            .await
            .map_err(|e| Self::map_bedrock_error(e))?;
        
        // Extract text from response
        let text = response
            .output()
            .ok_or(LlmError::InvalidResponse("No output".into()))?
            .as_message()
            .map_err(|_| LlmError::InvalidResponse("Output not a message".into()))?
            .content()
            .first()
            .ok_or(LlmError::InvalidResponse("No content".into()))?
            .as_text()
            .map_err(|_| LlmError::InvalidResponse("Content not text".into()))?
            .to_string();
        
        Ok(text)
    }
    
    fn extract_code(&self, response: &str) -> Result<String, LlmError> {
        llm_toolkit::extract_markdown_block_with_lang(response, "yaml")
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))
    }
    
    fn name(&self) -> &str {
        "AWS Bedrock"
    }
}

impl BedrockProvider {
    fn map_bedrock_error(error: SdkError<ConverseError>) -> LlmError {
        match error {
            SdkError::ServiceError(err) => match err.err() {
                ConverseError::ModelTimeoutException(_) => 
                    LlmError::NetworkError("Model timeout".into()),
                ConverseError::ModelNotReadyException(_) => 
                    LlmError::ModelNotAvailable("Model not ready".into()),
                ConverseError::AccessDeniedException(_) => 
                    LlmError::AuthenticationError("Access denied".into()),
                _ => LlmError::NetworkError(format!("{:?}", err)),
            },
            _ => LlmError::NetworkError(format!("{:?}", error)),
        }
    }
}
```

### 5. Refactored LLM Module (`llm.rs`)

**Purpose**: Orchestrate provider selection and usage.

```rust
use crate::config::{LlmConfig, LlmProvider as LlmProviderEnum};
use crate::provider::{LlmProvider, anthropic::AnthropicProvider, bedrock::BedrockProvider};

pub async fn scaffold(dir: &str) {
    let config = resolve_config().await;
    let provider = create_provider(&config).await
        .expect("Failed to create LLM provider");
    
    let desc = Text::new("Architecture Description:").prompt();
    let text = &desc.unwrap();
    
    println!("Asking {} for topology...", provider.name());
    let response = provider.send(&prompt(text)).await
        .expect("Failed to get LLM response");
    
    println!("Generating topology.yml...");
    let code = provider.extract_code(&response)
        .expect("Failed to extract code");
    
    let topo_file = format!("{}/topology.yml", dir);
    u::write_str(&topo_file, &code);
}

pub async fn send(text: &str) -> String {
    let config = resolve_config().await;
    let provider = create_provider(&config).await
        .expect("Failed to create LLM provider");
    
    provider.send(&prompt(text)).await
        .expect("Failed to get LLM response")
}

pub fn extract_code(response: &str) -> String {
    llm_toolkit::extract_markdown_block_with_lang(response, "yaml")
        .expect("Failed to extract code")
}

async fn resolve_config() -> LlmConfig {
    // Load .env file if it exists
    dotenv::dotenv().ok();
    
    // Resolve from environment (includes .env)
    let env_config = LlmConfig::from_env().unwrap_or_default();
    
    // Try to load from config file
    let file_config = LlmConfig::from_file("config.toml").ok();
    
    // CLI config would be passed from main.rs
    // For now, we only have env and file
    LlmConfig::merge(None, env_config, file_config)
}

async fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider {
        LlmProviderEnum::Bedrock => {
            let provider = BedrockProvider::new(
                config.model.clone(),
                config.aws_region.clone(),
                config.aws_profile.clone(),
            ).await?;
            Ok(Box::new(provider))
        }
        LlmProviderEnum::Anthropic => {
            let api_key = std::env::var("CLAUDE_API_KEY")
                .map_err(|_| LlmError::ConfigurationError(
                    "CLAUDE_API_KEY not set".into()
                ))?;
            let provider = AnthropicProvider::new(api_key, config.model.clone())?;
            Ok(Box::new(provider))
        }
    }
}

fn prompt(text: &str) -> String {
    // Existing prompt generation logic
    // ... (unchanged)
}
```

## Data Models

### Environment Variables

| Variable | Purpose | Example | Default |
|----------|---------|---------|---------|
| `TC_LLM_PROVIDER` | Select LLM provider | `bedrock` or `anthropic` | `bedrock` |
| `TC_LLM_MODEL` | Model identifier | `anthropic.claude-3-5-sonnet-20241022-v2:0` | Provider-specific default |
| `AWS_REGION` | AWS region for Bedrock | `us-east-1` | AWS SDK default |
| `AWS_PROFILE` | AWS profile name | `my-profile` | AWS SDK default |
| `CLAUDE_API_KEY` | Anthropic API key | `sk-ant-...` | None (required for Anthropic) |

### CLI Parameters

```rust
#[derive(Parser)]
pub struct ScaffoldArgs {
    /// LLM provider to use (bedrock or anthropic)
    #[arg(long, env = "TC_LLM_PROVIDER")]
    pub llm_provider: Option<String>,
    
    /// Model identifier
    #[arg(long, env = "TC_LLM_MODEL")]
    pub llm_model: Option<String>,
    
    /// AWS region for Bedrock
    #[arg(long, env = "AWS_REGION")]
    pub aws_region: Option<String>,
    
    /// AWS profile for Bedrock
    #[arg(long, env = "AWS_PROFILE")]
    pub aws_profile: Option<String>,
}
```

### Model Identifiers

**Bedrock Format**: `anthropic.claude-{version}-v{api-version}:{variant}`
- Example: `anthropic.claude-3-5-sonnet-20241022-v2:0`
- Example: `anthropic.claude-3-haiku-20240307-v1:0`

**Anthropic Format**: `claude-{version}-{date}`
- Example: `claude-sonnet-4-5-20250929`
- Example: `claude-3-5-sonnet-20241022`

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*


### Property Reflection

After analyzing all acceptance criteria, several redundancies were identified:

**Redundant Properties Identified:**
- Configuration precedence (2.1, 2.2, 3.1, 3.2, 4.1, 4.2, 5.1, 5.2) all test the same pattern for different fields → Combined into Property 1
- Provider-specific config ignored (4.5, 5.5) both test Anthropic ignoring AWS config → Combined into Property 6
- Default behaviors (1.1, 2.4) both test defaulting to Bedrock → Combined into one example test
- Error messages (7.5, 8.5) test the same missing API key scenario → Combined into one example test

**Property 1: Configuration precedence hierarchy**
*For any* configuration field (provider, model, region, profile), when values are specified at multiple levels (CLI, environment, config file), the value from the highest precedence source (CLI > Env > File) should be used in the final configuration.
**Validates: Requirements 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3**

**Property 2: Invalid provider rejection**
*For any* string that is not a valid provider name ("bedrock" or "anthropic"), attempting to parse it as a provider should result in an error that lists the valid options.
**Validates: Requirements 2.5**

**Property 3: Provider selection correctness**
*For any* valid provider enum value (Bedrock or Anthropic), creating a provider instance should return an implementation of the correct type that matches the enum.
**Validates: Requirements 6.2**

**Property 4: Provider error mapping**
*For any* provider-specific error (Bedrock SDK errors or Anthropic API errors), the error should be mapped to one of the common LlmError variants.
**Validates: Requirements 6.5**

**Property 5: AWS error propagation**
*For any* AWS Bedrock API error, the resulting error message should contain information from the original AWS error (code or message).
**Validates: Requirements 7.2**

**Property 6: Anthropic ignores AWS configuration**
*For any* AWS-specific configuration values (region, profile), when using the Anthropic provider, these values should not affect the provider's behavior or requests.
**Validates: Requirements 4.5, 5.5**

**Property 7: Environment variable loading from .env**
*For any* valid .env file containing key-value pairs, after loading the file, all variables should be accessible via std::env::var.
**Validates: Requirements 9.2**

**Property 8: Shell environment precedence over .env**
*For any* environment variable name, if the variable is set both in the shell environment and in a .env file, the shell environment value should be the one accessible after .env loading.
**Validates: Requirements 9.3**

## Error Handling

### Error Types

```rust
pub enum LlmError {
    /// Authentication failed (missing credentials, invalid credentials, access denied)
    AuthenticationError(String),
    
    /// Network or connectivity issues
    NetworkError(String),
    
    /// Requested model not available or not ready
    ModelNotAvailable(String),
    
    /// Response parsing or format issues
    InvalidResponse(String),
    
    /// Configuration issues (missing required config, invalid values)
    ConfigurationError(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::AuthenticationError(msg) => {
                write!(f, "Authentication failed: {}\n\
                    Hint: For Bedrock, ensure AWS credentials are configured via:\n\
                    - AWS SSO: aws sso login --profile <profile>\n\
                    - Environment: AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY\n\
                    - Config: ~/.aws/credentials\n\
                    For Anthropic, set CLAUDE_API_KEY environment variable.", msg)
            }
            LlmError::NetworkError(msg) => {
                write!(f, "Network error: {}\n\
                    Hint: Check your internet connection and try again.", msg)
            }
            LlmError::ModelNotAvailable(msg) => {
                write!(f, "Model not available: {}\n\
                    Hint: Verify the model ID is correct for your provider.\n\
                    Bedrock models: anthropic.claude-3-5-sonnet-20241022-v2:0\n\
                    Anthropic models: claude-sonnet-4-5-20250929", msg)
            }
            LlmError::InvalidResponse(msg) => {
                write!(f, "Invalid response: {}", msg)
            }
            LlmError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}\n\
                    Hint: Check your config.toml or environment variables.", msg)
            }
        }
    }
}
```

### Error Mapping

**Bedrock Errors → LlmError:**
- `AccessDeniedException` → `AuthenticationError`
- `ModelTimeoutException` → `NetworkError`
- `ModelNotReadyException` → `ModelNotAvailable`
- `ThrottlingException` → `NetworkError`
- Connection errors → `NetworkError`

**Anthropic Errors → LlmError:**
- 401 Unauthorized → `AuthenticationError`
- 404 Not Found → `ModelNotAvailable`
- 429 Rate Limit → `NetworkError`
- 5xx Server Error → `NetworkError`
- Connection errors → `NetworkError`

## Testing Strategy

### Unit Testing

Unit tests will verify:
- Configuration resolution logic with various combinations of CLI/env/file inputs
- Provider selection based on configuration
- Error type mapping from provider-specific errors to common errors
- Default value application when no configuration is provided
- .env file loading and precedence
- Model ID format validation

**Test Structure:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_config_precedence_cli_over_env() {
        // Test that CLI values override environment
    }
    
    #[test]
    fn test_config_precedence_env_over_file() {
        // Test that environment values override file
    }
    
    #[test]
    fn test_default_provider_is_bedrock() {
        // Test default when no config provided
    }
    
    #[test]
    fn test_invalid_provider_returns_error() {
        // Test error handling for invalid provider names
    }
    
    #[test]
    fn test_anthropic_ignores_aws_config() {
        // Test that AWS config doesn't affect Anthropic
    }
}
```

### Property-Based Testing

Property-based tests will verify universal properties across many inputs using the `proptest` crate:

**Property Test Requirements:**
- Each property-based test MUST run a minimum of 100 iterations
- Each test MUST be tagged with a comment referencing the design document property
- Tag format: `// Feature: aws-bedrock-integration, Property {number}: {property_text}`
- Each correctness property MUST be implemented by a SINGLE property-based test

**Property Tests:**

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    
    // Feature: aws-bedrock-integration, Property 1: Configuration precedence hierarchy
    proptest! {
        #[test]
        fn prop_config_precedence(
            cli_val in any::<Option<String>>(),
            env_val in any::<Option<String>>(),
            file_val in any::<Option<String>>()
        ) {
            // For any combination of CLI, env, and file values,
            // the merged config should use the highest precedence value
            let merged = merge_config_values(cli_val.clone(), env_val.clone(), file_val.clone());
            
            if cli_val.is_some() {
                assert_eq!(merged, cli_val);
            } else if env_val.is_some() {
                assert_eq!(merged, env_val);
            } else {
                assert_eq!(merged, file_val);
            }
        }
    }
    
    // Feature: aws-bedrock-integration, Property 2: Invalid provider rejection
    proptest! {
        #[test]
        fn prop_invalid_provider_error(
            invalid_name in "[a-z]{1,20}".prop_filter(
                "not a valid provider",
                |s| s != "bedrock" && s != "anthropic"
            )
        ) {
            // For any invalid provider string, parsing should fail with helpful error
            let result = LlmProvider::from_str(&invalid_name);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("bedrock") && err_msg.contains("anthropic"));
        }
    }
    
    // Feature: aws-bedrock-integration, Property 8: Shell environment precedence over .env
    proptest! {
        #[test]
        fn prop_shell_env_precedence(
            var_name in "[A-Z_]{3,20}",
            shell_value in ".*",
            dotenv_value in ".*".prop_filter("different from shell", |v| v != &shell_value)
        ) {
            // For any variable set in both shell and .env, shell value should win
            std::env::set_var(&var_name, &shell_value);
            // Simulate .env having different value
            // After loading, shell value should still be present
            assert_eq!(std::env::var(&var_name).unwrap(), shell_value);
        }
    }
}
```

### Integration Testing

Integration tests will verify:
- Actual AWS Bedrock API calls (requires AWS credentials)
- Actual Anthropic API calls (requires API key)
- End-to-end scaffolding workflow
- AWS SSO authentication flow
- Profile-based authentication

**Note:** Integration tests will be marked as `#[ignore]` by default and run separately in CI with proper credentials configured.

## Dependencies

### New Rust Crates

Add to `lib/scaffolder/Cargo.toml`:

```toml
[dependencies]
# Existing dependencies...
llm-toolkit = "0.21.0"
kit = { path = "../kit" }
compiler = { path = "../compiler" }
composer = { path = "../composer" }
visualizer = { path = "../visualizer" }

# New dependencies for AWS Bedrock
aws-config = "1.5"
aws-sdk-bedrockruntime = "1.50"
async-trait = "0.1"

# New dependencies for configuration
dotenv = "0.15"
toml = "0.8"

# Existing HTTP client (already used)
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }

[dev-dependencies]
proptest = "1.4"
mockall = "0.12"
```

### AWS SDK Version

Using AWS SDK for Rust v1.x (latest stable):
- `aws-config` for credential and region resolution
- `aws-sdk-bedrockruntime` for Bedrock API calls
- Supports all AWS credential sources (SSO, profiles, environment, instance metadata)

## Implementation Notes

### Default Model Selection

**Bedrock Default:** `anthropic.claude-3-5-sonnet-20241022-v2:0`
- Latest Claude 3.5 Sonnet available in Bedrock
- Good balance of capability and cost
- Widely available across AWS regions

**Anthropic Default:** `claude-sonnet-4-5-20250929`
- Latest Claude model from direct API
- Matches current implementation

### AWS Region Selection

When no region is specified:
1. Check `AWS_REGION` environment variable
2. Check `AWS_DEFAULT_REGION` environment variable
3. Check `~/.aws/config` for default region
4. Fall back to `us-east-1` (Bedrock is widely available there)

### Backward Compatibility

The existing `scaffold()` and `send()` functions maintain their signatures. Internal implementation changes are transparent to callers. The only breaking change is that Bedrock becomes the default instead of requiring `CLAUDE_API_KEY`.

### Migration Path

For users currently using Anthropic API:
1. Set `TC_LLM_PROVIDER=anthropic` environment variable, or
2. Add to `config.toml`:
   ```toml
   [llm]
   provider = "anthropic"
   ```
3. Keep existing `CLAUDE_API_KEY` in environment or `.env` file

## Security Considerations

1. **API Keys**: Never log or expose API keys in error messages
2. **AWS Credentials**: Rely on AWS SDK's secure credential handling
3. **.env Files**: Should be in `.gitignore` to prevent accidental commits
4. **Error Messages**: Sanitize error messages to avoid leaking sensitive information

## Performance Considerations

1. **AWS SDK Initialization**: Client initialization is async and may take 100-500ms for credential resolution
2. **Caching**: Consider caching the provider instance to avoid repeated initialization
3. **Timeouts**: Both providers should have reasonable timeouts (30-60 seconds for LLM calls)
4. **Retry Logic**: AWS SDK includes automatic retries; Anthropic client should implement similar logic

## Future Enhancements

Potential future additions (out of scope for this design):
1. Support for other Bedrock models (Meta Llama, Mistral, etc.)
2. Streaming responses for real-time feedback
3. Token usage tracking and cost estimation
4. Response caching to reduce API calls
5. Support for other LLM providers (OpenAI, Google, etc.)
6. Custom system prompts via configuration
