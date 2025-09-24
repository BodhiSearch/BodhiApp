# PACKAGE.md

This file provides implementation details and file references for the GitHub CI/CD infrastructure. See [CLAUDE.md](.github/CLAUDE.md) for architectural guidance.

## Quick Reference

### Core Commands
- `make test` - Trigger full test suite (matches CI/CD validation)
- `make ci.build` - Execute CI-style build process  
- `make ci.clean` - Clean build environment (as used in workflows)
- `make ci.coverage` - Generate coverage reports (matches CI process)
- `make format.all` - Format code and run linting (CI-compatible)

### Manual Workflow Triggers
```bash
# Trigger release workflow manually
gh workflow run release.yml -f draft=yes -f prerelease=yes

# Trigger Docker publishing
gh workflow run publish-docker.yml

# Trigger NPM package publishing  
gh workflow run publish-npm-napi.yml
```

## Workflow Architecture

### Primary Build Workflows

#### Fast Linux Build and Test
**File**: `.github/workflows/build.yml:1-240`

Core development workflow optimized for rapid feedback with Ubuntu-only execution:

```yaml
# Trigger configuration for main development flow
on:
  push:
    branches: [main, working]
    paths: ['crates/**', 'xtask/**']
  pull_request:
    branches: [main]
```

**Key Implementation Pattern**: 
- Two-stage execution: `build-and-test` → `playwright-tests`
- Conditional Playwright execution based on coverage success (`build.yml:93-96`)
- Artifact coordination between stages using actions/upload-artifact and actions/download-artifact

**Critical Flows**:
1. **Coverage Generation**: `.github/workflows/build.yml:67-86` - Executes `make ci.coverage` with CI_DEFAULT_VARIANT=cpu
2. **NAPI Build Coordination**: `.github/workflows/build.yml:87-92` - Uses `.github/actions/napi-build` action
3. **Artifact Downloads**: `.github/workflows/build.yml:112-123` - Downloads NAPI bindings and llama-server binaries for Playwright tests

#### Multi-Platform Build System
**File**: `.github/workflows/build-multiplatform.yml:1-180`

Comprehensive cross-platform validation for release preparation with matrix strategy covering macOS (ARM64), Linux (x86_64), and Windows (x86_64).

**Matrix Definition Pattern**:
```yaml
strategy:
  fail-fast: false
  matrix:
    include:
      - platform: macos-latest
        target: aarch64-apple-darwin
      - platform: ubuntu-latest  
        target: x86_64-unknown-linux-gnu
      - platform: windows-latest
        target: x86_64-pc-windows-msvc
```

### Release Orchestration Workflows

#### Desktop Application Release  
**File**: `.github/workflows/release.yml:1-203`

Sophisticated release workflow supporting multiple release types with version management and Apple code signing:

**Version Extraction Logic**: `.github/workflows/release.yml:102-123`
```bash
# Dynamic version determination from git tags
if [[ "${GITHUB_REF}" =~ ^refs/tags/v([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
  echo "VERSION=${BASH_REMATCH[1]}" >> $GITHUB_OUTPUT
  echo "TAG_BUILD=true" >> $GITHUB_OUTPUT
else
  # Auto-increment latest version for manual releases
  # Implementation details in lines 109-122
fi
```

**Critical Apple Developer Integration**: `.github/workflows/release.yml:182-202`
- Uses `tauri-apps/tauri-action@v0` with comprehensive Apple credential management
- Coordinates certificate installation, code signing, and app notarization
- Manages keychain setup and cleanup for secure credential handling

#### Multi-Variant Docker Publishing
**File**: `.github/workflows/publish-docker.yml:1-221`

Advanced container orchestration supporting multiple hardware variants with intelligent build matrices:

**Variant Matrix Strategy**: `.github/workflows/publish-docker.yml:74-90`
```yaml
matrix:
  variant: [cpu, cuda, rocm]  # vulkan commented out
  include:
    - variant: cpu
      platforms: 'linux/amd64,linux/arm64'  # Multi-platform
    - variant: cuda  
      platforms: 'linux/amd64'  # GPU variants AMD64-only
    - variant: rocm
      platforms: 'linux/amd64'
```

