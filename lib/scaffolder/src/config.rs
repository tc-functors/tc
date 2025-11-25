use std::str::FromStr;

/// Configuration for LLM provider selection and settings
#[derive(Debug, Clone, PartialEq)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: Option<String>,
    pub aws_region: Option<String>,
    pub aws_profile: Option<String>,
}

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    Bedrock,
    Anthropic,
}

/// Error type for configuration operations
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    InvalidProvider(String),
    MissingFile(String),
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidProvider(msg) => write!(f, "Invalid provider: {}", msg),
            ConfigError::MissingFile(msg) => write!(f, "Missing file: {}", msg),
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

impl FromStr for LlmProvider {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bedrock" => Ok(LlmProvider::Bedrock),
            "anthropic" => Ok(LlmProvider::Anthropic),
            _ => Err(ConfigError::InvalidProvider(format!(
                "Unknown provider '{}'. Valid options are: 'bedrock', 'anthropic'",
                s
            ))),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::Bedrock,
            model: None,
            aws_region: None,
            aws_profile: None,
        }
    }
}

impl LlmConfig {
    /// Load configuration from environment variables
    /// Supports: TC_LLM_PROVIDER, TC_LLM_MODEL, AWS_REGION, AWS_PROFILE
    /// 
    /// This method will attempt to load a .env file from the current directory
    /// before reading environment variables. Shell environment variables take
    /// precedence over .env file variables. Missing .env files are handled
    /// gracefully without error.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists (gracefully handle missing file)
        dotenv::dotenv().ok();
        
        let provider = std::env::var("TC_LLM_PROVIDER")
            .ok()
            .map(|s| LlmProvider::from_str(&s))
            .transpose()?
            .unwrap_or(LlmProvider::Bedrock);

        let model = std::env::var("TC_LLM_MODEL").ok();
        let aws_region = std::env::var("AWS_REGION").ok();
        let aws_profile = std::env::var("AWS_PROFILE").ok();

