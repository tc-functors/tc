# Integration Tests for AWS Bedrock Integration

This directory contains integration tests for the AWS Bedrock and Anthropic provider implementations. These tests make actual API calls and are marked with `#[ignore]` to prevent them from running during normal test runs.

## Test Files

### `integration_bedrock.rs`
Tests for AWS Bedrock provider functionality:
- Actual Bedrock API calls with default credentials
- AWS SSO authentication flow
- Profile-based authentication
- Region configuration
- Different model variants
- Error handling
- Code extraction from responses

**Requirements tested:** 1.1, 1.2, 1.3, 1.4, 1.5

### `integration_anthropic.rs`
Tests for Anthropic provider functionality:
- Actual Anthropic API calls
- Default and alternative model support
- Backward compatibility with existing implementation
- Error handling (missing/invalid API keys)
- Code extraction from responses
- .env file loading

**Requirements tested:** 8.1, 8.2, 8.3, 8.5, 9.2

### `integration_e2e.rs`
End-to-end scaffolding workflow tests:
- Complete scaffold workflow with Bedrock
- Complete scaffold workflow with Anthropic
- Different application types (WebSocket, async queues, event-driven)
- Provider selection based on configuration
- Topology file validation
- Backward compatibility verification

**Requirements tested:** 1.1, 8.1, 8.3

## Prerequisites

### For Bedrock Tests

You need valid AWS credentials configured through one of these methods:

1. **AWS SSO** (recommended):
   ```bash
   aws configure sso
   aws sso login --profile <your-profile>
   ```

2. **Environment variables**:
   ```bash
   export AWS_ACCESS_KEY_ID=your-access-key
   export AWS_SECRET_ACCESS_KEY=your-secret-key
   export AWS_REGION=us-east-1
   ```

3. **AWS credentials file** (`~/.aws/credentials`):
   ```ini
   [default]
   aws_access_key_id = your-access-key
   aws_secret_access_key = your-secret-key
   ```

4. **AWS config file** (`~/.aws/config`):
   ```ini
   [default]
   region = us-east-1
   ```

**IAM Permissions Required:**
- `bedrock:InvokeModel` for the Claude models you want to test
- Access to AWS Bedrock in your configured region

### For Anthropic Tests

You need a valid Anthropic API key:

```bash
export CLAUDE_API_KEY=sk-ant-your-api-key-here
```

Or create a `.env` file in the project root:
```
CLAUDE_API_KEY=sk-ant-your-api-key-here
```

## Running the Tests

### Run All Integration Tests

```bash
# Run all integration tests (requires both AWS and Anthropic credentials)
cargo test --package scaffolder --test integration_bedrock -- --ignored
cargo test --package scaffolder --test integration_anthropic -- --ignored
cargo test --package scaffolder --test integration_e2e -- --ignored
```

### Run Specific Test Files

```bash
# Run only Bedrock integration tests
cargo test --package scaffolder --test integration_bedrock -- --ignored

# Run only Anthropic integration tests
cargo test --package scaffolder --test integration_anthropic -- --ignored

# Run only end-to-end tests
cargo test --package scaffolder --test integration_e2e -- --ignored
```

### Run Specific Tests

```bash
# Run a specific test by name
cargo test --package scaffolder --test integration_bedrock test_bedrock_actual_api_call_default_credentials -- --ignored

# Run tests matching a pattern
cargo test --package scaffolder --test integration_bedrock sso -- --ignored
```

### Run with Output

```bash
# Show println! output even for passing tests
cargo test --package scaffolder --test integration_bedrock -- --ignored --nocapture
```

## Test Configuration

### Environment Variables

You can configure tests using environment variables:

- `AWS_PROFILE` - AWS profile to use for SSO tests
- `TEST_AWS_PROFILE` - AWS profile to use for profile-based tests
- `AWS_REGION` - AWS region for Bedrock tests
- `CLAUDE_API_KEY` - Anthropic API key

Example:
```bash
AWS_PROFILE=my-sso-profile cargo test --package scaffolder --test integration_bedrock test_bedrock_with_sso_authentication -- --ignored
```

## Cost Considerations

⚠️ **Warning:** These tests make actual API calls which may incur costs:

- **AWS Bedrock**: Charges per token (input and output)
- **Anthropic API**: Charges per token (input and output)

The tests are designed to use minimal prompts to keep costs low, but be aware that running the full test suite multiple times will accumulate charges.

Estimated cost per full test run:
- Bedrock tests: ~$0.10-0.50 (depending on region and model)
- Anthropic tests: ~$0.10-0.50
- E2E tests: ~$0.20-1.00

## Troubleshooting

### Bedrock Tests Fail with "AccessDeniedException"

1. Verify your AWS credentials are configured correctly
2. Check that you have `bedrock:InvokeModel` permission
3. Verify that Bedrock is available in your region
4. If using SSO, ensure you're logged in: `aws sso login --profile <profile>`

### Anthropic Tests Fail with "401 Unauthorized"

1. Verify `CLAUDE_API_KEY` is set correctly
2. Check that your API key is valid and not expired
3. Ensure you have sufficient API credits

### Tests Timeout

1. Check your internet connection
2. Verify the API endpoints are accessible
3. Try increasing the timeout (tests use default timeouts)
4. Check if there are any rate limits being hit

### "Model not available" Errors

1. Verify the model ID is correct for your provider
2. For Bedrock: Check that the model is available in your region
3. For Anthropic: Check that you have access to the specified model

## CI/CD Integration

These tests are marked with `#[ignore]` to prevent them from running in normal CI pipelines. To run them in CI:

1. Configure AWS credentials as secrets
2. Configure Anthropic API key as a secret
3. Run tests explicitly with the `--ignored` flag

Example GitHub Actions workflow:
```yaml
- name: Run Integration Tests
  env:
    AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
    AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
    AWS_REGION: us-east-1
    CLAUDE_API_KEY: ${{ secrets.CLAUDE_API_KEY }}
  run: |
    cargo test --package scaffolder --test integration_bedrock -- --ignored
    cargo test --package scaffolder --test integration_anthropic -- --ignored
    cargo test --package scaffolder --test integration_e2e -- --ignored
```

## Development Tips

1. **Start with unit tests**: Run unit tests first to catch basic issues
   ```bash
   cargo test --package scaffolder --lib
   ```

2. **Test one provider at a time**: Start with the provider you have credentials for

3. **Use specific tests during development**: Run individual tests to save time and costs

4. **Check test output**: Use `--nocapture` to see detailed output and responses

5. **Clean up test artifacts**: The e2e tests create temporary directories in `/tmp/tc-test-*`

## Contributing

When adding new integration tests:

1. Mark them with `#[ignore]` and appropriate reason
2. Document the requirements in the test docstring
3. Keep prompts minimal to reduce API costs
4. Add appropriate assertions to verify behavior
5. Clean up any resources created during the test
6. Update this README with any new prerequisites or configuration
