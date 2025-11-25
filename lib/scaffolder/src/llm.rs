use inquire::Text;
use kit as u;
use kit::*;

use crate::config::{LlmConfig, LlmProvider as LlmProviderEnum};
use crate::provider::{LlmProvider, LlmError, anthropic::AnthropicProvider, bedrock::BedrockProvider};

/// Default model ID for AWS Bedrock provider
/// Uses Claude Opus 4.5 which provides the highest capability
pub const DEFAULT_BEDROCK_MODEL: &str = "anthropic.claude-opus-4-5-20251101-v1:0";

/// Default model ID for Anthropic provider
/// Uses the latest Claude Sonnet model from the direct API
pub const DEFAULT_ANTHROPIC_MODEL: &str = "claude-sonnet-4-5-20250929";

fn prompt(text: &str) -> String {
    let lines = v![
        "You are an expert at creating tc topologies. tc is a graph-based, serverless application composer that uses high-level abstractions called Cloud Functors to define application architecture without infrastructure details.",
        "",
        "Here is the application you need to create a tc topology for:",
        "",
        "<application_description>",
        format!("{text}"),
        "</application_description>",
        "",
        "## Your Task",
        "Create a complete, production-ready tc topology YAML file for the described application. Focus on business logic and relationships, not infrastructure implementation details.",
        "",
        "## Core Principles",
        "- **Provider-agnostic**: Definitions work across cloud providers",
        "- **Composable**: Functions chain together naturally  ",
        "- **Namespaced**: All entities are isolated within their topology",
        "- **Stateless**: No external state management required",
        "- **Business-focused**: Abstract away infrastructure complexity",
        "",
        "## Available Entities",
        "",
        "### Routes",
        "HTTP endpoints that trigger functions:",
        "```yaml",
        "routes:",
        "  /api/endpoint:",
        "    method: POST|GET|PUT|DELETE|PATCH",
        "    function: function-name",
        "```",
        "",
        "### Functions",
        "Serverless compute units that can chain together:",
        "```yaml",
        "functions:",
        "  function-name:",
        "    function: next-function    # Chain to another function",
        "    page: page-name         # Webapp",
        "    event: event-name         # Trigger an event",
        "    queue: queue-name         # Send to queue",
        "    channel: channel-name     # Send to WebSocket",
        "```",
        "",
        "### Events",
        "Asynchronous event notifications:",
        "```yaml",
        "events:",
        "  EventName:",
        "    function: handler-function",
        "    queue: queue-name",
        "    channel: channel-name",
        "```",
        "",
        "### Queues",
        "Message queues for async processing:",
        "```yaml",
        "queues:",
        "  queue-name:",
        "    function: processor-function",
        "    batch_size: 10",
        "```",
        "",
        "### Channels",
        "WebSocket connections for real-time communication:",
        "```yaml",
        "channels:",
        "  channel-name:",
        "    type: websocket",
        "    function: handler-function",
        "```",
        "",
        "",
        "## Design Process",
        "",
        "1. **Analyze the application description** to understand the core use case and requirements",
        "2. **Identify user interactions** and map them to HTTP routes",
        "3. **Design the data flow** through chained functions",
        "4. **Determine state requirements** for data persistence",
        "5. **Add asynchronous processing** using events and queues where appropriate",
        "6. **Include real-time features** using channels if needed",
        "7. **Ensure proper composition** with logical function chaining",
        "",
        "## Naming Conventions",
        "- Use kebab-case for all entity names",
        "- Functions: action-based (`validate-input`, `process-payment`, `send-notification`)",
        "- Events: past tense (`OrderCreated`, `PaymentProcessed`, `UserRegistered`)",
        "- Queues: purpose-based (`processing-queue`, `email-queue`, `retry-queue`)",
        "",
        "## Design Patterns to Consider",
        "",
        "**Function Chaining**: For sequential operations",
        "```yaml",
        "functions:",
        "  validate-input:",
        "    function: process-data",
        "  process-data:",
        "    function: save-results",
        "```",
        "",
        "**Event-Driven**: For fire-and-forget operations",
        "```yaml",
        "functions:",
        "  main-handler:",
        "    event: DataProcessed",
        "events:",
        "  DataProcessed:",
        "    function: notification-handler",
        "```",
        "",
        "**Async Processing**: For heavy or batch operations",
        "```yaml",
        "functions:",
        "  api-handler:",
        "    queue: processing-queue",
        "queues:",
        "  processing-queue:",
        "    function: worker",
        "```",
        "",
        "**Real-Time Updates**: For user notifications",
        "```yaml",
        "functions:",
        "  update-handler:",
        "    channel: live-updates",
        "channels:",
        "  live-updates:",
        "    handler: default",
        "```",
        "",
        "## Validation Requirements",
        "- All referenced entities must be defined",
        "- Routes must have valid HTTP methods",
        "- Function chains must be logical and non-circular",
        "",
        "## Output Requirements",
        "",
        "Provide your response in the following format:",
        "",
        "1. **Brief architecture explanation** (2-3 sentences describing the overall design approach)",
        "",
        "2. **Complete topology YAML** inside <topology> tags with:",
        "   - Descriptive topology name in kebab-case",
        "   - All necessary entities properly defined",
        "   - Logical composition and flow",
        "   - Inline comments for key design decisions",
        "",
        "3. **Key design decisions** (bullet points explaining major architectural choices)",
        "",
        "## Example Structure",
        "```yaml",
        "name: descriptive-topology-name",
        "",
        "routes:",
        "  # HTTP endpoints",
        "",
        "functions:",
        "  # Business logic functions with chaining",
        "",
        "events:",
        "  # Async event definitions",
        "",
        "queues:",
        "  # Background processing queues",
        "",
        "channels:",
        "  # Real-time WebSocket channels",
        "",
        "Remember: Focus on business logic and relationships, not infrastructure details. Create a topology that is composable, maintainable, and follows tc best practices"
    ];
    lines.join("\n")
}

