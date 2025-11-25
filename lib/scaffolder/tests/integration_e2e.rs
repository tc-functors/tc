//! End-to-end integration tests for scaffolding workflow
//! 
//! These tests verify the complete scaffolding workflow from prompt to
//! generated topology.yml file. They are marked with #[ignore] to prevent
//! them from running in normal test runs.
//!
//! For Bedrock tests, you need:
//! - Valid AWS credentials (via SSO, profile, or environment variables)
//! - Access to AWS Bedrock in the configured region
//!
//! For Anthropic tests, you need:
//! - Valid CLAUDE_API_KEY environment variable
//!
//! Run these tests with: cargo test --test integration_e2e -- --ignored
//!
//! Requirements tested: 1.1, 8.1

use std::fs;
use std::path::Path;
use scaffolder::config::{LlmConfig, LlmProvider as LlmProviderEnum};
use scaffolder::provider::{LlmProvider, bedrock::BedrockProvider, anthropic::AnthropicProvider};

/// Helper function to create a test directory
fn create_test_dir(name: &str) -> String {
    let test_dir = format!("/tmp/tc-test-{}", name);
    // Clean up if exists
    let _ = fs::remove_dir_all(&test_dir);
    fs::create_dir_all(&test_dir).expect("Failed to create test directory");
    test_dir
}

/// Helper function to verify topology.yml file
fn verify_topology_file(dir: &str) {
    let topology_path = format!("{}/topology.yml", dir);
    assert!(
        Path::new(&topology_path).exists(),
        "topology.yml should be created"
    );
    
    let content = fs::read_to_string(&topology_path)
        .expect("Failed to read topology.yml");
    
    // Verify it's not empty
    assert!(!content.is_empty(), "topology.yml should not be empty");
    
    // Verify it contains YAML structure
    assert!(
        content.contains("name:") || content.contains("functions:") || content.contains("routes:"),
        "topology.yml should contain valid YAML structure"
    );
    
    // Try to parse as YAML to ensure it's valid
    let yaml_result: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
    assert!(
        yaml_result.is_ok(),
        "topology.yml should be valid YAML: {:?}",
        yaml_result.err()
    );
    
    println!("✓ topology.yml is valid");
    println!("Content preview:\n{}", &content[..content.len().min(500)]);
}

/// Test complete scaffold workflow with Bedrock provider
/// 
/// This test verifies the end-to-end scaffolding process:
/// 1. Create provider with Bedrock configuration
/// 2. Send a prompt to generate topology
/// 3. Extract YAML code from response
/// 4. Verify generated topology.yml file
///
/// Requirements: 1.1
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_e2e_scaffold_with_bedrock() {
    let test_dir = create_test_dir("bedrock-scaffold");
    
    // Create Bedrock provider
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    let provider = BedrockProvider::new(
        model.to_string(),
        None, // Use default region
        None, // Use default profile
    ).await.expect("Failed to create Bedrock provider");
    
    // Create a simple application prompt
    let prompt = r#"Create a simple REST API with the following:
- A GET endpoint at /api/hello that returns a greeting
- A POST endpoint at /api/echo that echoes back the request body

Keep it minimal and simple."#;
    
    // Generate the full prompt (simulating what scaffold() does)
    let full_prompt = format!(
        "You are an expert at creating tc topologies. Create a topology for: {}",
        prompt
    );
    
    println!("Sending prompt to Bedrock...");
    let response = provider.send(&full_prompt).await
        .expect("Failed to get response from Bedrock");
    
    println!("Extracting YAML code...");
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Write to file
    let topology_path = format!("{}/topology.yml", test_dir);
    fs::write(&topology_path, &code)
        .expect("Failed to write topology.yml");
    
    // Verify the generated file
    verify_topology_file(&test_dir);
    
    // Clean up
    fs::remove_dir_all(&test_dir).ok();
    
    println!("✓ End-to-end scaffold with Bedrock successful");
}

/// Test complete scaffold workflow with Anthropic provider
/// 
/// This test verifies the end-to-end scaffolding process with Anthropic:
/// 1. Create provider with Anthropic configuration
/// 2. Send a prompt to generate topology
/// 3. Extract YAML code from response
/// 4. Verify generated topology.yml file
///
/// Requirements: 8.1
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_e2e_scaffold_with_anthropic() {
    let test_dir = create_test_dir("anthropic-scaffold");
    
    // Get API key from environment
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    // Create Anthropic provider
    let model = "claude-sonnet-4-5-20250929";
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Create a simple application prompt
    let prompt = r#"Create a simple REST API with the following:
- A GET endpoint at /api/status that returns system status
- A POST endpoint at /api/data that processes data

Keep it minimal and simple."#;
    
    // Generate the full prompt (simulating what scaffold() does)
    let full_prompt = format!(
        "You are an expert at creating tc topologies. Create a topology for: {}",
        prompt
    );
    
    println!("Sending prompt to Anthropic...");
    let response = provider.send(&full_prompt).await
        .expect("Failed to get response from Anthropic");
    
    println!("Extracting YAML code...");
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Write to file
    let topology_path = format!("{}/topology.yml", test_dir);
    fs::write(&topology_path, &code)
        .expect("Failed to write topology.yml");
    
    // Verify the generated file
    verify_topology_file(&test_dir);
    
    // Clean up
    fs::remove_dir_all(&test_dir).ok();
    
    println!("✓ End-to-end scaffold with Anthropic successful");
}

