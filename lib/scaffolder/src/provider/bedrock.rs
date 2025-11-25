use crate::provider::{LlmError, LlmProvider};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_bedrockruntime::{
    error::SdkError,
    operation::converse::ConverseError,
    types::{ContentBlock, ConversationRole, Message},
    Client,
};

/// AWS Bedrock provider implementation using the Converse API
/// 
/// This provider communicates with AWS Bedrock using the AWS SDK,
/// supporting various authentication methods including SSO, profiles,
/// environment variables, and instance metadata.
#[derive(Debug, Clone)]
pub struct BedrockProvider {
    client: Client,
    model_id: String,
}

impl BedrockProvider {
    /// Create a new Bedrock provider instance
    /// 
    /// # Arguments
    /// * `model_id` - The Bedrock model identifier (e.g., "anthropic.claude-3-5-sonnet-20241022-v2:0")
    /// * `region` - Optional AWS region (if None, uses AWS SDK default resolution)
    /// * `profile` - Optional AWS profile name (if None, uses AWS SDK default resolution)
    /// 
    /// # Returns
    /// * `Ok(BedrockProvider)` - Successfully created provider with configured AWS client
    /// * `Err(LlmError)` - If AWS SDK initialization fails
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
        // Build message with user role and text content
        let message = Message::builder()
            .role(ConversationRole::User)
            .content(ContentBlock::Text(prompt.to_string()))
            .build()
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to build message: {}", e)))?;
        
        // Send request to Bedrock Converse API with inference parameters
        let response = self.client
            .converse()
            .model_id(&self.model_id)
            .messages(message)
            .inference_config(
                aws_sdk_bedrockruntime::types::InferenceConfiguration::builder()
                    .max_tokens(20000)
                    .temperature(0.7)
                    .build()
            )
            .send()
            .await
            .map_err(|e| Self::map_bedrock_error(e))?;
        
        // Extract text from response
        let text = response
            .output()
            .ok_or_else(|| LlmError::InvalidResponse("No output in response".to_string()))?
            .as_message()
            .map_err(|_| LlmError::InvalidResponse("Output is not a message".to_string()))?
            .content()
            .first()
            .ok_or_else(|| LlmError::InvalidResponse("No content in message".to_string()))?
            .as_text()
            .map_err(|_| LlmError::InvalidResponse("Content is not text".to_string()))?
            .to_string();
        
