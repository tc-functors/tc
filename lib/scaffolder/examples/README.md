# TC Scaffolder Configuration Examples

This directory contains example `config.toml` files for various TC scaffolder configurations. Choose the example that best matches your setup and copy it to your project root as `config.toml`.

## Available Examples

### AWS Bedrock Configurations

#### 1. `config-bedrock-sso.toml` - AWS Bedrock with SSO (Recommended for Organizations)

**Use when:**
- Your organization uses AWS IAM Identity Center (SSO)
- You want centralized authentication management
- You need temporary credentials with automatic rotation

**Setup:**
```bash
# Configure SSO
aws configure sso

# Login
aws sso login --profile my-company-sso

# Copy and customize the example
cp lib/scaffolder/examples/config-bedrock-sso.toml config.toml
# Edit config.toml to set your profile name

# Use TC
tc scaffold my-app
```

**Key features:**
- Centralized authentication
- Automatic credential rotation
- Multi-account support
- Enhanced security

---

#### 2. `config-bedrock-profile.toml` - AWS Bedrock with Named Profile

**Use when:**
- You have multiple AWS accounts or roles
- You use long-lived credentials in `~/.aws/credentials`
- You want to switch between different AWS configurations

**Setup:**
```bash
# Configure credentials
cat >> ~/.aws/credentials << EOF
[production]
aws_access_key_id = YOUR_ACCESS_KEY
aws_secret_access_key = YOUR_SECRET_KEY
EOF

# Copy and customize the example
cp lib/scaffolder/examples/config-bedrock-profile.toml config.toml
# Edit config.toml to set your profile name

# Use TC
tc scaffold my-app
```

**Key features:**
- Multiple profile support
- Persistent credentials
- Easy account switching

---

#### 3. `config-bedrock-default.toml` - AWS Bedrock with Default Credentials (Simplest)

**Use when:**
- You have AWS credentials already configured
- You want the simplest possible setup
- You're okay with default credential resolution

**Setup:**
```bash
# Ensure AWS credentials are configured (any method)
aws sts get-caller-identity

# Copy the example
cp lib/scaffolder/examples/config-bedrock-default.toml config.toml

# Use TC
tc scaffold my-app
```

**Key features:**
- Minimal configuration
- Automatic credential discovery
- Works with any AWS credential source

---

### Anthropic API Configuration

#### 4. `config-anthropic.toml` - Anthropic Direct API

**Use when:**
- You prefer simpler setup without AWS infrastructure
- You don't have AWS credentials
- You want to use the latest Anthropic models directly

**Setup:**
```bash
# Get API key from https://console.anthropic.com

# Create .env file
cat > .env << EOF
CLAUDE_API_KEY=sk-ant-your-api-key-here
EOF

# Add .env to .gitignore
echo ".env" >> .gitignore

# Copy the example
cp lib/scaffolder/examples/config-anthropic.toml config.toml

# Use TC
tc scaffold my-app
```

**Key features:**
- Simple setup
- No AWS account needed
- Direct access to latest models
- Lower latency (no AWS proxy)

---

### Multi-Environment Configuration

#### 5. `config-multi-environment.toml` - Multi-Environment Setup

**Use when:**
- You work across multiple environments (dev, staging, prod)
- You want different models for different environments
- You need cost optimization for development

**Setup:**
```bash
# Copy the example
cp lib/scaffolder/examples/config-multi-environment.toml config.toml

# Development (fast, cheap)
tc scaffold my-app --llm-model anthropic.claude-3-haiku-20240307-v1:0

# Staging (balanced)
tc scaffold my-app --aws-profile staging

# Production (most capable)
tc scaffold my-app --aws-profile production --llm-model anthropic.claude-3-opus-20240229-v1:0
```

**Key features:**
- Environment-specific configurations
- Cost optimization
- Flexible overrides

---

## Quick Start Guide

### For AWS Bedrock Users (Recommended)

1. **Choose your authentication method:**
   - SSO: Use `config-bedrock-sso.toml`
   - Named profile: Use `config-bedrock-profile.toml`
   - Default credentials: Use `config-bedrock-default.toml`

2. **Copy the example:**
   ```bash
   cp lib/scaffolder/examples/config-bedrock-sso.toml config.toml
   ```

3. **Customize the configuration:**
   - Edit `config.toml`
   - Set your profile name, region, and preferred model

4. **Verify AWS access:**
   ```bash
   aws sts get-caller-identity
   aws bedrock list-foundation-models --region us-east-1
   ```

5. **Use TC:**
   ```bash
   tc scaffold my-app
   ```

### For Anthropic API Users

1. **Get your API key:**
   - Sign up at https://console.anthropic.com
   - Generate an API key

2. **Copy the example:**
   ```bash
   cp lib/scaffolder/examples/config-anthropic.toml config.toml
   ```

3. **Set your API key:**
   ```bash
   echo "CLAUDE_API_KEY=sk-ant-your-api-key" > .env
   echo ".env" >> .gitignore
   ```

4. **Use TC:**
   ```bash
   tc scaffold my-app
   ```

