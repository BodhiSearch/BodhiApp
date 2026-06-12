# Plan: Fix Docker Build rustc/time Crate Compatibility

## Problem

Docker builds failing in `publish-docker.yml` and `publish-docker-multiplatform.yml` with error:
```
error: rustc 1.87.0 is not supported by the following packages:
  time@0.3.46 requires rustc 1.88.0
  time-core@0.1.8 requires rustc 1.88.0
  time-macros@0.2.26 requires rustc 1.88.0
```

## Root Cause

- Docker image uses Rust 1.87.0 (`rust:1.87.0-bookworm`)
- Dockerfile regenerates `Cargo.lock` via `cargo generate-lockfile` (lines 50 and 74)
- This pulls latest `time` crate (0.3.46) which requires Rust 1.88.0
- The `deranged` crate (time dependency) is the actual source of the version requirement

## Existing Fix Pattern

`.github/actions/setup-rust/action.yml` (lines 32-35):
```yaml
- name: Update for deranged issue
  shell: bash
  run: |
    cargo update -p deranged
```

This updates `deranged` to a version compatible with Rust 1.87.0.

## Implementation

### Phase docker-fix: Update Dockerfile

**File**: `devops/app-binary.Dockerfile`

**Change 1** - Line 48-50 (dependency caching stage):
```dockerfile
# Before:
RUN python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml && \
    mv Cargo.filtered.toml Cargo.toml && \
    cargo generate-lockfile

# After:
RUN python3 scripts/filter-cargo-toml.py Cargo.toml Cargo.filtered.toml && \
    mv Cargo.filtered.toml Cargo.toml && \
    cargo generate-lockfile && \
    cargo update -p deranged
```

**Change 2** - Line 74 (application build stage):
```dockerfile
# Before:
RUN cargo generate-lockfile

# After:
# Update deranged for time crate compatibility with Rust 1.87.0 (matches setup-rust action fix)
RUN cargo generate-lockfile && cargo update -p deranged
```

## Verification

1. Run `make docker.dev.cpu.amd64` locally to test CPU build
2. Push changes and trigger CI workflow on a test branch/tag

## Files Modified

- `devops/app-binary.Dockerfile` (2 edits)