/// Resolve LLM configuration from CLI arguments, environment, and config file
/// Loads .env file if present, then merges CLI, env, and file configs
async fn resolve_config(
    cli_provider: Option<String>,
    cli_model: Option<String>,
    cli_region: Option<String>,
    cli_profile: Option<String>,
) -> Result<LlmConfig, crate::config::ConfigError> {
    use crate::config::LlmProvider as LlmProviderEnum;
    use std::str::FromStr;
    
    // Resolve from environment (includes .env loading)
    // Note: LlmConfig::from_env() already calls dotenv::dotenv().ok()
    let env_config = LlmConfig::from_env().unwrap_or_default();
    
    // Try to load from config file
    let file_config = LlmConfig::from_file("config.toml").ok();
    
    // Build CLI config only if CLI arguments were provided
    // We need to be careful here: only include fields that were actually provided via CLI
    // Otherwise, None from CLI will override actual values from env/file
    let cli_config = if cli_provider.is_some() || cli_model.is_some() || cli_region.is_some() || cli_profile.is_some() {
        // Start with env config as base, then override with CLI values
        let mut config = env_config.clone();
        
        if let Some(p) = cli_provider {
            config.provider = LlmProviderEnum::from_str(&p)?;
        }
        
        if cli_model.is_some() {
            config.model = cli_model;
        }
        
        if cli_region.is_some() {
            config.aws_region = cli_region;
        }
        
        if cli_profile.is_some() {
            config.aws_profile = cli_profile;
        }
        
        Some(config)
    } else {
        None
    };
    
    Ok(LlmConfig::merge(cli_config, env_config, file_config))
}