---

## Configuration Precedence

TC resolves configuration in this order (highest to lowest priority):

1. **CLI Parameters** - Explicit command-line arguments
   ```bash
   tc scaffold my-app --llm-provider bedrock --aws-region us-west-2
   ```

2. **Environment Variables** - Shell environment or `.env` file
   ```bash
   export TC_LLM_PROVIDER=bedrock
   export AWS_REGION=us-west-2
   ```

3. **Config File** - `config.toml` in the current directory
   ```toml
   [llm]
   provider = "bedrock"
   
   [llm.aws]
   region = "us-west-2"
   ```

4. **Defaults** - Built-in defaults
   - Provider: `bedrock`
   - Model: Provider-specific default
   - Region: AWS SDK default

---

## Common Customizations

### Change the Model

Edit the `model` field in your `config.toml`:

**For Bedrock:**
```toml
[llm]
model = "anthropic.claude-3-opus-20240229-v1:0"  # Most capable
# or
model = "anthropic.claude-3-haiku-20240307-v1:0"  # Fastest, cheapest
```

**For Anthropic:**
```toml
[llm]
model = "claude-3-opus-20240229"  # Most capable
# or
model = "claude-3-haiku-20240307"  # Fastest, cheapest
```

### Change the Region

Edit the `region` field in your `config.toml`:

```toml
[llm.aws]
region = "us-west-2"  # or eu-west-1, ap-southeast-1, etc.
```

### Switch Between Providers

Change the `provider` field:

```toml
[llm]
provider = "anthropic"  # or "bedrock"
```

Or use CLI override:
```bash
tc scaffold my-app --llm-provider anthropic
```

---

## Environment-Specific Configurations

### Development Environment

**Goal:** Fast iteration, low cost

```toml
[llm]
provider = "bedrock"
model = "anthropic.claude-3-haiku-20240307-v1:0"  # Fastest, cheapest

[llm.aws]
region = "us-east-1"
profile = "development"
```

### Production Environment

**Goal:** Best quality, reliability

```toml
[llm]
provider = "bedrock"
model = "anthropic.claude-3-opus-20240229-v1:0"  # Most capable

[llm.aws]
region = "us-west-2"
profile = "production"
```

### CI/CD Environment

**Goal:** Automated, secure

Use environment variables instead of config file:

```bash
export TC_LLM_PROVIDER=bedrock
export AWS_ACCESS_KEY_ID=${{ secrets.AWS_ACCESS_KEY_ID }}
export AWS_SECRET_ACCESS_KEY=${{ secrets.AWS_SECRET_ACCESS_KEY }}
export AWS_REGION=us-east-1
```

---

## Troubleshooting

### Configuration Not Applied

1. Check configuration precedence (CLI > Env > File > Defaults)
2. Verify file location: `config.toml` must be in current directory
3. Check file syntax: Use a TOML validator
4. Verify environment variables: `env | grep TC_LLM`

### AWS Authentication Issues

1. Verify credentials: `aws sts get-caller-identity`
2. Check IAM permissions: Ensure `bedrock:InvokeModel` permission
3. For SSO: Re-login with `aws sso login --profile <profile>`
4. Check region availability: `aws bedrock list-foundation-models --region <region>`

### Anthropic API Issues

1. Verify API key: `echo $CLAUDE_API_KEY`
2. Check `.env` file exists and is loaded
3. Ensure `.env` is not in `.gitignore` before committing (wait, it should be!)
4. Test API key with curl:
   ```bash
   curl https://api.anthropic.com/v1/messages \
     -H "x-api-key: $CLAUDE_API_KEY" \
     -H "anthropic-version: 2023-06-01" \
     -H "content-type: application/json" \
     -d '{"model":"claude-3-haiku-20240307","max_tokens":100,"messages":[{"role":"user","content":"Hello"}]}'
   ```

---

## Security Best Practices

1. **Never commit secrets:**
   ```bash
   echo ".env" >> .gitignore
   echo "config.toml" >> .gitignore  # If it contains secrets
   ```

2. **Use environment variables for secrets:**
   - Store API keys in `.env` or environment
   - Don't put API keys in `config.toml`

3. **Rotate credentials regularly:**
   - AWS: Rotate access keys every 90 days
   - Anthropic: Rotate API keys periodically

4. **Use least privilege:**
   - Grant only `bedrock:InvokeModel` permission
   - Restrict to specific model ARNs if possible

5. **Use SSO when possible:**
   - Centralized authentication
   - Automatic credential rotation
   - Better audit trail

---

## Additional Resources

- [TC Scaffolder Documentation](../README.md)
- [AWS Bedrock Documentation](https://docs.aws.amazon.com/bedrock/)
- [Anthropic API Documentation](https://docs.anthropic.com/)
- [AWS CLI Configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html)
- [AWS SSO Setup](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-sso.html)

---

## Contributing

Found an issue or have a suggestion for these examples? Please open an issue or pull request at [github.com/tc-functors/tc](https://github.com/tc-functors/tc).