/// Test scaffold workflow with different application types
/// 
/// This test verifies that the scaffolding works for various application types
/// using Bedrock provider.
///
/// Requirements: 1.1
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_e2e_scaffold_different_app_types() {
    let test_cases = vec![
        ("websocket-chat", "A real-time chat application using WebSockets"),
        ("async-queue", "An async job processing system with queues"),
        ("event-driven", "An event-driven notification system"),
    ];
    
    for (name, description) in test_cases {
        println!("\nTesting application type: {}", name);
        
        let test_dir = create_test_dir(&format!("bedrock-{}", name));
        
        // Create Bedrock provider
        let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
        let provider = BedrockProvider::new(
            model.to_string(),
            None,
            None,
        ).await.expect("Failed to create Bedrock provider");
        
        // Generate topology
        let prompt = format!(
            "You are an expert at creating tc topologies. Create a topology for: {}",
            description
        );
        
        let response = provider.send(&prompt).await
            .expect("Failed to get response from Bedrock");
        
        let code = provider.extract_code(&response)
            .expect("Failed to extract code from response");
        
        // Write to file
        let topology_path = format!("{}/topology.yml", test_dir);
        fs::write(&topology_path, &code)
            .expect("Failed to write topology.yml");
        
        // Verify the generated file
        verify_topology_file(&test_dir);
        
        // Clean up
        fs::remove_dir_all(&test_dir).ok();
        
        println!("✓ Application type '{}' generated successfully", name);
    }
}

/// Test backward compatibility: Anthropic provider generates same quality output
/// 
/// This test verifies that the Anthropic provider maintains backward compatibility
/// and generates topology files of the same quality as before.
///
/// Requirements: 8.1, 8.3
#[tokio::test]
#[ignore = "requires CLAUDE_API_KEY and makes actual API calls"]
async fn test_e2e_anthropic_backward_compatibility() {
    let test_dir = create_test_dir("anthropic-compat");
    
    // Get API key from environment
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY must be set for this test");
    
    // Create Anthropic provider with default model
    let model = scaffolder::llm::DEFAULT_ANTHROPIC_MODEL;
    let provider = AnthropicProvider::new(api_key, model.to_string())
        .expect("Failed to create Anthropic provider");
    
    // Use a standard prompt that would have worked with the old implementation
    let prompt = r#"Create a REST API for a todo list application with:
- GET /api/todos - list all todos
- POST /api/todos - create a new todo
- PUT /api/todos/:id - update a todo
- DELETE /api/todos/:id - delete a todo"#;
    
    let full_prompt = format!(
        "You are an expert at creating tc topologies. Create a topology for: {}",
        prompt
    );
    
    println!("Testing backward compatibility with Anthropic...");
    let response = provider.send(&full_prompt).await
        .expect("Failed to get response from Anthropic");
    
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Write to file
    let topology_path = format!("{}/topology.yml", test_dir);
    fs::write(&topology_path, &code)
        .expect("Failed to write topology.yml");
    
    // Verify the generated file
    verify_topology_file(&test_dir);
    
    // Additional checks for backward compatibility
    let content = fs::read_to_string(&topology_path).unwrap();
    
    // Should contain routes for the REST API
    assert!(
        content.contains("routes:") || content.contains("/api/"),
        "Should contain API routes"
    );
    
    // Should contain functions
    assert!(
        content.contains("functions:"),
        "Should contain functions"
    );
    
    // Clean up
    fs::remove_dir_all(&test_dir).ok();
    
    println!("✓ Backward compatibility verified");
}