        Ok(Self {
            provider,
            model,
            aws_region,
            aws_profile,
        })
    }

    /// Load configuration from a TOML file
    /// Expected format:
    /// [llm]
    /// provider = "bedrock"
    /// model = "anthropic.claude-3-5-sonnet-20241022-v2:0"
    ///
    /// [llm.aws]
    /// region = "us-east-1"
    /// profile = "my-profile"
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct ConfigFile {
            llm: Option<LlmSection>,
        }

        #[derive(Deserialize)]
        struct LlmSection {
            provider: Option<String>,
            model: Option<String>,
            aws: Option<AwsSection>,
        }

        #[derive(Deserialize)]
        struct AwsSection {
            region: Option<String>,
            profile: Option<String>,
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::MissingFile(e.to_string()))?;

        let config: ConfigFile = toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;

        let llm = config.llm.unwrap_or(LlmSection {
            provider: None,
            model: None,
            aws: None,
        });

        let provider = llm
            .provider
            .map(|s| LlmProvider::from_str(&s))
            .transpose()?
            .unwrap_or(LlmProvider::Bedrock);

        let (aws_region, aws_profile) = if let Some(aws) = llm.aws {
            (aws.region, aws.profile)
        } else {
            (None, None)
        };

        Ok(Self {
            provider,
            model: llm.model,
            aws_region,
            aws_profile,
        })
    }

    /// Merge configurations with precedence: CLI > Env > File > Default
    /// Each field is merged independently
    pub fn merge(cli: Option<Self>, env: Self, file: Option<Self>) -> Self {
        let default = Self::default();

        // Helper to select value with precedence
        let select_provider = |cli: Option<LlmProvider>, env: LlmProvider, file: Option<LlmProvider>| {
            cli.or(Some(env)).or(file).unwrap_or(default.provider)
        };

        let select_option = |cli: Option<String>, env: Option<String>, file: Option<String>| {
            cli.or(env).or(file)
        };

        let provider = select_provider(
            cli.as_ref().map(|c| c.provider),
            env.provider,
            file.as_ref().map(|f| f.provider),
        );

        let model = select_option(
            cli.as_ref().and_then(|c| c.model.clone()),
            env.model,
            file.as_ref().and_then(|f| f.model.clone()),
        );

        let aws_region = select_option(
            cli.as_ref().and_then(|c| c.aws_region.clone()),
            env.aws_region,
            file.as_ref().and_then(|f| f.aws_region.clone()),
        );

        let aws_profile = select_option(
            cli.as_ref().and_then(|c| c.aws_profile.clone()),
            env.aws_profile,
            file.as_ref().and_then(|f| f.aws_profile.clone()),
        );

        Self {
            provider,
            model,
            aws_region,
            aws_profile,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_provider_is_bedrock() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, LlmProvider::Bedrock);
    }

    #[test]
    fn test_invalid_provider_returns_error() {
        let result = LlmProvider::from_str("invalid");
        assert!(result.is_err());
        
        if let Err(ConfigError::InvalidProvider(msg)) = result {
            assert!(msg.contains("bedrock"));
            assert!(msg.contains("anthropic"));
        } else {
            panic!("Expected InvalidProvider error");
        }
    }

    #[test]
    fn test_valid_provider_parsing() {
        assert_eq!(LlmProvider::from_str("bedrock").unwrap(), LlmProvider::Bedrock);
        assert_eq!(LlmProvider::from_str("Bedrock").unwrap(), LlmProvider::Bedrock);
        assert_eq!(LlmProvider::from_str("BEDROCK").unwrap(), LlmProvider::Bedrock);
        assert_eq!(LlmProvider::from_str("anthropic").unwrap(), LlmProvider::Anthropic);
        assert_eq!(LlmProvider::from_str("Anthropic").unwrap(), LlmProvider::Anthropic);
    }

    #[test]
    fn test_config_merge_cli_precedence() {
        let cli = Some(LlmConfig {
            provider: LlmProvider::Anthropic,
            model: Some("cli-model".to_string()),
            aws_region: Some("cli-region".to_string()),
            aws_profile: Some("cli-profile".to_string()),
        });

        let env = LlmConfig {
            provider: LlmProvider::Bedrock,
            model: Some("env-model".to_string()),
            aws_region: Some("env-region".to_string()),
            aws_profile: Some("env-profile".to_string()),
        };

        let file = Some(LlmConfig {
            provider: LlmProvider::Bedrock,
            model: Some("file-model".to_string()),
            aws_region: Some("file-region".to_string()),
            aws_profile: Some("file-profile".to_string()),
        });

        let merged = LlmConfig::merge(cli, env, file);

        assert_eq!(merged.provider, LlmProvider::Anthropic);
        assert_eq!(merged.model, Some("cli-model".to_string()));
        assert_eq!(merged.aws_region, Some("cli-region".to_string()));
        assert_eq!(merged.aws_profile, Some("cli-profile".to_string()));
    }

    #[test]
    fn test_config_merge_env_precedence() {
        let env = LlmConfig {
            provider: LlmProvider::Anthropic,
            model: Some("env-model".to_string()),
            aws_region: Some("env-region".to_string()),
            aws_profile: Some("env-profile".to_string()),
        };

        let file = Some(LlmConfig {
            provider: LlmProvider::Bedrock,
            model: Some("file-model".to_string()),
            aws_region: Some("file-region".to_string()),
            aws_profile: Some("file-profile".to_string()),
        });

        let merged = LlmConfig::merge(None, env, file);

        assert_eq!(merged.provider, LlmProvider::Anthropic);
        assert_eq!(merged.model, Some("env-model".to_string()));
        assert_eq!(merged.aws_region, Some("env-region".to_string()));
        assert_eq!(merged.aws_profile, Some("env-profile".to_string()));
    }

    #[test]
    fn test_config_merge_file_precedence() {
        // Env has no values set (only defaults)
        let env = LlmConfig {
            provider: LlmProvider::Bedrock,
            model: None,
            aws_region: None,
            aws_profile: None,
        };

        let file = Some(LlmConfig {
            provider: LlmProvider::Anthropic,
            model: Some("file-model".to_string()),
            aws_region: Some("file-region".to_string()),
            aws_profile: Some("file-profile".to_string()),
        });

        let merged = LlmConfig::merge(None, env, file);

        // Env provider wins (Bedrock) because env is always present
        assert_eq!(merged.provider, LlmProvider::Bedrock);
        // File values win for optional fields where env has None
        assert_eq!(merged.model, Some("file-model".to_string()));
        assert_eq!(merged.aws_region, Some("file-region".to_string()));
        assert_eq!(merged.aws_profile, Some("file-profile".to_string()));
    }

    #[test]
    fn test_config_merge_partial_values() {
        let cli = Some(LlmConfig {
            provider: LlmProvider::Anthropic,
            model: None,
            aws_region: None,
            aws_profile: None,
        });

        let env = LlmConfig {
            provider: LlmProvider::Bedrock,
            model: Some("env-model".to_string()),
            aws_region: None,
            aws_profile: None,
        };

        let file = Some(LlmConfig {
            provider: LlmProvider::Bedrock,
            model: Some("file-model".to_string()),
            aws_region: Some("file-region".to_string()),
            aws_profile: Some("file-profile".to_string()),
        });

        let merged = LlmConfig::merge(cli, env, file);

        // CLI provider wins
        assert_eq!(merged.provider, LlmProvider::Anthropic);
        // Env model wins (CLI was None)
        assert_eq!(merged.model, Some("env-model".to_string()));
        // File region wins (CLI and env were None)
        assert_eq!(merged.aws_region, Some("file-region".to_string()));
        assert_eq!(merged.aws_profile, Some("file-profile".to_string()));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    // Feature: aws-bedrock-integration, Property 2: Invalid provider rejection
    // Validates: Requirements 2.5
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_invalid_provider_rejection(
            invalid_name in "[a-z]{1,20}".prop_filter(
                "not a valid provider",
                |s| s != "bedrock" && s != "anthropic"
            )
        ) {
            // For any invalid provider string, parsing should fail with helpful error
            let result = LlmProvider::from_str(&invalid_name);
            prop_assert!(result.is_err());
            
            if let Err(ConfigError::InvalidProvider(msg)) = result {
                prop_assert!(msg.contains("bedrock"), "Error message should mention 'bedrock'");
                prop_assert!(msg.contains("anthropic"), "Error message should mention 'anthropic'");
            } else {
                return Err(proptest::test_runner::TestCaseError::fail("Expected InvalidProvider error"));
            }
        }
    }

    // Feature: aws-bedrock-integration, Property 1: Configuration precedence hierarchy
    // Validates: Requirements 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_config_precedence_provider(
            cli_provider in proptest::option::of(prop_oneof![
                Just(LlmProvider::Bedrock),
                Just(LlmProvider::Anthropic),
            ]),
            env_provider in prop_oneof![
                Just(LlmProvider::Bedrock),
                Just(LlmProvider::Anthropic),
            ],
            file_provider in proptest::option::of(prop_oneof![
                Just(LlmProvider::Bedrock),
                Just(LlmProvider::Anthropic),
            ]),
        ) {
            let cli = cli_provider.map(|p| LlmConfig {
                provider: p,
                model: None,
                aws_region: None,
                aws_profile: None,
            });
            
            let env = LlmConfig {
                provider: env_provider,
                model: None,
                aws_region: None,
                aws_profile: None,
            };
            
            let file = file_provider.map(|p| LlmConfig {
                provider: p,
                model: None,
                aws_region: None,
                aws_profile: None,
            });
            
            let merged = LlmConfig::merge(cli.clone(), env.clone(), file.clone());
            
            // Verify precedence: CLI > Env > File > Default
            let expected_provider = cli
                .map(|c| c.provider)
                .or(Some(env.provider))
                .or(file.map(|f| f.provider))
                .unwrap_or(LlmProvider::Bedrock);
            
            prop_assert_eq!(merged.provider, expected_provider);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_config_precedence_model(
            cli_model in proptest::option::of("[a-z0-9.-]{5,30}"),
            env_model in proptest::option::of("[a-z0-9.-]{5,30}"),
            file_model in proptest::option::of("[a-z0-9.-]{5,30}"),
        ) {
            let cli = cli_model.clone().map(|m| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: Some(m),
                aws_region: None,
                aws_profile: None,
            });
            
            let env = LlmConfig {
                provider: LlmProvider::Bedrock,
                model: env_model.clone(),
                aws_region: None,
                aws_profile: None,
            };
            
            let file = file_model.clone().map(|m| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: Some(m),
                aws_region: None,
                aws_profile: None,
            });
            
            let merged = LlmConfig::merge(cli.clone(), env.clone(), file.clone());
            
            // Verify precedence: CLI > Env > File > Default (None)
            let expected_model = cli
                .and_then(|c| c.model)
                .or(env.model)
                .or(file.and_then(|f| f.model));
            
            prop_assert_eq!(merged.model, expected_model);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_config_precedence_region(
            cli_region in proptest::option::of("[a-z]{2}-[a-z]+-[0-9]"),
            env_region in proptest::option::of("[a-z]{2}-[a-z]+-[0-9]"),
            file_region in proptest::option::of("[a-z]{2}-[a-z]+-[0-9]"),
        ) {
            let cli = cli_region.clone().map(|r| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: Some(r),
                aws_profile: None,
            });
            
            let env = LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: env_region.clone(),
                aws_profile: None,
            };
            
            let file = file_region.clone().map(|r| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: Some(r),
                aws_profile: None,
            });
            
            let merged = LlmConfig::merge(cli.clone(), env.clone(), file.clone());
            
            // Verify precedence: CLI > Env > File > Default (None)
            let expected_region = cli
                .and_then(|c| c.aws_region)
                .or(env.aws_region)
                .or(file.and_then(|f| f.aws_region));
            
            prop_assert_eq!(merged.aws_region, expected_region);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_config_precedence_profile(
            cli_profile in proptest::option::of("[a-z0-9_-]{3,20}"),
            env_profile in proptest::option::of("[a-z0-9_-]{3,20}"),
            file_profile in proptest::option::of("[a-z0-9_-]{3,20}"),
        ) {
            let cli = cli_profile.clone().map(|p| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: None,
                aws_profile: Some(p),
            });
            
            let env = LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: None,
                aws_profile: env_profile.clone(),
            };
            
            let file = file_profile.clone().map(|p| LlmConfig {
                provider: LlmProvider::Bedrock,
                model: None,
                aws_region: None,
                aws_profile: Some(p),
            });
            
            let merged = LlmConfig::merge(cli.clone(), env.clone(), file.clone());
            
            // Verify precedence: CLI > Env > File > Default (None)
            let expected_profile = cli
                .and_then(|c| c.aws_profile)
                .or(env.aws_profile)
                .or(file.and_then(|f| f.aws_profile));
            
            prop_assert_eq!(merged.aws_profile, expected_profile);
        }
    }

    // Feature: aws-bedrock-integration, Property 8: Shell environment precedence over .env
    // Validates: Requirements 9.3
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_shell_env_precedence_over_dotenv(
            var_name in "[A-Z_]{5,15}",
            shell_value in "[a-z0-9-]{5,20}",
            dotenv_value in "[a-z0-9-]{5,20}",
            random_id in 0u64..1000000u64
        ) {
            // Skip if values are the same (we want to test precedence)
            prop_assume!(shell_value != dotenv_value);
            
            // Create a unique temporary directory for this test
            let temp_dir = std::env::temp_dir().join(format!("tc_test_shell_{}_{}_{}", std::process::id(), random_id, var_name));
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            // Create a .env file with the dotenv value
            let dotenv_path = temp_dir.join(".env");
            std::fs::write(&dotenv_path, format!("{}={}", var_name, dotenv_value)).unwrap();
            
            // Set the shell environment variable
            unsafe {
                std::env::set_var(&var_name, &shell_value);
            }
            
            // Load .env file from the specific path (this is what from_env() does)
            dotenv::from_path(&dotenv_path).ok();
            
            // Verify shell value takes precedence
            let actual_value = std::env::var(&var_name).unwrap();
            prop_assert_eq!(actual_value, shell_value, 
                "Shell environment variable should take precedence over .env file");
            
            // Cleanup
            unsafe {
                std::env::remove_var(&var_name);
            }
            std::fs::remove_dir_all(&temp_dir).ok();
        }
    }

    // Feature: aws-bedrock-integration, Property 7: Environment variable loading from .env
    // Validates: Requirements 9.2
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]
        
        #[test]
        fn prop_dotenv_variable_loading(
            var_name in "[A-Z_]{5,15}",
            var_value in "[a-z0-9-]{5,20}",
            random_id in 0u64..1000000u64
        ) {
            // Create a unique temporary directory for this test
            let temp_dir = std::env::temp_dir().join(format!("tc_test_dotenv_{}_{}_{}", std::process::id(), random_id, var_name));
            std::fs::create_dir_all(&temp_dir).unwrap();
            
            // Ensure the variable is not set in the shell environment
            unsafe {
                std::env::remove_var(&var_name);
            }
            
            // Create a .env file with the variable
            let dotenv_path = temp_dir.join(".env");
            std::fs::write(&dotenv_path, format!("{}={}", var_name, var_value)).unwrap();
            
            // Load .env file from the specific path (this is what from_env() does)
            dotenv::from_path(&dotenv_path).ok();
            
            // Verify the variable is now accessible
            let actual_value = std::env::var(&var_name);
            prop_assert!(actual_value.is_ok(), 
                "Variable from .env file should be accessible via std::env::var");
            prop_assert_eq!(actual_value.unwrap(), var_value, 
                "Variable value from .env file should match the written value");
            
            // Cleanup
            unsafe {
                std::env::remove_var(&var_name);
            }
            std::fs::remove_dir_all(&temp_dir).ok();
        }
    }
}
