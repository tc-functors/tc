---
name: page-asset-deployment
description: Automates SPA/static asset deployment to AWS S3 and CloudFront when a `pages:` block is present in the topology YAML. Use this skill when adding a new page entity, deploying the SPA front-end, hosting static site assets, or modifying the `pages:` section in `topology.yml`. Do NOT use for backend function or non-page code changes. Capabilities include generating asset manifests, deploying to S3 buckets, invalidating CloudFront distributions, and updating topology state accordingly.
---
# Page Asset Deployment

## Critical
- Only act when a new or changed `pages:` section is defined in the project YAML configuration, and when actual assets exist in the associated SPA build folders.
- Deployment and invalidation steps are performed via Rust CLI using modules in `lib/deployer/src/aws/`.

## Instructions

1. **Build SPA Assets**
```sh
cd <spa-src-dir>
npm install
npm run build
cd -
```
   - Output must match the SPA build dir referenced in your config. Do not refer to missing build paths; only use existing directories and assets.

2. **Deploy Via CLI**
```sh
tc deploy page
```
   - This will invoke the logic in `lib/deployer/src/aws/` for S3 upload and CloudFront invalidation. Output and logs are handled by the CLI and logged to `logs/combined.log` and `logs/error.log` during operations.

3. **Verify/Log**
   - Confirm deployment hash logs, asset upload messages, and CloudFront invalidation output. Check `logs/` directory files for deployment records.

4. **Asset State Tracking**
   - After deploy, state is tracked and logs are committed using code from the `lib/snapshotter/` module.

5. **Test Deployment**
   - Validate the deployed asset at the CloudFront URL output by `tc deploy page` (referenced in console output and/or `logs/combined.log`). Use browser or `curl` to confirm asset presence and correct headers.

## Examples

User: "Deploy dashboard SPA page."

- Build SPA assets
- Deploy via CLI:
```sh
tc deploy page
```
- Validate URL from logs output; confirm files exist on CloudFront

## Common Issues
- Asset directory not found? Ensure build output exists at correct location.
- AWS/CloudFront errors? Examine CLI output and the contents of `logs/error.log`.
- Caching/stale version? Make sure CloudFront invalidation triggered and force reload the page in your browser.

---
To validate end-to-end asset deployment and state tracking, run:
```sh
make integration-test
```