**Advanced Tagging Strategy**: `.github/workflows/publish-docker.yml:115-117`
- Version-specific tags: `{version}-{variant}{suffix}`
- Latest tags: `latest-{variant}{suffix}`  
- Development variant support with `-development` suffix

### Package Publishing Workflows

#### NPM NAPI Bindings
**File**: `.github/workflows/publish-npm-napi.yml:1-120`

NAPI package publishing with cross-platform binary coordination and automated version management.

#### TypeScript Client Publishing
**File**: `.github/workflows/publish-ts-client.yml:1-85`

TypeScript client package publishing with OpenAPI schema validation and automated type generation.

## Reusable GitHub Actions

### Environment Setup Actions

#### Core Environment Setup
**File**: `.github/actions/setup-environment/action.yml:1-39`

Foundational setup action handling git configuration, submodules, and Python dependencies:

```yaml
# Key implementation patterns
- name: Enable symlinks and configure git
  run: |
    git config --global core.symlinks true
    git config --global core.autocrlf false
    git config --global core.eol lf

- name: Rewrite submodule URLs and checkout submodules  
  run: |
    git config --global url.https://gh_pat:${{ inputs.gh-pat }}@github.com/.insteadOf git@github.com:
    git submodule sync --recursive
    git submodule update --init --recursive --depth=1
```

**Python Dependencies**: `.github/actions/setup-environment/action.yml:34-39`
- Installs requirements from `crates/objs/tests/scripts/requirements.txt`
- Includes Hugging Face CLI for model management

#### Rust Toolchain Setup
**File**: `.github/actions/setup-rust/action.yml:1-65`

Comprehensive Rust toolchain setup with target-specific configuration and caching optimization.

#### Node.js Environment Setup
**File**: `.github/actions/setup-node/action.yml:1-45`

Node.js setup with npm caching and version consistency management across workflow executions.

### Build and Test Actions

#### Core Build and Test Execution
**File**: `.github/actions/build-and-test/action.yml:1-68`

Central build and test orchestration with coverage reporting and artifact management:

**Coverage Pipeline**: `.github/actions/build-and-test/action.yml:26-32`
```yaml
- name: Generate code coverage
  id: coverage  
  shell: bash
  run: make ci.coverage
  env:
    CI_DEFAULT_VARIANT: cpu
```

**Artifact Coordination**: `.github/actions/build-and-test/action.yml:45-50`
- Uploads llama_server_proc binaries for downstream consumption
- Coordinates UI test execution and coverage reporting
- Manages Codecov integration with multi-language support

#### NAPI Binary Building
**File**: `.github/actions/napi-build/action.yml:1-55`

NAPI binding compilation with platform-specific optimization and artifact coordination.

### Testing Infrastructure Actions

#### Playwright Test Setup  
**File**: `.github/actions/setup-playwright/action.yml:1-35`

Playwright testing environment setup with browser installation and dependency management.

#### Model Management Setup
**File**: `.github/actions/setup-models/action.yml:1-30`

LLM model downloading and setup for integration testing scenarios.

### Platform-Specific Actions

#### Windows Environment Setup
**File**: `.github/actions/setup-win/action.yml:1-25`

Windows-specific toolchain setup handling Visual Studio Build Tools and platform dependencies.

#### Docker Rust Setup
**File**: `.github/actions/setup-rust-docker/action.yml:1-40`  

Docker-optimized Rust setup for container build environments.

## Configuration Files

### Dependency Management
**File**: `.github/dependabot.yml:1-13`

Basic Dependabot configuration for devcontainer dependency updates:

```yaml
version: 2
updates:
- package-ecosystem: "devcontainers"
  directory: "/"
  schedule:
    interval: weekly
```

**Extension Opportunity**: Configuration could be expanded to include Rust, npm, and GitHub Actions dependency management.

## Implementation Patterns

### Artifact Flow Coordination

The CI/CD system implements sophisticated artifact passing between workflow jobs:

