use async_trait::async_trait;

pub mod anthropic;
pub mod bedrock;

/// Common error type for all LLM provider operations
#[derive(Debug, Clone, PartialEq)]
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

impl std::error::Error for LlmError {}

impl LlmError {
    /// Map a generic error description to an appropriate LlmError variant
    /// This is used by providers to convert their specific errors to common types
    pub fn from_error_description(error_type: &str, message: String) -> Self {
        match error_type.to_lowercase().as_str() {
            "auth" | "authentication" | "unauthorized" | "access_denied" | "accessdenied" => {
                LlmError::AuthenticationError(message)
            }
            "network" | "connection" | "timeout" | "throttling" => {
                LlmError::NetworkError(message)
            }
            "model" | "model_not_found" | "model_not_ready" | "modelnotready" => {
                LlmError::ModelNotAvailable(message)
            }
            "response" | "parse" | "invalid" => {
                LlmError::InvalidResponse(message)
            }
            "config" | "configuration" => {
                LlmError::ConfigurationError(message)
            }
            _ => LlmError::NetworkError(message)
        }
    }
}

/// Common trait for all LLM providers
/// 
/// This trait defines the interface that all LLM providers must implement,
/// allowing the scaffolder to work with different providers (Bedrock, Anthropic, etc.)
/// without knowing the specific implementation details.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Send a prompt to the LLM and receive a response
    /// 
    /// # Arguments
    /// * `prompt` - The text prompt to send to the LLM
    /// 
    /// # Returns
    /// * `Ok(String)` - The LLM's response text
    /// * `Err(LlmError)` - An error if the request fails
    async fn send(&self, prompt: &str) -> Result<String, LlmError>;
    
    /// Extract YAML code from the LLM response
    /// 
    /// # Arguments
    /// * `response` - The full response text from the LLM
    /// 
    /// # Returns
    /// * `Ok(String)` - The extracted YAML code
    /// * `Err(LlmError)` - An error if code extraction fails
    fn extract_code(&self, response: &str) -> Result<String, LlmError>;
    
    /// Get the provider name for logging/debugging
    /// 
    /// # Returns
    /// * A string identifying the provider (e.g., "AWS Bedrock", "Anthropic")
    fn name(&self) -> &str;
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_includes_hints() {
        let auth_error = LlmError::AuthenticationError("test".to_string());
        let display = format!("{}", auth_error);
        assert!(display.contains("Hint:"));
        assert!(display.contains("AWS SSO") || display.contains("CLAUDE_API_KEY"));

        let network_error = LlmError::NetworkError("test".to_string());
        let display = format!("{}", network_error);
        assert!(display.contains("Hint:"));
        assert!(display.contains("internet connection"));

        let model_error = LlmError::ModelNotAvailable("test".to_string());
        let display = format!("{}", model_error);
        assert!(display.contains("Hint:"));
        assert!(display.contains("model ID"));
    }

    #[test]
    fn test_error_mapping_authentication() {
        let error = LlmError::from_error_description("auth", "test message".to_string());
        assert!(matches!(error, LlmError::AuthenticationError(_)));

        let error = LlmError::from_error_description("unauthorized", "test".to_string());
        assert!(matches!(error, LlmError::AuthenticationError(_)));

        let error = LlmError::from_error_description("AccessDenied", "test".to_string());
        assert!(matches!(error, LlmError::AuthenticationError(_)));
    }

    #[test]
    fn test_error_mapping_network() {
        let error = LlmError::from_error_description("network", "test".to_string());
        assert!(matches!(error, LlmError::NetworkError(_)));

        let error = LlmError::from_error_description("timeout", "test".to_string());
        assert!(matches!(error, LlmError::NetworkError(_)));

        let error = LlmError::from_error_description("throttling", "test".to_string());
        assert!(matches!(error, LlmError::NetworkError(_)));
    }

    #[test]
    fn test_error_mapping_model() {
        let error = LlmError::from_error_description("model", "test".to_string());
        assert!(matches!(error, LlmError::ModelNotAvailable(_)));

        let error = LlmError::from_error_description("ModelNotReady", "test".to_string());
        assert!(matches!(error, LlmError::ModelNotAvailable(_)));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: aws-bedrock-integration, Property 4: Provider error mapping
    // Validates: Requirements 6.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_provider_error_mapping(
            error_type in prop_oneof![
                Just("auth"),
                Just("authentication"),
                Just("unauthorized"),
                Just("access_denied"),
                Just("AccessDenied"),
                Just("network"),
                Just("connection"),
                Just("timeout"),
                Just("throttling"),
                Just("model"),
                Just("model_not_found"),
                Just("model_not_ready"),
                Just("ModelNotReady"),
                Just("response"),
                Just("parse"),
                Just("invalid"),
                Just("config"),
                Just("configuration"),
            ],
            message in "[a-zA-Z0-9 ]{5,50}"
        ) {
            // For any provider-specific error type and message,
            // the error should be mapped to one of the common LlmError variants
            let error = LlmError::from_error_description(error_type, message.clone());
            
            // Verify the error is one of the expected variants
            let is_valid_variant = matches!(
                error,
                LlmError::AuthenticationError(_) |
                LlmError::NetworkError(_) |
                LlmError::ModelNotAvailable(_) |
                LlmError::InvalidResponse(_) |
                LlmError::ConfigurationError(_)
            );
            
            prop_assert!(is_valid_variant, "Error should be mapped to a valid LlmError variant");
            
            // Verify the message is preserved
            let error_message = match &error {
                LlmError::AuthenticationError(msg) => msg,
                LlmError::NetworkError(msg) => msg,
                LlmError::ModelNotAvailable(msg) => msg,
                LlmError::InvalidResponse(msg) => msg,
                LlmError::ConfigurationError(msg) => msg,
            };
            
            prop_assert_eq!(error_message, &message, "Error message should be preserved");
            
            // Verify specific mappings
            match error_type.to_lowercase().as_str() {
                "auth" | "authentication" | "unauthorized" | "access_denied" | "accessdenied" => {
                    prop_assert!(matches!(error, LlmError::AuthenticationError(_)));
                }
                "network" | "connection" | "timeout" | "throttling" => {
                    prop_assert!(matches!(error, LlmError::NetworkError(_)));
                }
                "model" | "model_not_found" | "model_not_ready" | "modelnotready" => {
                    prop_assert!(matches!(error, LlmError::ModelNotAvailable(_)));
                }
                "response" | "parse" | "invalid" => {
                    prop_assert!(matches!(error, LlmError::InvalidResponse(_)));
                }
                "config" | "configuration" => {
                    prop_assert!(matches!(error, LlmError::ConfigurationError(_)));
                }
                _ => {}
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_unknown_error_defaults_to_network(
            unknown_type in "[a-z]{3,15}".prop_filter(
                "not a known error type",
                |s| !matches!(
                    s.as_str(),
                    "auth" | "authentication" | "unauthorized" | "access_denied" | "accessdenied" |
                    "network" | "connection" | "timeout" | "throttling" |
                    "model" | "model_not_found" | "model_not_ready" | "modelnotready" |
                    "response" | "parse" | "invalid" |
                    "config" | "configuration"
                )
            ),
            message in "[a-zA-Z0-9 ]{5,50}"
        ) {
            // For any unknown error type, it should default to NetworkError
            let error = LlmError::from_error_description(&unknown_type, message.clone());
            
            prop_assert!(
                matches!(error, LlmError::NetworkError(_)),
                "Unknown error types should default to NetworkError"
            );
            
            if let LlmError::NetworkError(msg) = error {
                prop_assert_eq!(msg, message, "Error message should be preserved");
            }
        }
    }
}
