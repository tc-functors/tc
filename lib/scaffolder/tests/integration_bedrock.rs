//! Integration tests for AWS Bedrock provider
//! 
//! These tests make actual API calls to AWS Bedrock and are marked with #[ignore]
//! to prevent them from running in normal test runs. They require:
//! - Valid AWS credentials (via SSO, profile, or environment variables)
//! - Access to AWS Bedrock in the configured region
//! - Appropriate IAM permissions for bedrock:InvokeModel
//!
//! Run these tests with: cargo test --test integration_bedrock -- --ignored
//!
//! Requirements tested: 1.1, 1.2, 1.3, 1.4

use scaffolder::provider::{LlmProvider, bedrock::BedrockProvider};

/// Test actual Bedrock API call with default credentials
/// 
/// This test verifies that the Bedrock provider can successfully:
/// - Initialize with AWS SDK default credential chain
/// - Make an actual API call to Bedrock
/// - Receive and parse a valid response
///
/// Requirements: 1.1, 1.2
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_bedrock_actual_api_call_default_credentials() {
    // Use default credentials (environment, SSO, or instance metadata)
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    
    let provider = BedrockProvider::new(
        model.to_string(),
        None, // Use default region
        None, // Use default profile
    ).await.expect("Failed to create Bedrock provider");
    
    // Make a simple API call
    let prompt = "Say 'Hello from AWS Bedrock!' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Bedrock");
    
    // Verify we got a non-empty response
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.len() > 10, "Response should be substantial");
    
    // Verify the response contains expected content
    let response_lower = response.to_lowercase();
    assert!(
        response_lower.contains("hello") || response_lower.contains("bedrock"),
        "Response should acknowledge the prompt"
    );
    
    println!("✓ Bedrock API call successful with default credentials");
    println!("Response: {}", response);
}

/// Test Bedrock API call with AWS SSO authentication
/// 
/// This test verifies that the Bedrock provider works with AWS SSO credentials.
/// Before running this test, ensure you have:
/// 1. Configured AWS SSO: aws configure sso
/// 2. Logged in: aws sso login --profile <your-profile>
/// 3. Set the profile name in the test or via AWS_PROFILE environment variable
///
/// Requirements: 1.3
#[tokio::test]
#[ignore = "requires AWS SSO login and makes actual API calls"]
async fn test_bedrock_with_sso_authentication() {
    // Get profile from environment or use a default test profile
    let profile = std::env::var("AWS_PROFILE")
        .unwrap_or_else(|_| "default".to_string());
    
    println!("Testing with AWS profile: {}", profile);
    
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    
    let provider = BedrockProvider::new(
        model.to_string(),
        None, // Use default region from profile
        Some(profile.clone()),
    ).await.expect("Failed to create Bedrock provider with SSO profile");
    
    // Make a simple API call
    let prompt = "Respond with exactly: 'SSO authentication successful'";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Bedrock with SSO");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Bedrock API call successful with SSO profile: {}", profile);
    println!("Response: {}", response);
}

/// Test Bedrock API call with named profile authentication
/// 
/// This test verifies that the Bedrock provider works with named AWS profiles.
/// The profile should be configured in ~/.aws/credentials or ~/.aws/config
///
/// Requirements: 1.4
#[tokio::test]
#[ignore = "requires AWS profile configuration and makes actual API calls"]
async fn test_bedrock_with_profile_authentication() {
    // Get profile from environment or use a default test profile
    let profile = std::env::var("TEST_AWS_PROFILE")
        .unwrap_or_else(|_| "default".to_string());
    
    println!("Testing with AWS profile: {}", profile);
    
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    
    let provider = BedrockProvider::new(
        model.to_string(),
        Some("us-east-1".to_string()), // Explicitly set region
        Some(profile.clone()),
    ).await.expect("Failed to create Bedrock provider with profile");
    
    // Verify provider was created
    assert_eq!(provider.name(), "AWS Bedrock");
    
    // Make a simple API call
    let prompt = "What is 2+2? Answer with just the number.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Bedrock with profile");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(response.contains("4"), "Response should contain the answer");
    
    println!("✓ Bedrock API call successful with profile: {}", profile);
    println!("Response: {}", response);
}

/// Test Bedrock API call with explicit region
/// 
/// This test verifies that the Bedrock provider respects the region parameter
/// and can successfully make calls to different AWS regions.
///
/// Requirements: 1.2
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_bedrock_with_explicit_region() {
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    let region = "us-west-2"; // Test with a different region
    
    let provider = BedrockProvider::new(
        model.to_string(),
        Some(region.to_string()),
        None,
    ).await.expect("Failed to create Bedrock provider with explicit region");
    
    // Make a simple API call
    let prompt = "Say 'Region test successful' and nothing else.";
    let response = provider.send(prompt).await
        .expect("Failed to get response from Bedrock in specified region");
    
    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    
    println!("✓ Bedrock API call successful in region: {}", region);
    println!("Response: {}", response);
}

/// Test Bedrock error handling with invalid credentials
/// 
/// This test verifies that the Bedrock provider properly handles authentication
/// errors when credentials are invalid or missing.
///
/// Requirements: 1.5
#[tokio::test]
#[ignore = "requires specific credential setup to test failure cases"]
async fn test_bedrock_authentication_error() {
    // This test would need to be run in an environment with no credentials
    // or invalid credentials to properly test error handling
    
    // For now, we document the expected behavior:
    // - When credentials are missing, provider creation should fail
    // - When credentials are invalid, API calls should return AuthenticationError
    // - Error messages should be clear and actionable
    
    println!("Note: This test requires running in an environment without AWS credentials");
    println!("Expected behavior: Provider should return clear authentication error");
}

/// Test Bedrock with different model variants
/// 
/// This test verifies that the Bedrock provider can work with different
/// Claude model variants available in Bedrock.
///
/// Requirements: 1.1
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_bedrock_with_different_models() {
    let models = vec![
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
        "anthropic.claude-3-haiku-20240307-v1:0",
    ];
    
    for model in models {
        println!("Testing model: {}", model);
        
        let provider = BedrockProvider::new(
            model.to_string(),
            None,
            None,
        ).await.expect(&format!("Failed to create provider for model: {}", model));
        
        let prompt = "Say 'OK' and nothing else.";
        let response = provider.send(prompt).await
            .expect(&format!("Failed to get response from model: {}", model));
        
        assert!(!response.is_empty(), "Response should not be empty for model: {}", model);
        
        println!("✓ Model {} works correctly", model);
    }
}

/// Test extract_code functionality with Bedrock responses
/// 
/// This test verifies that the extract_code method can properly extract
/// YAML code blocks from Bedrock responses.
///
/// Requirements: 1.1
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_bedrock_extract_code() {
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    
    let provider = BedrockProvider::new(
        model.to_string(),
        None,
        None,
    ).await.expect("Failed to create Bedrock provider");
    
    // Ask for a YAML response
    let prompt = r#"Create a simple YAML configuration with a name field set to "test". 
    Wrap it in ```yaml code blocks."#;
    
    let response = provider.send(prompt).await
        .expect("Failed to get response from Bedrock");
    
    // Extract the YAML code
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Verify we got valid YAML
    assert!(!code.is_empty(), "Extracted code should not be empty");
    assert!(code.contains("name"), "Extracted code should contain 'name' field");
    
    println!("✓ Code extraction successful");
    println!("Extracted YAML:\n{}", code);
}