/// Create the appropriate provider based on configuration
async fn create_provider(config: &LlmConfig) -> Result<Box<dyn LlmProvider>, LlmError> {
    match config.provider {
        LlmProviderEnum::Bedrock => {
            // Use configured model or default for Bedrock
            let model = config.model.clone()
                .unwrap_or_else(|| DEFAULT_BEDROCK_MODEL.to_string());
            
            let provider = BedrockProvider::new(
                model,
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
            
            // Use configured model or default for Anthropic
            let model = config.model.clone()
                .unwrap_or_else(|| DEFAULT_ANTHROPIC_MODEL.to_string());
            
            let provider = AnthropicProvider::new(api_key, model)?;
            Ok(Box::new(provider))
        }
    }
}

pub async fn send(text: &str) -> String {
    let config = resolve_config(None, None, None, None).await
        .expect("Failed to resolve configuration");
    let provider = create_provider(&config).await
        .expect("Failed to create LLM provider");
    
    provider.send(&prompt(text)).await
        .expect("Failed to get LLM response")
}

pub fn extract_code(response: &str) -> String {
    let code = llm_toolkit::extract_markdown_block_with_lang(&response, "yaml").unwrap();
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_provider_returns_bedrock_for_bedrock_enum() {
        let config = LlmConfig {
            provider: LlmProviderEnum::Bedrock,
            model: Some("test-model".to_string()),
            aws_region: Some("us-east-1".to_string()),
            aws_profile: None,
        };

        let result = create_provider(&config).await;
        assert!(result.is_ok(), "Should create Bedrock provider successfully");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "AWS Bedrock", "Should return AWS Bedrock provider");
    }

    #[tokio::test]
    async fn test_create_provider_returns_anthropic_for_anthropic_enum() {
        // Set up API key for Anthropic (must be non-empty)
        unsafe {
            std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-key-12345");
        }

        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: Some("test-model".to_string()),
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;

        assert!(result.is_ok(), "Should create Anthropic provider successfully");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "Anthropic", "Should return Anthropic provider");
    }

    #[tokio::test]
    async fn test_create_provider_error_when_anthropic_api_key_missing() {
        // Ensure API key is not set for this test
        // Save current value if it exists
        let saved_key = std::env::var("CLAUDE_API_KEY").ok();
        unsafe {
            std::env::remove_var("CLAUDE_API_KEY");
        }

        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: Some("test-model".to_string()),
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;
        
        assert!(result.is_err(), "Should fail when CLAUDE_API_KEY is missing");
        
        if let Err(LlmError::ConfigurationError(msg)) = result {
            assert!(msg.contains("CLAUDE_API_KEY"), 
                "Error message should mention CLAUDE_API_KEY");
        } else {
            panic!("Expected ConfigurationError");
        }
        
        // Restore the saved key if it existed
        if let Some(key) = saved_key {
            unsafe {
                std::env::set_var("CLAUDE_API_KEY", key);
            }
        }
    }

    #[tokio::test]
    async fn test_create_provider_uses_default_model_for_bedrock() {
        let config = LlmConfig {
            provider: LlmProviderEnum::Bedrock,
            model: None, // No model specified
            aws_region: Some("us-east-1".to_string()),
            aws_profile: None,
        };

        let result = create_provider(&config).await;
        assert!(result.is_ok(), "Should create Bedrock provider with default model");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "AWS Bedrock");
    }

    #[tokio::test]
    async fn test_create_provider_uses_default_model_for_anthropic() {
        // Set up API key for Anthropic (must be non-empty)
        unsafe {
            std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-key-12345");
        }

        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: None, // No model specified
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;

        assert!(result.is_ok(), "Should create Anthropic provider with default model");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[test]
    fn test_bedrock_default_model_constant() {
        // Test that the Bedrock default model constant has the expected value
        assert_eq!(
            DEFAULT_BEDROCK_MODEL,
            "anthropic.claude-opus-4-5-20251101-v1:0",
            "Bedrock default model should be Claude Opus 4.5"
        );
    }

    #[test]
    fn test_anthropic_default_model_constant() {
        // Test that the Anthropic default model constant has the expected value
        assert_eq!(
            DEFAULT_ANTHROPIC_MODEL,
            "claude-sonnet-4-5-20250929",
            "Anthropic default model should be Claude Sonnet 4.5"
        );
    }

    #[tokio::test]
    async fn test_bedrock_uses_configured_model_over_default() {
        let custom_model = "anthropic.claude-3-haiku-20240307-v1:0";
        let config = LlmConfig {
            provider: LlmProviderEnum::Bedrock,
            model: Some(custom_model.to_string()),
            aws_region: Some("us-east-1".to_string()),
            aws_profile: None,
        };

        let result = create_provider(&config).await;
        assert!(result.is_ok(), "Should create Bedrock provider with custom model");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "AWS Bedrock");
        // The provider should be created with the custom model, not the default
    }

    #[tokio::test]
    async fn test_anthropic_uses_configured_model_over_default() {
        // Set up API key for Anthropic
        unsafe {
            std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-key-12345");
        }

        let custom_model = "claude-3-5-sonnet-20241022";
        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: Some(custom_model.to_string()),
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;

        assert!(result.is_ok(), "Should create Anthropic provider with custom model");
        
        let provider = result.unwrap();
        assert_eq!(provider.name(), "Anthropic");
        // The provider should be created with the custom model, not the default
    }

    #[tokio::test]
    async fn test_bedrock_default_model_applied_when_none() {
        // Test that when model is None, the default is applied
        let config = LlmConfig {
            provider: LlmProviderEnum::Bedrock,
            model: None,
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;
        assert!(result.is_ok(), "Should create Bedrock provider with default model");
        
        // Verify the provider was created successfully
        let provider = result.unwrap();
        assert_eq!(provider.name(), "AWS Bedrock");
    }

    #[tokio::test]
    async fn test_anthropic_default_model_applied_when_none() {
        // Set up API key for Anthropic
        unsafe {
            std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-key-12345");
        }

        // Test that when model is None, the default is applied
        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: None,
            aws_region: None,
            aws_profile: None,
        };

        let result = create_provider(&config).await;

        assert!(result.is_ok(), "Should create Anthropic provider with default model");
        
        // Verify the provider was created successfully
        let provider = result.unwrap();
        assert_eq!(provider.name(), "Anthropic");
    }

    #[tokio::test]
    async fn test_cli_provider_overrides_env() {
        // Clean up first
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("AWS_PROFILE");
        }

        // Set environment variable
        unsafe {
            std::env::set_var("TC_LLM_PROVIDER", "anthropic");
        }

        // CLI specifies bedrock
        let config = resolve_config(
            Some("bedrock".to_string()),
            None,
            None,
            None,
        ).await.unwrap();

        // Clean up
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
        }

        assert_eq!(config.provider, LlmProviderEnum::Bedrock, 
            "CLI provider should override environment variable");
    }

    #[tokio::test]
    async fn test_cli_model_overrides_env() {
        // Clean up first
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("AWS_PROFILE");
        }

        // Set environment variable
        unsafe {
            std::env::set_var("TC_LLM_MODEL", "env-model");
        }

        // CLI specifies different model
        let config = resolve_config(
            None,
            Some("cli-model".to_string()),
            None,
            None,
        ).await.unwrap();

        // Clean up
        unsafe {
            std::env::remove_var("TC_LLM_MODEL");
        }

        assert_eq!(config.model, Some("cli-model".to_string()), 
            "CLI model should override environment variable");
    }

    #[tokio::test]
    async fn test_cli_region_overrides_env() {
        // Clean up first
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("AWS_PROFILE");
        }

        // Set environment variable
        unsafe {
            std::env::set_var("AWS_REGION", "us-west-2");
        }

        // CLI specifies different region
        let config = resolve_config(
            None,
            None,
            Some("us-east-1".to_string()),
            None,
        ).await.unwrap();

        // Clean up
        unsafe {
            std::env::remove_var("AWS_REGION");
        }

        assert_eq!(config.aws_region, Some("us-east-1".to_string()), 
            "CLI region should override environment variable");
    }

    #[tokio::test]
    async fn test_cli_profile_overrides_env() {
        // Clean up first
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("AWS_PROFILE");
        }

        // Set environment variable
        unsafe {
            std::env::set_var("AWS_PROFILE", "env-profile");
        }

        // CLI specifies different profile
        let config = resolve_config(
            None,
            None,
            None,
            Some("cli-profile".to_string()),
        ).await.unwrap();

        // Clean up
        unsafe {
            std::env::remove_var("AWS_PROFILE");
        }

        assert_eq!(config.aws_profile, Some("cli-profile".to_string()), 
            "CLI profile should override environment variable");
    }

    #[tokio::test]
    async fn test_cli_arguments_override_file_config() {
        // Create a temporary config file
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_config_cli_override.toml");
        
        let config_content = r#"
[llm]
provider = "anthropic"
model = "file-model"

[llm.aws]
region = "us-west-2"
profile = "file-profile"
"#;
        std::fs::write(&config_path, config_content).unwrap();

        // Change to temp directory so config file is found
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        // Rename to config.toml so it's found by resolve_config
        let config_toml_path = temp_dir.join("config.toml");
        std::fs::copy(&config_path, &config_toml_path).unwrap();

        // CLI specifies different values
        let config = resolve_config(
            Some("bedrock".to_string()),
            Some("cli-model".to_string()),
            Some("us-east-1".to_string()),
            Some("cli-profile".to_string()),
        ).await.unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // Clean up
        std::fs::remove_file(&config_path).ok();
        std::fs::remove_file(&config_toml_path).ok();

        assert_eq!(config.provider, LlmProviderEnum::Bedrock, 
            "CLI provider should override file config");
        assert_eq!(config.model, Some("cli-model".to_string()), 
            "CLI model should override file config");
        assert_eq!(config.aws_region, Some("us-east-1".to_string()), 
            "CLI region should override file config");
        assert_eq!(config.aws_profile, Some("cli-profile".to_string()), 
            "CLI profile should override file config");
    }

    #[tokio::test]
    async fn test_partial_cli_arguments() {
        // Clean up any existing env vars first
        unsafe {
            std::env::remove_var("TC_LLM_PROVIDER");
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
            std::env::remove_var("AWS_PROFILE");
        }

        // Set environment variables
        unsafe {
            std::env::set_var("TC_LLM_MODEL", "env-model");
            std::env::set_var("AWS_REGION", "us-west-2");
        }

        // CLI only specifies provider and profile
        let config = resolve_config(
            Some("bedrock".to_string()),
            None,
            None,
            Some("cli-profile".to_string()),
        ).await.unwrap();

        // Clean up
        unsafe {
            std::env::remove_var("TC_LLM_MODEL");
            std::env::remove_var("AWS_REGION");
        }

        // CLI values should be used where provided
        assert_eq!(config.provider, LlmProviderEnum::Bedrock, 
            "CLI provider should be used");
        assert_eq!(config.aws_profile, Some("cli-profile".to_string()), 
            "CLI profile should be used");
        
        // Env values should be used where CLI didn't provide
        assert_eq!(config.model, Some("env-model".to_string()), 
            "Env model should be used when CLI doesn't provide");
        assert_eq!(config.aws_region, Some("us-west-2".to_string()), 
            "Env region should be used when CLI doesn't provide");
    }

    #[tokio::test]
    async fn test_invalid_cli_provider_returns_error() {
        let result = resolve_config(
            Some("invalid-provider".to_string()),
            None,
            None,
            None,
        ).await;

        assert!(result.is_err(), "Should return error for invalid provider");
        
        if let Err(crate::config::ConfigError::InvalidProvider(msg)) = result {
            assert!(msg.contains("bedrock"), "Error should mention valid providers");
            assert!(msg.contains("anthropic"), "Error should mention valid providers");
        } else {
            panic!("Expected InvalidProvider error");
        }
    }
}

