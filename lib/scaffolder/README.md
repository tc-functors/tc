# TC Scaffolder - LLM-Assisted Topology Generation

The TC scaffolder module enables AI-assisted generation of topology YAML files using Large Language Models (LLMs). It supports both AWS Bedrock and Anthropic's direct API, allowing you to leverage Claude models through your preferred infrastructure.

## Table of Contents

- [Quick Start](#quick-start)
- [Configuration](#configuration)
  - [Environment Variables](#environment-variables)
  - [Configuration File](#configuration-file)
  - [CLI Parameters](#cli-parameters)
  - [Configuration Precedence](#configuration-precedence)
- [AWS Bedrock Setup](#aws-bedrock-setup)
  - [Prerequisites](#prerequisites)
  - [AWS SSO Authentication](#aws-sso-authentication)
  - [Profile-Based Authentication](#profile-based-authentication)
  - [Environment Variable Authentication](#environment-variable-authentication)
- [Anthropic API Setup](#anthropic-api-setup)
- [Migration Guide](#migration-guide)
- [Supported Models](#supported-models)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Quick Start

By default, TC uses AWS Bedrock with your default AWS credentials:

```bash
# Generate a topology using AWS Bedrock (default)
tc scaffold my-app

# Or explicitly specify Bedrock
tc scaffold my-app --llm-provider bedrock

# Use Anthropic API instead
tc scaffold my-app --llm-provider anthropic
```

## Configuration

TC supports multiple configuration methods with a clear precedence hierarchy: **CLI > Environment Variables > Config File > Defaults**.

### Environment Variables

Set these environment variables to configure the LLM provider:

| Variable | Description | Example | Default |
|----------|-------------|---------|---------|
| `TC_LLM_PROVIDER` | LLM provider to use | `bedrock` or `anthropic` | `bedrock` |
| `TC_LLM_MODEL` | Model identifier | `us.anthropic.claude-opus-4-5-20251101-v1:0` | Provider-specific default |
| `AWS_REGION` | AWS region for Bedrock | `us-east-1` | AWS SDK default |
| `AWS_PROFILE` | AWS profile name | `my-profile` | AWS SDK default |
| `CLAUDE_API_KEY` | Anthropic API key (required for Anthropic provider) | `sk-ant-...` | None |

**Example:**

```bash
export TC_LLM_PROVIDER=bedrock
export TC_LLM_MODEL=us.anthropic.claude-opus-4-5-20251101-v1:0
export AWS_REGION=us-west-2
export AWS_PROFILE=my-sso-profile

tc scaffold my-app
```

### Configuration File

Create a `config.toml` file in your project directory:

```toml
[llm]
provider = "bedrock"  # or "anthropic"
model = "anthropic.claude-opus-4-5-20251101-v1:0"

[llm.aws]
region = "us-east-1"
profile = "my-profile"
```

See the [examples](#examples) section for complete configuration file examples.

### CLI Parameters

Override any configuration using command-line parameters:

```bash
tc scaffold my-app \
  --llm-provider bedrock \
  --llm-model us.anthropic.claude-sonnet-4-5-20250929-v1:0\
  --aws-region us-west-2 \
  --aws-profile my-sso-profile
```

### Configuration Precedence

Configuration is resolved in this order (highest to lowest priority):

1. **CLI Parameters** - Explicit command-line arguments
2. **Environment Variables** - Shell environment or `.env` file
3. **Config File** - `config.toml` in the current directory
4. **Defaults** - Bedrock provider with sensible defaults

**Example:** If you set `TC_LLM_PROVIDER=anthropic` in your environment but pass `--llm-provider bedrock` on the CLI, Bedrock will be used.

## AWS Bedrock Setup

AWS Bedrock is the default provider and integrates seamlessly with AWS infrastructure.

### Prerequisites

1. **AWS Account** with Bedrock access
2. **IAM Permissions**: Your AWS credentials must have the `bedrock:InvokeModel` permission
3. **Model Access**: Enable Claude models in the AWS Bedrock console for your region
4. **AWS CLI** (optional but recommended): Install from [aws.amazon.com/cli](https://aws.amazon.com/cli/)

### AWS SSO Authentication

AWS SSO (Single Sign-On) is the recommended authentication method for organizations using AWS IAM Identity Center.

#### Initial Setup

1. Configure AWS SSO:

```bash
aws configure sso
```

Follow the prompts to:
- Enter your SSO start URL
- Select your AWS region
- Choose your AWS account
- Select your IAM role
- Set a profile name (e.g., `my-sso-profile`)

2. Login to AWS SSO:

```bash
aws sso login --profile my-sso-profile
```

This will open your browser for authentication.

#### Using SSO with TC

**Option 1: Set AWS_PROFILE environment variable**

```bash
export AWS_PROFILE=my-sso-profile
tc scaffold my-app
```

**Option 2: Use CLI parameter**

```bash
tc scaffold my-app --aws-profile my-sso-profile
```

**Option 3: Set in config.toml**

```toml
[llm.aws]
profile = "my-sso-profile"
```

#### SSO Session Management

SSO sessions expire after a period (typically 8-12 hours). When your session expires:

```bash
# Re-authenticate
aws sso login --profile my-sso-profile

# Verify your credentials
aws sts get-caller-identity --profile my-sso-profile
```

### Profile-Based Authentication

Use named profiles from your AWS credentials file for different accounts or roles.

#### Setup

1. Configure your AWS credentials file (`~/.aws/credentials`):

```ini
[default]
aws_access_key_id = YOUR_ACCESS_KEY
aws_secret_access_key = YOUR_SECRET_KEY

[production]
aws_access_key_id = PROD_ACCESS_KEY
aws_secret_access_key = PROD_SECRET_KEY

[development]
aws_access_key_id = DEV_ACCESS_KEY
aws_secret_access_key = DEV_SECRET_KEY
```

2. Configure regions in `~/.aws/config`:

```ini
[default]
region = us-east-1

[profile production]
region = us-west-2

[profile development]
region = us-east-1
```

#### Using Profiles with TC

```bash
# Use a specific profile
tc scaffold my-app --aws-profile production

# Or set via environment
export AWS_PROFILE=production
tc scaffold my-app
```

### Environment Variable Authentication

For CI/CD pipelines or temporary credentials:

```bash
export AWS_ACCESS_KEY_ID=your-access-key
export AWS_SECRET_ACCESS_KEY=your-secret-key
export AWS_SESSION_TOKEN=your-session-token  # Optional, for temporary credentials
export AWS_REGION=us-east-1

tc scaffold my-app
```

### Verifying AWS Credentials

Before using TC, verify your AWS credentials are working:

```bash
# Check your identity
aws sts get-caller-identity

# Test Bedrock access (requires AWS CLI with Bedrock support)
aws bedrock-runtime invoke-model \
  --model-id anthropic.claude-3-5-sonnet-20241022-v2:0 \
  --body '{"anthropic_version":"bedrock-2023-05-31","messages":[{"role":"user","content":[{"type":"text","text":"Hello"}]}],"max_tokens":100}' \
  --cli-binary-format raw-in-base64-out \
  output.json
```

## Anthropic API Setup

Use Anthropic's direct API for simpler setup without AWS infrastructure.

### Prerequisites

1. **Anthropic Account**: Sign up at [console.anthropic.com](https://console.anthropic.com)
2. **API Key**: Generate an API key from the Anthropic console

### Configuration

**Option 1: Environment variable**

```bash
export TC_LLM_PROVIDER=anthropic
export CLAUDE_API_KEY=sk-ant-your-api-key-here

tc scaffold my-app
```

**Option 2: .env file**

Create a `.env` file in your project directory:

```env
TC_LLM_PROVIDER=anthropic
CLAUDE_API_KEY=sk-ant-your-api-key-here
```

**Option 3: config.toml**

```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
```

Then set the API key via environment:

```bash
export CLAUDE_API_KEY=sk-ant-your-api-key-here
```

**Note:** For security reasons, API keys should not be stored in `config.toml`. Use environment variables or `.env` files instead.

### Using .env Files

TC automatically loads environment variables from a `.env` file in the current directory:

1. Create a `.env` file:

```env
CLAUDE_API_KEY=sk-ant-your-api-key-here
TC_LLM_PROVIDER=anthropic
TC_LLM_MODEL=claude-sonnet-4-5-20250929
```

2. Add `.env` to your `.gitignore`:

```bash
echo ".env" >> .gitignore
```

3. Run TC normally:

```bash
tc scaffold my-app
```

**Important:** Shell environment variables take precedence over `.env` file values.

## Migration Guide

### Migrating from Anthropic API to AWS Bedrock

If you're currently using the Anthropic API and want to switch to AWS Bedrock:

#### Before (Anthropic API)

```bash
export CLAUDE_API_KEY=sk-ant-your-api-key
tc scaffold my-app
```

#### After (AWS Bedrock)

**Option 1: Use defaults (simplest)**

```bash
# Remove or unset the provider override
unset TC_LLM_PROVIDER

# TC will now use Bedrock by default
tc scaffold my-app
```

**Option 2: Explicit configuration**

```bash
export TC_LLM_PROVIDER=bedrock
export AWS_PROFILE=my-sso-profile  # If using SSO
export AWS_REGION=us-east-1

tc scaffold my-app
```

**Option 3: Update config.toml**

```toml
[llm]
provider = "bedrock"
model = "us.anthropic.claude-sonnet-4-5-20250929-v1:0"

[llm.aws]
region = "us-east-1"
profile = "my-sso-profile"
```

### Maintaining Backward Compatibility

To continue using the Anthropic API after upgrading TC:

**Option 1: Environment variable**

```bash
export TC_LLM_PROVIDER=anthropic
export CLAUDE_API_KEY=sk-ant-your-api-key
```

**Option 2: CLI parameter**

```bash
tc scaffold my-app --llm-provider anthropic
```

**Option 3: config.toml**

```toml
[llm]
provider = "anthropic"
```

### Switching Between Providers

You can easily switch between providers for different projects:

```bash
# Project A: Use Bedrock with SSO
cd project-a
export AWS_PROFILE=my-sso-profile
tc scaffold app-a

# Project B: Use Anthropic API
cd ../project-b
export TC_LLM_PROVIDER=anthropic
export CLAUDE_API_KEY=sk-ant-...
tc scaffold app-b
```

Or use project-specific `config.toml` files:

```bash
# project-a/config.toml
[llm]
provider = "bedrock"

[llm.aws]
profile = "my-sso-profile"
```

```bash
# project-b/config.toml
[llm]
provider = "anthropic"
```

## Supported Models

### AWS Bedrock Models

Bedrock uses a specific model ID format: `anthropic.claude-{version}-v{api-version}:{variant}`

#### Claude 4 Series (Latest)

| Model | Model ID | Description |
|-------|----------|-------------|
| **Claude Opus 4.5 (Default)** | `us.anthropic.claude-opus-4-5-20251101-v1:0` | Most capable model, highest quality output |
| Claude Sonnet 4.5 | `us.anthropic.claude-sonnet-4-5-20250929-v1:0` | Excellent balance of speed and capability |
| Claude Haiku 4.5 | `us.anthropic.claude-haiku-4-5-20250929-v1:0` | Fastest Claude 4, optimized for speed |

#### Claude 3.5 Series

| Model | Model ID | Description |
|-------|----------|-------------|
| Claude 3.5 Sonnet v2 | `us.anthropic.claude-3-5-sonnet-20241022-v2:0` | Latest Claude 3.5, excellent balance |
| Claude 3.5 Sonnet v1 | `us.anthropic.claude-3-5-sonnet-20240620-v1:0` | Previous version of Claude 3.5 |

#### Claude 3 Series

| Model | Model ID | Description |
|-------|----------|-------------|
| Claude 3 Opus | `us.anthropic.claude-3-opus-20240229-v1:0` | Most capable Claude 3 |
| Claude 3 Sonnet | `us.anthropic.claude-3-sonnet-20240229-v1:0` | Balanced Claude 3 |
| Claude 3 Haiku | `us.anthropic.claude-3-haiku-20240307-v1:0` | Fastest, lowest cost |

**Note:** Model availability varies by AWS region. Check the [AWS Bedrock documentation](https://docs.aws.amazon.com/bedrock/latest/userguide/models-supported.html) for your region.

### Anthropic API Models

Anthropic uses a simpler model ID format: `claude-{version}-{date}`

#### Claude 4 Series (Latest)

| Model | Model ID | Description |
|-------|----------|-------------|
| **Claude Sonnet 4.5 (Default)** | `claude-sonnet-4-5-20250929` | Latest Claude model, excellent performance |
| Claude Opus 4.5 | `claude-opus-4-5-20251101` | Highest capability Claude 4 |
| Claude Haiku 4.5 | `claude-haiku-4-5-20250929` | Fastest Claude 4, optimized for speed |

#### Claude 3.5 Series

| Model | Model ID | Description |
|-------|----------|-------------|
| Claude 3.5 Sonnet v2 | `claude-3-5-sonnet-20241022` | Latest Claude 3.5 |
| Claude 3.5 Sonnet v1 | `claude-3-5-sonnet-20240620` | Previous Claude 3.5 |

#### Claude 3 Series

| Model | Model ID | Description |
|-------|----------|-------------|
| Claude 3 Opus | `claude-3-opus-20240229` | Most capable Claude 3 |
| Claude 3 Sonnet | `claude-3-sonnet-20240229` | Balanced Claude 3 |
| Claude 3 Haiku | `claude-3-haiku-20240307` | Fastest Claude 3 |

### Specifying a Model

```bash
# Bedrock - Use Claude Opus 4.5 (default)
tc scaffold my-app --llm-provider bedrock

# Bedrock - Use a different model
tc scaffold my-app \
  --llm-provider bedrock \
  --llm-model us.anthropic.claude-3-5-sonnet-20241022-v2:0

# Anthropic - Use Claude Sonnet 4.5 (default)
tc scaffold my-app --llm-provider anthropic

# Anthropic - Use a different model
tc scaffold my-app \
  --llm-provider anthropic \
  --llm-model claude-opus-4-5-20251101
```

### Adding New Models

When new Claude models are released, you can use them immediately without updating TC:

#### For AWS Bedrock

1. **Check model availability** in your AWS region:
   ```bash
   aws bedrock list-foundation-models --region us-east-1 \
     --by-provider anthropic
   ```

2. **Enable model access** in the AWS Bedrock console:
   - Navigate to AWS Bedrock → Model access
   - Request access to the new model

3. **Use the new model** with TC:
   ```bash
   tc scaffold my-app \
     --llm-provider bedrock \
     --llm-model us.anthropic.claude-{new-model-id}
   ```

4. **Set as default** (optional) in your config.toml:
   ```toml
   [llm]
   provider = "bedrock"
   model = "us.anthropic.claude-{new-model-id}"
   ```

#### For Anthropic API

1. **Check model availability** in the [Anthropic documentation](https://docs.anthropic.com/en/docs/models-overview)

2. **Use the new model** with TC:
   ```bash
   tc scaffold my-app \
     --llm-provider anthropic \
     --llm-model claude-{new-model-id}
   ```

3. **Set as default** (optional) in your config.toml:
   ```toml
   [llm]
   provider = "anthropic"
   model = "claude-{new-model-id}"
   ```

#### Model ID Format Reference

- **Bedrock format**: `us.anthropic.claude-{version}-{date}-v{api-version}:{variant}`
  - Example: `us.anthropic.claude-opus-4-5-20251101-v1:0`
  - The `v{api-version}:{variant}` suffix is required by Bedrock

- **Anthropic format**: `claude-{version}-{date}`
  - Example: `claude-opus-4-5-20251101`
  - Simpler format without API version suffix

**Tip:** When a new model is released, check both the Anthropic and AWS Bedrock documentation, as there may be a delay before new models are available in Bedrock.

## Examples

### Example 1: Bedrock with SSO (Recommended for Organizations)

**config.toml:**

```toml
[llm]
provider = "bedrock"
model = "us.anthropic.claude-opus-4-5-20251101-v1:0"

[llm.aws]
region = "us-east-1"
profile = "my-company-sso"
```

**Usage:**

```bash
# Login to SSO
aws sso login --profile my-company-sso

# Generate topology
tc scaffold my-app
```

### Example 2: Bedrock with Named Profile

**config.toml:**

```toml
[llm]
provider = "bedrock"
model = "us.anthropic.claude-opus-4-5-20251101-v1:0"

[llm.aws]
region = "us-west-2"
profile = "production"
```

**~/.aws/credentials:**

```ini
[production]
aws_access_key_id = YOUR_ACCESS_KEY
aws_secret_access_key = YOUR_SECRET_KEY
```

**Usage:**

```bash
tc scaffold my-app
```

### Example 3: Anthropic API with .env File

**config.toml:**

```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-5-20250929"
```

**.env:**

```env
CLAUDE_API_KEY=sk-ant-your-api-key-here
```

**.gitignore:**

```
.env
```

**Usage:**

```bash
tc scaffold my-app
```

### Example 4: Multi-Environment Setup

**Development (config.toml):**

```toml
[llm]
provider = "bedrock"
model = "us.anthropic.claude-3-haiku-20240307-v1:0"  # Faster, cheaper for dev

[llm.aws]
region = "us-east-1"
profile = "dev-profile"
```

**Production (override with CLI):**

```bash
tc scaffold my-app \
  --llm-model us.anthropic.claude-opus-4-5-20251101-v1:0 \
  --aws-profile prod-profile
```

### Example 5: CI/CD Pipeline

**GitHub Actions:**

```yaml
name: Generate Topology

on: [push]

jobs:
  scaffold:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Configure AWS Credentials
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1
      
      - name: Install TC
        run: |
          # Install TC (adjust for your installation method)
          cargo install tc
      
      - name: Generate Topology
        run: |
          tc scaffold my-app --llm-provider bedrock
```

## Troubleshooting

### Common Issues

#### "Authentication failed" with Bedrock

**Symptoms:**
```
Error: Authentication failed: Access denied
```

**Solutions:**

1. Verify AWS credentials are configured:
   ```bash
   aws sts get-caller-identity
   ```

2. Check IAM permissions - ensure you have `bedrock:InvokeModel`:
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Effect": "Allow",
         "Action": "bedrock:InvokeModel",
         "Resource": "arn:aws:bedrock:*::foundation-model/anthropic.claude-*"
       }
     ]
   }
   ```

3. If using SSO, ensure you're logged in:
   ```bash
   aws sso login --profile my-profile
   ```

4. Verify Bedrock is available in your region:
   ```bash
   aws bedrock list-foundation-models --region us-east-1
   ```

#### "Model not available" Error

**Symptoms:**
```
Error: Model not available: Model not ready
```

**Solutions:**

1. Check model ID format:
   - Bedrock: `us.anthropic.claude-3-5-sonnet-20241022-v2:0`
   - Anthropic: `claude-sonnet-4-5-20250929`

2. Verify model access in AWS Bedrock console:
   - Go to AWS Bedrock console
   - Navigate to "Model access"
   - Enable access to Claude models

3. Check regional availability:
   - Not all models are available in all regions
   - Try `us-east-1` or `us-west-2` which have broad model support

#### "CLAUDE_API_KEY not set" with Anthropic

**Symptoms:**
```
Error: Configuration error: CLAUDE_API_KEY not set
```

**Solutions:**

1. Set the environment variable:
   ```bash
   export CLAUDE_API_KEY=sk-ant-your-api-key
   ```

2. Or create a `.env` file:
   ```env
   CLAUDE_API_KEY=sk-ant-your-api-key
   ```

3. Verify the variable is set:
   ```bash
   echo $CLAUDE_API_KEY
   ```

#### Configuration Not Being Applied

**Symptoms:** TC uses different settings than expected

**Solutions:**

1. Check configuration precedence: CLI > Env > File > Defaults

2. Verify environment variables:
   ```bash
   env | grep TC_LLM
   env | grep AWS
   ```

3. Check for `.env` file in current directory:
   ```bash
   cat .env
   ```

4. Verify `config.toml` location and syntax:
   ```bash
   cat config.toml
   ```

5. Use CLI parameters to override:
   ```bash
   tc scaffold my-app --llm-provider bedrock --aws-region us-east-1
   ```

#### Network or Timeout Errors

**Symptoms:**
```
Error: Network error: connection timeout
```

**Solutions:**

1. Check internet connectivity

2. Verify firewall/proxy settings allow HTTPS to:
   - AWS Bedrock: `bedrock-runtime.{region}.amazonaws.com`
   - Anthropic: `api.anthropic.com`

3. Check AWS service health:
   - Visit [AWS Service Health Dashboard](https://status.aws.amazon.com/)

4. Try a different region (for Bedrock):
   ```bash
   tc scaffold my-app --aws-region us-west-2
   ```

### Getting Help

If you encounter issues not covered here:

1. **Check the logs**: TC provides detailed error messages with hints
2. **Verify credentials**: Use AWS CLI or curl to test API access directly
3. **Review configuration**: Double-check all configuration files and environment variables
4. **Open an issue**: Report bugs at [github.com/tc-functors/tc/issues](https://github.com/tc-functors/tc/issues)
5. **Join discussions**: Ask questions at [github.com/orgs/tc-functors/discussions](https://github.com/orgs/tc-functors/discussions)

### Debug Mode

For detailed debugging information:

```bash
# Enable Rust logging
export RUST_LOG=debug
tc scaffold my-app

# Or for specific modules
export RUST_LOG=scaffolder=debug
tc scaffold my-app
```

## Security Best Practices

1. **Never commit API keys**: Add `.env` to `.gitignore`
2. **Use IAM roles**: Prefer IAM roles over access keys when possible
3. **Rotate credentials**: Regularly rotate API keys and access keys
4. **Principle of least privilege**: Grant only necessary permissions
5. **Use SSO**: Leverage AWS SSO for centralized authentication
6. **Audit access**: Monitor CloudTrail logs for Bedrock API usage

## Cost Optimization

1. **Choose appropriate models**:
   - Development: Use Haiku (fastest, cheapest)
   - Production: Use Sonnet (balanced)
   - Complex tasks: Use Opus (most capable)

2. **Monitor usage**:
   - AWS Bedrock: Check AWS Cost Explorer
   - Anthropic: Monitor usage in Anthropic console

3. **Cache results**: Reuse generated topologies when possible

4. **Use defaults**: Default models are chosen for good cost/performance balance

## Additional Resources

- [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock/)
- [Anthropic API Documentation](https://docs.anthropic.com/)
- [TC Documentation](https://tc-functors.org/)
- [AWS CLI Configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
- [AWS SSO Setup](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html)

## License

This module is part of the TC project. See the main repository for license information.