**Pattern 1: Build → Test Coordination**
```yaml  
# In build job (.github/workflows/build.yml:45-50)
- name: Upload llama_server_proc binaries
  uses: actions/upload-artifact@v4
  with:
    name: llama-server-binaries-${{ inputs.target }}
    path: crates/llama_server_proc/bin/

# In test job (.github/workflows/build.yml:118-122)  
- name: Download llama_server_proc binaries
  uses: actions/download-artifact@v4
  with:
    name: llama-server-binaries-x86_64-unknown-linux-gnu
    path: crates/llama_server_proc/bin/
```

### Cross-Platform Binary Management

**Pattern 2: Platform-Specific Binary Coordination**
```bash
# Binary verification with platform-specific paths (.github/workflows/build.yml:124-140)
BINARY_PATH="crates/llama_server_proc/bin/x86_64-unknown-linux-gnu/cpu/llama-server"
if [ -f "$BINARY_PATH" ]; then
  echo "✅ Binary found at: $BINARY_PATH"  
  chmod +x "$BINARY_PATH"
else
  echo "❌ Binary not found at expected location!"
  exit 1
fi
```

### Conditional Execution Patterns

**Pattern 3: Coverage-Based Test Gating**
```yaml
# Conditional job execution (.github/workflows/build.yml:93-96)
playwright-tests:
  needs: build-and-test
  if: needs.build-and-test.outputs.coverage-success == 'true'
```

### Security and Credential Management

**Pattern 4: Secure Credential Passing**
```yaml
# Environment variable coordination (.github/workflows/build.yml:74-86)
env:
  HF_TEST_TOKEN_ALLOWED: ${{ secrets.HF_TEST_TOKEN_ALLOWED }}
  INTEG_TEST_AUTH_URL: ${{ vars.INTEG_TEST_AUTH_URL }}
  INTEG_TEST_USERNAME: ${{ secrets.INTEG_TEST_USERNAME }}
```

## Extension Guidelines

### Adding New Workflow Triggers

When adding new trigger conditions, coordinate updates across related workflows:

1. **Path-based triggers**: Update `paths` filters in `.github/workflows/build.yml:8-11`
2. **Branch triggers**: Coordinate `branches` configuration across workflows  
3. **Manual triggers**: Add `workflow_dispatch` with appropriate inputs

### Platform Support Extension

Adding new target platforms requires:

1. **Matrix updates**: Add platform/target combinations to build matrices
2. **Setup actions**: Create platform-specific setup actions following naming patterns
3. **Binary paths**: Update artifact coordination for platform-specific binary locations
4. **Test adaptation**: Modify test execution for platform-specific requirements

### Monitoring Integration

The workflows support monitoring integration through:

1. **Structured outputs**: All critical steps emit structured status information
2. **Artifact retention**: Configurable retention periods for debugging and analysis
3. **External reporting**: Integration points for external monitoring systems
4. **Failure notifications**: Comprehensive error reporting and notification patterns

## Debugging and Troubleshooting

### Common Debugging Patterns

**Pattern 1: Artifact Verification**
```bash
# Before using artifacts, verify their existence
if [ ! -f app-bindings.*.node ]; then
  echo "Error: NAPI bindings not found. Build may have failed."
  exit 1
fi
```

**Pattern 2: Environment Validation**  
```bash
# Validate required environment variables
echo "Checking test results..."
echo "NAPI tests outcome: ${{ steps.napi-tests.outcome }}"
echo "Playwright tests outcome: ${{ steps.playwright-run.outcome }}"
```

### Test Result Management

**Test Reporter Integration**: `.github/workflows/build.yml:169-177`
```yaml
- name: Publish NAPI binding test results
  uses: dorny/test-reporter@v1
  with:
    name: NAPI Binding Tests (Linux)
    path: crates/lib_bodhiserver_napi/test-results/vitest-junit.xml
    reporter: java-junit
    fail-on-error: false
```

The CI/CD system provides comprehensive failure analysis through artifact uploads, structured test reporting, and detailed logging at each stage of the pipeline execution.