pub async fn scaffold(
    dir: &str,
    cli_provider: Option<String>,
    cli_model: Option<String>,
    cli_region: Option<String>,
    cli_profile: Option<String>,
) {
    let config = resolve_config(cli_provider, cli_model, cli_region, cli_profile).await
        .expect("Failed to resolve configuration");
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


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: aws-bedrock-integration, Property 3: Provider selection correctness
    // Validates: Requirements 6.2
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_provider_selection_correctness(
            provider_enum in prop_oneof![
                Just(LlmProviderEnum::Bedrock),
                Just(LlmProviderEnum::Anthropic),
            ]
        ) {
            // For any valid provider enum value, creating a provider instance
            // should return an implementation of the correct type
            
            // Set up environment for Anthropic (always set it to avoid race conditions)
            unsafe {
                std::env::set_var("CLAUDE_API_KEY", "sk-ant-test-key-for-property-test-12345");
            }
            
            let config = LlmConfig {
                provider: provider_enum,
                model: Some("test-model".to_string()),
                aws_region: Some("us-east-1".to_string()),
                aws_profile: None,
            };
            
            // Create provider using tokio runtime
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(create_provider(&config));
            
            // Verify provider was created successfully
            if let Err(ref e) = result {
                eprintln!("Error creating provider: {:?}", e);
            }
            prop_assert!(result.is_ok(), "Provider creation should succeed for valid enum");
            
            let provider = result.unwrap();
            
            // Verify the provider name matches the enum
            match provider_enum {
                LlmProviderEnum::Bedrock => {
                    prop_assert_eq!(provider.name(), "AWS Bedrock", 
                        "Bedrock enum should create AWS Bedrock provider");
                }
                LlmProviderEnum::Anthropic => {
                    prop_assert_eq!(provider.name(), "Anthropic", 
                        "Anthropic enum should create Anthropic provider");
                }
            }
            
            // Note: We don't clean up CLAUDE_API_KEY here because:
            // 1. Property tests run multiple times and may run in parallel
            // 2. Other tests may need this environment variable
            // 3. The test environment is isolated anyway
        }
    }
}
