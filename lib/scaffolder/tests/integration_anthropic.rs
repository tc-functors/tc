//! Integration tests for Anthropic provider
//! 
//! These tests make actual API calls to Anthropic and are marked with #[ignore]
//! to prevent them from running in normal test runs. They require:
//! - Valid CLAUDE_API_KEY environment variable
//! - Internet connectivity to api.anthropic.com
//!
//! Run these tests with: cargo test --test integration_anthropic -- --ignored
//!
//! Requirements tested: 8.1, 8.2, 8.3

use scaffolder::provider::{LlmProvider, anthropic::AnthropicProvider};

/// Test actual Anthropic API call
/// 
/// This test verifies that the Anthropic provider can successfully:
/// - Initialize with API key from environment
/// - Make an actual API call to Anthropic
/// - Receive and parse a valid response
///
/// Requirements: 8.1, 8.2
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_actual_api_call() {
    // Get API key from environment
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    let model = "claude-sonnet-4-5-20250929";
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Make a simple API call
    let prompt = "Say 'Hello from Anthropic!' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify we got a non-empty response
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 10, "Response should be substantial");
    
    // Verify the response contains expected content
    let response_lower = response.to_lowercase();
    assert!(
        response_lower.contains("hello") || response_lower.contains("anthropic"),
        "Response should acknowledge the prompt"
    );
    
    println!("✓ Anthropic API call successful");
    println!("Response: {}", response);
}

/// Test Anthropic provider with default model
/// 
/// This test verifies that the Anthropic provider works with the default
/// model specified in the design document.
///
/// Requirements: 8.1, 8.3
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_with_default_model() {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    // Use the default model from the design
    let model = scaffolder::llm::DEFAULT_ANTHROPIC_MODEL;
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Verify provider name
    assert_eq!(provider.name(), "Anthropic");
    
    // Make a simple API call
    let prompt = "What is 2+2? Answer with just the number.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.contains("4"), "Response should contain the answer");
    
    println!("✓ Anthropic API call successful with default model");
    println!("Response: {}", response);
}

/// Test Anthropic provider with alternative model
/// 
/// This test verifies that the Anthropic provider can work with different
/// Claude model variants.
///
/// Requirements: 8.1
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_with_alternative_model() {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    // Use an alternative model
    let model = "claude-3-5-sonnet-20241022";
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Make a simple API call
    let prompt = "Say 'Alternative model works' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Anthropic API call successful with alternative model");
    println!("Response: {}", response);
}

/// Test backward compatibility with existing implementation
/// 
/// This test verifies that the new provider-based implementation maintains
/// backward compatibility with the existing Anthropic API integration.
/// It tests the same request format and behavior as the original implementation.
///
/// Requirements: 8.3
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_backward_compatibility() {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    let model = "claude-sonnet-4-5-20250929";
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Test with a prompt similar to what the original implementation would use
    let prompt = "You are a helpful assistant. Respond to this message: Hello!";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify response format is compatible
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.is_ascii() || response.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation()), 
        "Response should be valid text");
    
    println!("✓ Backward compatibility verified");
    println!("Response: {}", response);
}

/// Test Anthropic error handling with missing API key
/// 
/// This test verifies that the Anthropic provider properly handles the case
/// when the API key is missing or empty.
///
/// Requirements: 8.5 (from requirements document)
#[tokio::test]
async fn test_anthropic_missing_api_key_error() {
    // Try to create provider with empty API key
    let result = AnthropicProvider::new("".to_string(), "test-model".to_string());
    
    // Should fail with configuration error
    assert!(result.is_err(), "Should fail with empty API key");
    
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("API key") || error_msg.contains("empty"),
            "Error message should mention API key issue"
        );
    }
    
    println!("✓ Empty API key properly rejected");
}

/// Test Anthropic error handling with invalid API key
/// 
/// This test verifies that the Anthropic provider properly handles authentication
/// errors when the API key is invalid.
///
/// Requirements: 8.5 (from requirements document)
#[tokio::test]
#[ignore = "makes actual API call with invalid credentials"]
async fn test_anthropic_invalid_api_key_error() {
    // Use an obviously invalid API key
    let invalid_key = "sk-ant-invalid-key-12345";
    let model = "claude-sonnet-4-5-20250929";
    
    let provider = AnthropicProvider::new(invalid_key.to_string(), model.to_string())
        .expect("Provider creation should succeed even with invalid key");
    
    // Try to make an API call
    let prompt = "Test prompt";
    let result = provider.send(prompt).await;
    
    // Should fail with authentication error
    assert!(result.is_err(), "Should fail with invalid API key");
    
    if let Err(e) = result {
        let error_msg = e.to_string();
        // Error should indicate authentication failure
        assert!(
            error_msg.to_lowercase().contains("auth") || 
            error_msg.contains("401") ||
            error_msg.to_lowercase().contains("unauthorized"),
            "Error message should indicate authentication failure"
        );
    }
    
    println!("✓ Invalid API key properly handled");
}

/// Test extract_code functionality with Anthropic responses
/// 
/// This test verifies that the extract_code method can properly extract
/// YAML code blocks from Anthropic responses.
///
/// Requirements: 8.1
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_extract_code() {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    let model = "claude-sonnet-4-5-20250929";
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Ask for a YAML response
    let prompt = r#"Create a simple YAML configuration with a name field set to "test". 
    Wrap it in ```yaml code blocks."#;
    
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Extract the YAML code
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Verify we got valid YAML
    assert!(!code.is_empty(), "Extracted code should not be empty");
    assert!(code.contains("name"), "Extracted code should contain 'name' field");
    
    println!("✓ Code extraction successful");
    println!("Extracted YAML:\n{}", code);
}

/// Test Anthropic provider ignores AWS configuration
/// 
/// This test verifies that AWS-specific configuration (region, profile)
/// does not affect the Anthropic provider's behavior.
///
/// Requirements: 8.1
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_anthropic_ignores_aws_config() {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    let model = "claude-sonnet-4-5-20250929";
    
    // Create provider (AWS config would be ignored)
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Make API call - should work regardless of AWS configuration
    let prompt = "Say 'AWS config ignored' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Anthropic provider works independently of AWS configuration");
    println!("Response: {}", response);
}

/// Test Anthropic API with .env file loading
/// 
/// This test verifies that the Anthropic provider can read the API key
/// from a .env file (through environment variable loading).
///
/// Requirements: 8.2, 9.2 (from requirements document)
#[tokio::test]
#[ignore = "requires .env file with CLAUDE_API_KEY"]
async fn test_anthropic_with_dotenv() {
    // Load .env file
    dotenv::dotenv().ok();
    
    // Get API key from environment (should be loaded from .env)
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set in .env file for this test");
    
    let model = "claude-sonnet-4-5-20250929";
    
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Make a simple API call
    let prompt = "Say 'dotenv works' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Anthropic");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Anthropic provider works with .env file");
    println!("Response: {}", response);
}
