---
name: docker-build-layer
description: Adds standard Dockerfile and .dockerignore files for building Python, Ruby, or Node.js layers. Use when adding new function or layer entities that require containerization under supported runtimes. Trigger on commands or prompts containing 'dockerize', 'build layer', or when a new function/layer entity is introduced. Do NOT use this skill to modify or update existing container images or Dockerfiles.
---
# Docker Build Layer

## Critical

- Always generate the Dockerfile and .dockerignore inside the source directory of the function or layer.
- Follow the structure, labels, and commands exactly as in `lib/builder/src/lib.rs` Dockerfile patterns.
- Use `--secret id=aws,src=$AWS_SHARED_CREDENTIALS_FILE` for AWS credentials mounting.
- Ensure the `.dockerignore` excludes `node_modules/`, `dist/`, `.venv/`, `*.zip`, and other patterns exactly as per the project-level configuration embedded in `lib/builder/src/lib.rs`.
- Verify the runtime (Python, Ruby, Node) to apply runtime-specific instructions.
- Do NOT alter existing Dockerfiles or layers; only create for new entities.

## Instructions

1. **Determine the source directory for the layer**
   - Locate the layer or function source directory path from the topology YAML or scaffolding context.
   - Example: `functions/my-python-layer/`
   - Verify the directory exists and is writable before proceeding.

2. **Generate `.dockerignore`**
   - Create a `.dockerignore` file in the source directory containing these patterns:
     ```
     node_modules/
     dist/
     logs/
     .venv/
     *.zip
     *.tar.gz
     Dockerfile
     .dockerignore
     *.DS_Store
     .git
     ```
   - Use the exact patterns and order as seen in `lib/builder/src/lib.rs` to ensure caching and build efficiency.
   - Verify `.dockerignore` is saved correctly before next step.

3. **Generate the Dockerfile**
   - Start with official base image depending on runtime:
     - Python: `python:3.11-slim`
     - Ruby: `ruby:3.2-slim`
     - Node.js: `node:18-alpine`
   - Add labels to identify the build: e.g., `org.opencontainers.image.source` referencing the repo.
   - Install any required system dependencies (e.g., `build-essential`, `libffi-dev` for Python).
   - Copy or mount source files appropriately.
   - Use `--secret id=aws,src=${AWS_SHARED_CREDENTIALS_FILE}` to mount AWS credentials.
   - Bundle dependencies and layer code into `/opt` folder (or as per layer standard).
   - For Python, run `pip install -r requirements.txt --target /opt` inside the image.
   - For Ruby, use `bundle install --path /opt/ruby`.
   - For Node, run `npm install --production --prefix /opt/nodejs`.
   - Final command should prepare the layer zip compatible for deployment.

   - Verify the Dockerfile compiles and builds locally (e.g., `docker buildx build ...`) before finalizing.

4. **Update build tooling if needed**
   - If relevant, update or verify `lib/builder/src/layer/mod.rs` to ensure the build pipeline recognizes the new layer and handles Docker buildx commands correctly.
   - Verify consistent naming and tagging convention for images.

5. **Validation and testing**
   - Run `make build` or `cargo test` scoped to builder layers to verify integration.
   - Run manual docker build within the layer source directory to ensure no syntax errors.

## Examples

User: "Create Docker support files for a new Python layer directory at `functions/my-python-layer`"

Actions:
- Check directory exists.
- Write `.dockerignore` with project-standard exclusion patterns into `functions/my-python-layer/.dockerignore`.
- Generate `Dockerfile` using `python:3.11-slim`, adding AWS secret, copying files, pip installing dependencies into `/opt`.
- Confirm files saved successfully.

Result:
```
functions/my-python-layer/
  Dockerfile
  .dockerignore
```

Dockerfile content uses registered base image and commands matching `lib/builder/src/lib.rs` patterns.

## Common Issues

- **"Cannot find Dockerfile or .dockerignore in source directory"**
  1. Verify correct source path is used.
  2. Confirm the write permissions on the directory.

- **"Docker build fails due to missing AWS credentials"**
  1. Ensure `AWS_SHARED_CREDENTIALS_FILE` env variable is set and file exists.
  2. Confirm `--secret id=aws,...` correctly used in Docker build command.

- **"Dependency install command fails inside Docker"**
  1. Check base image matches runtime.
  2. Verify that `requirements.txt` / `Gemfile` / `package.json` exist and are copied properly.
  3. Confirm layer paths (e.g., `/opt`) are consistent with project standards.

- **"Layer zip exceeds size limit after build"**
  1. Check that `lib/builder/src/layer/mod.rs` is configured to split zips.
  2. Avoid unnecessary files by verifying `.dockerignore`.

- **"Build cache not used, causing slow builds"**
  1. Follow Dockerfile layering patterns from `lib/builder/src/lib.rs`.
  2. Make sure `.dockerignore` excludes heavy folders.


Run `make build` and `cargo test` after adding or modifying build-related files to validate integration.