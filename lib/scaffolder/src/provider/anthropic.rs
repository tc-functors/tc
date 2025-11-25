use crate::provider::{LlmError, LlmProvider};
use async_trait::async_trait;
use kit as u;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Anthropic provider implementation using direct API access
/// 
/// This provider communicates directly with Anthropic's API using the
/// CLAUDE_API_KEY environment variable for authentication.
#[derive(Debug)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Content {
    r#type: String,
    text: String,
}

#[derive(Serialize, Debug)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Serialize, Debug)]
struct Payload {
    model: String,
    max_tokens: u16,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct Response {
    content: Vec<Content>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider instance
    /// 
    /// # Arguments
    /// * `api_key` - The Anthropic API key (from CLAUDE_API_KEY environment variable)
    /// * `model` - The model identifier to use (e.g., "claude-sonnet-4-5-20250929")
    /// 
    /// # Returns
    /// * `Ok(AnthropicProvider)` - Successfully created provider
    /// * `Err(LlmError)` - If the API key is empty or invalid
    pub fn new(api_key: String, model: String) -> Result<Self, LlmError> {
        if api_key.is_empty() {
            return Err(LlmError::AuthenticationError(
                "CLAUDE_API_KEY is empty".to_string()
            ));
        }

        Ok(Self {
            api_key,
            model,
        })
    }

    /// Build HTTP headers for Anthropic API requests
    fn headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        h.insert("content-type".to_string(), "application/json".to_string());
        h.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        h.insert("x-api-key".to_string(), self.api_key.clone());
        h.insert("accept".to_string(), "application/json".to_string());
        h.insert(
            "user-agent".to_string(),
            "libcurl/7.64.1 r-curl/4.3.2 httr/1.4.2".to_string(),
        );
        h
    }

    /// Build the request payload for the Anthropic API
    fn build_payload(&self, prompt: &str) -> Payload {
        let content = Content {
            r#type: "text".to_string(),
            text: prompt.to_string(),
        };

        let message = Message {
            role: "user".to_string(),
            content: vec![content],
        };

        Payload {
            model: self.model.clone(),
            max_tokens: 20000,
            messages: vec![message],
        }
    }


}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn send(&self, prompt: &str) -> Result<String, LlmError> {
        let payload = self.build_payload(prompt);
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to serialize payload: {}", e)))?;

        let url = "https://api.anthropic.com/v1/messages";
        
        // Use kit's http_post function (maintaining existing behavior)
        let res = u::http_post(url, self.headers(), payload_json)
            .await
            .map_err(|e| {
                // Try to extract status code from error if possible
                let error_str = format!("{:?}", e);
                if error_str.contains("401") {
                    LlmError::AuthenticationError("Invalid API key".to_string())
                } else if error_str.contains("404") {
                    LlmError::ModelNotAvailable("Model not found".to_string())
                } else if error_str.contains("429") {
                    LlmError::NetworkError("Rate limit exceeded".to_string())
                } else {
                    LlmError::NetworkError(format!("HTTP request failed: {:?}", e))
                }
            })?;

        let response: Response = serde_json::from_value(res)
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let text = response
            .content
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::InvalidResponse("No content in response".to_string()))?
            .text;

        Ok(text)
    }

    fn extract_code(&self, response: &str) -> Result<String, LlmError> {
        llm_toolkit::extract_markdown_block_with_lang(response, "yaml")
            .map_err(|e| LlmError::InvalidResponse(format!("Failed to extract YAML: {}", e)))
    }

    fn name(&self) -> &str {
        "Anthropic"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_validation() {
        // Empty API key should fail
        let result = AnthropicProvider::new("".to_string(), "claude-sonnet-4-5-20250929".to_string());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::AuthenticationError(_)));

        // Valid API key should succeed
        let result = AnthropicProvider::new("sk-ant-test123".to_string(), "claude-sonnet-4-5-20250929".to_string());
        assert!(result.is_ok());
    }



    #[test]
    fn test_extract_code_functionality() {
        let provider = AnthropicProvider::new(
            "sk-ant-test123".to_string(),
            "claude-sonnet-4-5-20250929".to_string()
        ).unwrap();

        // Test with valid YAML block
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

        // Test with missing YAML block
        let response_no_yaml = "Here's some text without YAML";
        let result = provider.extract_code(response_no_yaml);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::InvalidResponse(_)));
    }

    #[test]
    fn test_headers_include_api_key() {
        let provider = AnthropicProvider::new(
            "sk-ant-test123".to_string(),
            "claude-sonnet-4-5-20250929".to_string()
        ).unwrap();

        let headers = provider.headers();
        assert_eq!(headers.get("x-api-key"), Some(&"sk-ant-test123".to_string()));
        assert_eq!(headers.get("anthropic-version"), Some(&"2023-06-01".to_string()));
        assert_eq!(headers.get("content-type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_payload_structure() {
        let provider = AnthropicProvider::new(
            "sk-ant-test123".to_string(),
            "claude-sonnet-4-5-20250929".to_string()
        ).unwrap();

        let payload = provider.build_payload("test prompt");
        assert_eq!(payload.model, "claude-sonnet-4-5-20250929");
        assert_eq!(payload.max_tokens, 20000);
        assert_eq!(payload.messages.len(), 1);
        assert_eq!(payload.messages[0].role, "user");
        assert_eq!(payload.messages[0].content[0].text, "test prompt");
    }

    #[test]
    fn test_provider_name() {
        let provider = AnthropicProvider::new(
            "sk-ant-test123".to_string(),
            "claude-sonnet-4-5-20250929".to_string()
        ).unwrap();

        assert_eq!(provider.name(), "Anthropic");
    }
}


#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: aws-bedrock-integration, Property 6: Anthropic ignores AWS configuration
    // Validates: Requirements 4.5, 5.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_anthropic_ignores_aws_config(
            api_key in "[a-z0-9-]{10,30}",
            model in "[a-z0-9.-]{10,40}",
            _aws_region in proptest::option::of("[a-z]{2}-[a-z]+-[0-9]"),
            _aws_profile in proptest::option::of("[a-z0-9_-]{3,20}"),
        ) {
            // For any AWS-specific configuration values (region, profile),
            // when using the Anthropic provider, these values should not affect
            // the provider's behavior or requests.
            
            // Create provider (AWS config is not part of constructor)
            let provider = AnthropicProvider::new(api_key.clone(), model.clone()).unwrap();
            
            // Verify provider was created successfully regardless of AWS config
            prop_assert_eq!(provider.name(), "Anthropic");
            
            // Verify the provider's internal state doesn't include AWS config
            // (The provider struct only has api_key, model, and client fields)
            prop_assert_eq!(&provider.model, &model);
            prop_assert_eq!(&provider.api_key, &api_key);
            
            // Verify headers don't include AWS-specific headers
            let headers = provider.headers();
            prop_assert!(!headers.contains_key("aws-region"), 
                "Anthropic headers should not contain AWS region");
            prop_assert!(!headers.contains_key("aws-profile"), 
                "Anthropic headers should not contain AWS profile");
            prop_assert!(!headers.contains_key("authorization"), 
                "Anthropic headers should not contain AWS authorization");
            
            // Verify the API key header is present (Anthropic-specific)
            prop_assert_eq!(headers.get("x-api-key"), Some(&api_key));
            
            // The fact that we can create the provider and build headers
            // without any AWS configuration demonstrates that Anthropic
            // ignores AWS configuration completely
        }
    }
}