        Ok(text)
    }
    
    fn extract_code(&self, response: &str) -> Result<String, LlmError> {
        llm_toolkit::extract_markdown_block_with_lang(response, "yaml")
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to extract YAML: {}", e)))
    }
    
    fn name(&self) -> &str {
        "AWS Bedrock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bedrock_provider_creation() {
        // Test creating provider with no region or profile (uses defaults)
        let provider = BedrockProvider::new(
            "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            None,
            None,
        ).await;
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.model_id, "anthropic.claude-3-5-sonnet-20241022-v2:0");
        assert_eq!(provider.name(), "AWS Bedrock");
    }

    #[tokio::test]
    async fn test_bedrock_provider_with_region() {
        // Test creating provider with explicit region
        let provider = BedrockProvider::new(
            "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            Some("us-west-2".to_string()),
            None,
        ).await;
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.model_id, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    }

    #[tokio::test]
    async fn test_bedrock_provider_with_profile() {
        // Test creating provider with explicit profile
        let provider = BedrockProvider::new(
            "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            None,
            Some("test-profile".to_string()),
        ).await;
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.model_id, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    }

    #[tokio::test]
    async fn test_bedrock_provider_with_region_and_profile() {
        // Test creating provider with both region and profile
        let provider = BedrockProvider::new(
            "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            Some("eu-west-1".to_string()),
            Some("test-profile".to_string()),
        ).await;
        assert!(provider.is_ok());
        let provider = provider.unwrap();
        assert_eq!(provider.model_id, "anthropic.claude-3-5-sonnet-20241022-v2:0");
    }

    #[test]
    fn test_error_mapping_access_denied() {
        // Test that AccessDeniedException maps to AuthenticationError
        let err = ConverseError::AccessDeniedException(
            aws_sdk_bedrockruntime::types::error::AccessDeniedException::builder()
                .message("Access denied")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(403).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::AuthenticationError(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Access denied"));
    }

    #[test]
    fn test_error_mapping_model_timeout() {
        // Test that ModelTimeoutException maps to NetworkError
        let err = ConverseError::ModelTimeoutException(
            aws_sdk_bedrockruntime::types::error::ModelTimeoutException::builder()
                .message("Model timeout")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(408).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::NetworkError(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Model timeout"));
    }

    #[test]
    fn test_error_mapping_model_not_ready() {
        // Test that ModelNotReadyException maps to ModelNotAvailable
        let err = ConverseError::ModelNotReadyException(
            aws_sdk_bedrockruntime::types::error::ModelNotReadyException::builder()
                .message("Model not ready")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(503).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::ModelNotAvailable(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Model not ready"));
    }

    #[test]
    fn test_error_mapping_throttling() {
        // Test that ThrottlingException maps to NetworkError
        let err = ConverseError::ThrottlingException(
            aws_sdk_bedrockruntime::types::error::ThrottlingException::builder()
                .message("Throttled")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(429).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::NetworkError(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Throttled"));
    }

    #[test]
    fn test_error_mapping_validation() {
        // Test that ValidationException maps to InvalidResponse
        let err = ConverseError::ValidationException(
            aws_sdk_bedrockruntime::types::error::ValidationException::builder()
                .message("Validation failed")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(400).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::InvalidResponse(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Validation failed"));
    }

    #[test]
    fn test_error_mapping_resource_not_found() {
        // Test that ResourceNotFoundException maps to ModelNotAvailable
        let err = ConverseError::ResourceNotFoundException(
            aws_sdk_bedrockruntime::types::error::ResourceNotFoundException::builder()
                .message("Model not found")
                .build()
        );
        let response = aws_smithy_runtime_api::http::Response::new(
            aws_smithy_runtime_api::http::StatusCode::try_from(404).unwrap(),
            aws_smithy_types::body::SdkBody::empty()
        );
        let sdk_error = SdkError::service_error(err, response);
        let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
        
        assert!(matches!(llm_error, LlmError::ModelNotAvailable(_)));
        let error_msg = format!("{:?}", llm_error);
        assert!(error_msg.contains("Model not found"));
    }

    #[test]
    fn test_extract_code_with_yaml() {
        // Test that extract_code properly extracts YAML from markdown
        let provider = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(BedrockProvider::new(
                "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
                None,
                None,
            ))
            .unwrap();

        let response = r#"
Here's the topology:

```yaml
name: test-topology
functions:
  test-function:
    handler: test
```

That should work!
"#;

        let result = provider.extract_code(response);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("name: test-topology"));
        assert!(code.contains("test-function"));
    }

    #[test]
    fn test_extract_code_without_yaml() {
        // Test that extract_code fails when no YAML is present
        let provider = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(BedrockProvider::new(
                "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
                None,
                None,
            ))
            .unwrap();

        let response = "Here's some text without YAML";
        let result = provider.extract_code(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::InvalidResponse(_)));
    }

    #[test]
    fn test_model_id_formatting() {
        // Test that various model ID formats are accepted
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        // Bedrock format
        let provider = runtime.block_on(BedrockProvider::new(
            "anthropic.claude-3-5-sonnet-20241022-v2:0".to_string(),
            None,
            None,
        ));
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().model_id, "anthropic.claude-3-5-sonnet-20241022-v2:0");
        
        // Another Bedrock format
        let provider = runtime.block_on(BedrockProvider::new(
            "anthropic.claude-3-haiku-20240307-v1:0".to_string(),
            None,
            None,
        ));
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().model_id, "anthropic.claude-3-haiku-20240307-v1:0");
        
        // Custom model ID
        let provider = runtime.block_on(BedrockProvider::new(
            "custom-model-id".to_string(),
            None,
            None,
        ));
        assert!(provider.is_ok());
        assert_eq!(provider.unwrap().model_id, "custom-model-id");
    }
}

impl BedrockProvider {
    /// Map Bedrock SDK errors to common LlmError variants
    /// 
    /// This function provides clear error messages with resolution hints
    /// for common AWS Bedrock errors.
    fn map_bedrock_error(error: SdkError<ConverseError>) -> LlmError {
        match error {
            SdkError::ServiceError(err) => {
                let service_error = err.err();
                match service_error {
                    ConverseError::AccessDeniedException(e) => {
                        LlmError::AuthenticationError(format!(
                            "Access denied to Bedrock: {}. \
                            Ensure your AWS credentials have bedrock:InvokeModel permission.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    ConverseError::ModelTimeoutException(e) => {
                        LlmError::NetworkError(format!(
                            "Model timeout: {}. The request took too long to process.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    ConverseError::ModelNotReadyException(e) => {
                        LlmError::ModelNotAvailable(format!(
                            "Model not ready: {}. The model may still be loading.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    ConverseError::ThrottlingException(e) => {
                        LlmError::NetworkError(format!(
                            "Request throttled: {}. Too many requests, please retry later.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    ConverseError::ValidationException(e) => {
                        LlmError::InvalidResponse(format!(
                            "Validation error: {}. Check your request parameters.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    ConverseError::ResourceNotFoundException(e) => {
                        LlmError::ModelNotAvailable(format!(
                            "Model not found: {}. Verify the model ID is correct.",
                            e.message().unwrap_or("No details provided")
                        ))
                    }
                    _ => {
                        LlmError::NetworkError(format!(
                            "Bedrock service error: {:?}",
                            service_error
                        ))
                    }
                }
            }
            SdkError::DispatchFailure(e) => {
                LlmError::NetworkError(format!(
                    "Network dispatch failure: {:?}. Check your internet connection.",
                    e
                ))
            }
            SdkError::TimeoutError(_) => {
                LlmError::NetworkError(
                    "Request timeout. The request took too long to complete.".to_string()
                )
            }
            SdkError::ResponseError(e) => {
                LlmError::InvalidResponse(format!(
                    "Invalid response from Bedrock: {:?}",
                    e
                ))
            }
            _ => {
                LlmError::NetworkError(format!(
                    "AWS SDK error: {:?}",
                    error
                ))
            }
        }
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: aws-bedrock-integration, Property 5: AWS error propagation
    // Validates: Requirements 7.2
    // 
    // This property test verifies that AWS error messages are properly propagated
    // through our error mapping. We test that for any error message, when it's
    // wrapped in an AWS error type and mapped to LlmError, the original message
    // is preserved in the resulting error.
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_aws_error_message_propagation(
            error_message in "[a-zA-Z0-9 ]{10,50}",
        ) {
            // For any error message, when we create AWS error types with that message,
            // the resulting LlmError should contain the original message.
            
            // Test that AccessDeniedException messages are propagated
            {
                let err = ConverseError::AccessDeniedException(
                    aws_sdk_bedrockruntime::types::error::AccessDeniedException::builder()
                        .message(&error_message)
                        .build()
                );
                // Create a minimal response for the service error
                let response = aws_smithy_runtime_api::http::Response::new(
                    aws_smithy_runtime_api::http::StatusCode::try_from(403).unwrap(),
                    aws_smithy_types::body::SdkBody::empty()
                );
                let sdk_error = SdkError::service_error(err, response);
                let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
                
                // Verify the error message contains the original AWS error message
                let error_display = format!("{:?}", llm_error);
                prop_assert!(
                    error_display.contains(&error_message),
                    "Error message should contain original AWS error message"
                );
                prop_assert!(matches!(llm_error, LlmError::AuthenticationError(_)));
            }
            
            // Test that ModelTimeoutException messages are propagated
            {
                let err = ConverseError::ModelTimeoutException(
                    aws_sdk_bedrockruntime::types::error::ModelTimeoutException::builder()
                        .message(&error_message)
                        .build()
                );
                let response = aws_smithy_runtime_api::http::Response::new(
                    aws_smithy_runtime_api::http::StatusCode::try_from(408).unwrap(),
                    aws_smithy_types::body::SdkBody::empty()
                );
                let sdk_error = SdkError::service_error(err, response);
                let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
                
                let error_display = format!("{:?}", llm_error);
                prop_assert!(error_display.contains(&error_message));
                prop_assert!(matches!(llm_error, LlmError::NetworkError(_)));
            }
            
            // Test that ModelNotReadyException messages are propagated
            {
                let err = ConverseError::ModelNotReadyException(
                    aws_sdk_bedrockruntime::types::error::ModelNotReadyException::builder()
                        .message(&error_message)
                        .build()
                );
                let response = aws_smithy_runtime_api::http::Response::new(
                    aws_smithy_runtime_api::http::StatusCode::try_from(503).unwrap(),
                    aws_smithy_types::body::SdkBody::empty()
                );
                let sdk_error = SdkError::service_error(err, response);
                let llm_error = BedrockProvider::map_bedrock_error(sdk_error);
                
                let error_display = format!("{:?}", llm_error);
                prop_assert!(error_display.contains(&error_message));
                prop_assert!(matches!(llm_error, LlmError::ModelNotAvailable(_)));
            }
        }
    }
}