/// Test configuration-based provider selection
/// 
/// This test verifies that the scaffolding workflow correctly uses
/// the provider specified in configuration.
///
/// Requirements: 1.1, 8.1
#[tokio::test]
#[ignore = "requires both AWS credentials and CLAUDE_API_KEY"]
async fn test_e2e_provider_selection() {
    // Test with Bedrock
    {
        let test_dir = create_test_dir("provider-bedrock");
        
        let config = LlmConfig {
            provider: LlmProviderEnum::Bedrock,
            model: Some("anthropic.claude-3-5-sonnet-20241022-v2:0".to_string()),
            aws_region: None,
            aws_profile: None,
        };
        
        // Create provider based on config
        let provider: Box<dyn LlmProvider> = match config.provider {
            LlmProviderEnum::Bedrock => {
                Box::new(BedrockProvider::new(
                    config.model.clone().unwrap(),
                    config.aws_region.clone(),
                    config.aws_profile.clone(),
                ).await.expect("Failed to create Bedrock provider"))
            }
            LlmProviderEnum::Anthropic => {
                panic!("Should not reach here");
            }
        };
        
        assert_eq!(provider.name(), "AWS Bedrock");
        
        // Generate a simple topology
        let prompt = "Create a simple hello world API";
        let response = provider.send(prompt).await
            .expect("Failed to get response");
        
        let code = provider.extract_code(&response)
            .expect("Failed to extract code");
        
        let topology_path = format!("{}/topology.yml", test_dir);
        fs::write(&topology_path, &code)
            .expect("Failed to write topology.yml");
        
        verify_topology_file(&test_dir);
        fs::remove_dir_all(&test_dir).ok();
        
        println!("✓ Bedrock provider selected correctly");
    }
    
    // Test with Anthropic
    {
        let test_dir = create_test_dir("provider-anthropic");
        
        let api_key = std::env::var("CLAUDE_API_KEY")
            .expect("CLAUDE_API_KEY must be set");
        
        let config = LlmConfig {
            provider: LlmProviderEnum::Anthropic,
            model: Some("claude-sonnet-4-5-20250929".to_string()),
            aws_region: None,
            aws_profile: None,
        };
        
        // Create provider based on config
        let provider: Box<dyn LlmProvider> = match config.provider {
            LlmProviderEnum::Bedrock => {
                panic!("Should not reach here");
            }
            LlmProviderEnum::Anthropic => {
                Box::new(AnthropicProvider::new(
                    api_key,
                    config.model.clone().unwrap(),
                ).expect("Failed to create Anthropic provider"))
            }
        };
        
        assert_eq!(provider.name(), "Anthropic");
        
        // Generate a simple topology
        let prompt = "Create a simple hello world API";
        let response = provider.send(prompt).await
            .expect("Failed to get response");
        
        let code = provider.extract_code(&response)
            .expect("Failed to extract code");
        
        let topology_path = format!("{}/topology.yml", test_dir);
        fs::write(&topology_path, &code)
            .expect("Failed to write topology.yml");
        
        verify_topology_file(&test_dir);
        fs::remove_dir_all(&test_dir).ok();
        
        println!("✓ Anthropic provider selected correctly");
    }
}

/// Test that generated topologies are valid and can be parsed
/// 
/// This test verifies that the generated topology.yml files are not only
/// valid YAML but also contain the expected structure for tc topologies.
///
/// Requirements: 1.1, 8.1
#[tokio::test]
#[ignore = "requires AWS credentials and makes actual API calls"]
async fn test_e2e_topology_validity() {
    let test_dir = create_test_dir("topology-validity");
    
    // Create Bedrock provider
    let model = "anthropic.claude-3-5-sonnet-20241022-v2:0";
    let provider = BedrockProvider::new(
        model.to_string(),
        None,
        None,
    ).await.expect("Failed to create Bedrock provider");
    
    // Generate a topology with specific requirements
    let prompt = r#"Create a REST API with:
- A name field
- At least one route
- At least one function
- Proper function chaining"#;
    
    let full_prompt = format!(
        "You are an expert at creating tc topologies. Create a topology for: {}",
        prompt
    );
    
    let response = provider.send(&full_prompt).await
        .expect("Failed to get response from Bedrock");
    
    let code = provider.extract_code(&response)
        .expect("Failed to extract code from response");
    
    // Write to file
    let topology_path = format!("{}/topology.yml", test_dir);
    fs::write(&topology_path, &code)
        .expect("Failed to write topology.yml");
    
    // Parse and validate structure
    let content = fs::read_to_string(&topology_path).unwrap();
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .expect("Should be valid YAML");
    
    // Check for expected fields
    let yaml_map = yaml.as_mapping()
        .expect("Root should be a mapping");
    
    // Should have a name
    assert!(
        yaml_map.contains_key(&serde_yaml::Value::String("name".to_string())),
        "Topology should have a name field"
    );
    
    // Should have at least routes or functions
    let has_routes = yaml_map.contains_key(&serde_yaml::Value::String("routes".to_string()));
    let has_functions = yaml_map.contains_key(&serde_yaml::Value::String("functions".to_string()));
    
    assert!(
        has_routes || has_functions,
        "Topology should have routes or functions"
    );
    
    // Clean up
    fs::remove_dir_all(&test_dir).ok();
    
    println!("✓ Generated topology has valid structure");
}
