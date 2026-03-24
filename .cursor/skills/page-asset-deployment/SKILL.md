---
name: page-asset-deployment
description: Deploy static assets for SPA pages as defined in the `pages:` blocks of the topology YAML. Use when adding a new SPA page or updating existing page assets requiring S3 hosting and CloudFront distribution setup. This skill ensures topology YAML updates, packaging, and deployment commands follow the established patterns. Do NOT use this skill for non-page static assets or backend function deployment.
---
# Page Asset Deployment

## Critical

1. Only update the `pages:` section in `topology.yml` to define SPA page assets.
2. Ensure `tc compose` is run after editing `pages:` to generate deployment entities.
3. Deploy must use `tc deploy page` command to properly handle S3 and CloudFront resource creation.
4. Asset directories must be present and contain a valid SPA (e.g. Svelte or Mithril build output).

## Instructions

1. **Edit Topology YAML to add or update pages**
   - File: `topology.yml` at the workspace root.
   - Add or modify the `pages:` section following existing examples.
   - Example pattern:
     ```yaml
     pages:
       - name: my-spa-page
         path: /mypage
         source: spa-svelte/dist/
         cache_control: max-age=3600
     ```
   - Verify the path is unique and source directory exists.
   - Verify `topology.yml` indentation and YAML syntax is valid.

2. **Run topology composition**
   - Command: `tc compose [dir]` where `[dir]` contains `topology.yml`.
   - This generates internal graphs and AWS CloudFormation entities.
   - Verify no errors are reported; verify `target/compose/*.json` files updated.

3. **Build SPA assets if needed**
   - Location: Navigate to the SPA source directory (`spa-svelte/` or `spa-mithril/`).
   - Commands (example for Svelte):
     ```bash
     cd spa-svelte
     npm install
     npm run build
     cd -
     ```
   - Verify `dist/` directory or configured output matches `source:` in topology.

4. **Deploy the page assets to AWS**
   - Use: `tc deploy page` from workspace root.
   - This uploads assets to S3 bucket defined by the topology and configures CloudFront.
   - Verify deployment logs for successful S3 sync and CloudFront invalidation.

5. **Validate deployment**
   - Access the page URL from the deployed CloudFront domain.
   - Confirm assets load correctly (HTML, JS, CSS).
   - Verify cache control headers are set per `cache_control` in topology.

## Examples

User says: "Add a new SPA page for the marketing site and deploy it."

Actions taken:
- Edited `topology.yml` to add under `pages:` the entry for `marketing-spa` with source `spa-svelte/dist/` and path `/marketing`.
- Ran `tc compose .` to generate deployment graph.
- Built the SPA assets by running npm build in `spa-svelte/`.
- Ran `tc deploy page`.
- Validated deployment URL loads the marketing SPA with proper caching.

Result:
- New SPA page deployed to S3 and served through CloudFront.
- Topology now tracks this page as a deployable entity.

## Common Issues

- **Error: `source directory does not exist`**
  1. Verify the `source:` path in `pages:` matches the relative path from the workspace root.
  2. Confirm the directory contains build artifacts (e.g. `index.html`).

- **Error: `tc compose` fails with validation errors on pages**
  1. Check YAML syntax and indentation in `topology.yml`.
  2. Ensure all required fields (name, path, source) are present.

- **S3 upload fails or assets missing after deploy**
  1. Check AWS CLI credentials and permissions.
  2. Confirm `tc deploy page` completed without errors.
  3. Ensure asset files are not ignored by project `.dockerignore` or `.gitignore` (only relevant if deployed from build environment).

- **CloudFront cache not invalidated after deploy**
  1. Verify `tc deploy page` logs show CloudFront invalidation trigger.
  2. If not, manually invalidate CloudFront distribution via AWS Console.

- **Page loads with stale assets or errors**
  1. Confirm browser cache cleared.
  2. Verify `cache_control` values in topology are correct.
  3. Check distributed assets via AWS Console S3 bucket.

---

Follow these patterns strictly for consistent SPA page static asset deployment across all teams and automation workflows.