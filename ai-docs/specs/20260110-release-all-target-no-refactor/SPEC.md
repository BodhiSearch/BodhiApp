# Unified Release Target - Non-Refactoring Implementation

**Date**: 2026-01-10
**Status**: IMPLEMENTED
**Approach**: Additive only - no changes to existing targets

---

## Executive Summary

This specification documents the **implemented** unified `make release.all` target that atomically creates and pushes tags for all BodhiApp components (ts-client, app-bindings, native app, Docker) on the same commit. This implementation is **additive only** - no existing release targets were modified.

Additionally, this includes a fix for concurrent version bump conflicts that occur when multiple GitHub workflows attempt to push version bumps simultaneously after releases.

---

## Problem Statement

### Current Pain Points

1. **Manual Coordination**: Releasing all components requires running 4 separate commands:
   - `make release-ts-client`
   - `make release-app-bindings`
   - `make release-app`
   - `make release-docker`

2. **Timing Skew**: Tags created on different commits if main branch receives concurrent pushes between releases

3. **Concurrent Bump Conflicts**: When `release.all` triggers 4 workflows simultaneously, their post-release version bump commits conflict:
   ```
   Workflow A: bump ts-client version → push to main (conflicts with B, C, D)
   Workflow B: bump app-bindings version → push to main (conflicts)
   Workflow C: bump app version → push to main (conflicts)
   Workflow D: bump docker version → push to main (conflicts)
   ```

### Solution Goals

1. Single command to release all components
2. All tags created on identical commit (atomic)
3. Single atomic git push operation for all tags
4. Fix concurrent version bump conflicts in CI

---

## Implementation Approach

### Design Philosophy

- **Zero Refactoring**: No changes to existing release targets
- **Function Reuse**: Leverage existing `scripts/release.mk` functions
- **Additive Only**: New target doesn't impact existing workflows
- **Backwards Compatible**: All existing functionality preserved

### Architecture

```
┌─────────────────────────────────────────────────┐
│         Existing release.mk functions           │
│  (check_git_branch, get_npm_version, etc.)     │
└──────────────────┬──────────────────────────────┘
                   │
                   │ Reused by
                   │
    ┌──────────────┴───────────────┐
    │                              │
    │                              │
┌───▼────────────┐    ┌────────────▼─────┐
│ Individual     │    │  NEW: release.all│
│ Releases       │    │  (Unified)       │
│ (Unchanged)    │    │                  │
│ - ts-client    │    │  All components  │
│ - app-bindings │    │  atomically      │
│ - app          │    │                  │
│ - docker       │    │                  │
└────────────────┘    └──────────────────┘
```

---

## Components Released

| Component | Tag Format | Version Source | Workflow Triggered |
|-----------|------------|----------------|-------------------|
| TypeScript client | `ts-client/vX.Y.Z` | npm registry | `publish-ts-client.yml` |
| NAPI bindings | `bodhi-app-bindings/vX.Y.Z` | npm registry | `publish-app-bindings.yml` |
| Native app | `app/vX.Y.Z` | GitHub releases | `release.yml` |
| Docker (prod) | `docker/vX.Y.Z` | GHCR | `publish-docker.yml` + `publish-docker-multiplatform.yml` |

**Not included**:
- `docker-dev` (optional, separate release)
- `website` (has special timing dependencies on app release URLs)

---

## Implementation Details

### 1. New `release.all` Target

**File**: `Makefile` (lines 155-219)

**Flow**:
1. Validate git state via `check_git_branch` (on main, synced with origin)
2. Fetch current versions from all sources (npm, GitHub releases, GHCR)
3. Calculate next versions (patch increment)
4. Display release plan showing current→next for all components
5. Check each tag for existence via `delete_tag_if_exists` (4 prompts if tags exist)
6. Final confirmation: "Create and push all release tags? [y/N]"
7. Create all 4 tags locally on current commit
8. **Sequential push**: Push each tag individually with 1-second delay between pushes
   - Required for GitHub Actions to create separate workflow runs per tag
   - All tags still point to identical commit SHA (commit-level atomicity preserved)
   - 1-second delay ensures webhook events are processed separately
   - Continue pushing all tags even if some fail (no early termination)
9. On completion: display summary of succeeded and failed pushes
   - If all succeeded: show successful tags and triggered workflows
   - If any failed: show both succeeded and failed tags with manual push commands
   - Exit with error code if any push failed
10. Local tags are preserved (not deleted) regardless of push success/failure

**Key Features**:
- Reuses existing validation and version-fetching functions
- All tags point to identical commit SHA (commit-level atomicity)
- Sequential git push operations (required for GitHub Actions workflow triggering)
- 1-second delay between pushes to ensure proper webhook delivery
- Resilient push process: continues even if individual tags fail
- Local tags preserved for manual recovery
- Clear reporting of succeeded and failed pushes

### 2. Concurrent Version Bump Fix

**Problem**: When 4 workflows run simultaneously and each tries to push a version bump commit to main, they conflict.

**Solution**: Enhanced retry logic in `.github/actions/commit-and-push-with-retry/action.yml`

**New Input**: `bump-command`
- Optional command to regenerate changes after rebase
- Allows workflows to recalculate version on retry

**Enhanced Retry Logic**:
```yaml
On push failure:
  1. git reset --hard HEAD~1  # Undo bump commit
  2. git pull --rebase origin main  # Sync with upstream
  3. eval "$BUMP_CMD"  # Re-run bump command (recalculates version)
  4. git add files
  5. git commit
  6. Retry push (up to 3 times)
```

**Why This Works**:
- Fresh pull ensures we have latest commits from other workflows
- Re-running bump command recalculates version based on latest state
- Each retry starts fresh, not with stale version from initial attempt

### 3. Workflow Updates

Three workflows updated to use `bump-command`:

**publish-ts-client.yml** (line 146):
```yaml
bump-command: "cd ts-client && npm version $VERSION --no-git-tag-version"
```

**publish-app-bindings.yml** (line 197):
```yaml
bump-command: "cd crates/lib_bodhiserver_napi && npm version $VERSION --no-git-tag-version && npm install"
```

**release.yml** (lines 316-319):
```yaml
bump-command: |
  NEXT_VERSION=$(./scripts/increment_version.js "$VERSION" patch dev)
  make ci.update-version VERSION="$NEXT_VERSION"
  cargo update --workspace
```

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Approach | Additive only | No risk to existing workflows, easy rollback |
| Function reuse | Use existing release.mk | Consistency, less duplication |
| Validation | Use check_git_branch once | Same validation as individual targets |
| Tag prompts | Use delete_tag_if_exists per tag | Consistent with existing UX (4 prompts if needed) |
| Final confirmation | Yes, before creating/pushing | Extra safety for critical operation |
| Push strategy | Sequential with 1s delay | Required for GitHub Actions to trigger separate workflow runs per tag |
| Failure handling | Continue on error, report all failures | Maximize successful pushes, provide clear recovery commands |
| Tag preservation | Keep local tags on failure | Enable manual recovery without recreating tags |
| Bump fix | Include in this change | Solves immediate problem caused by release.all |

### GitHub Actions Limitation: Sequential Push Requirement

**Issue Discovered**: When pushing multiple tags atomically (`git push origin tag1 tag2 tag3 tag4`), GitHub Actions does not create separate push events for each tag. This results in workflows not being triggered.

**Root Cause**: GitHub's webhook system bundles multiple tag pushes into a single event or processes only the first tag when tags are pushed simultaneously.

**Solution**: Push tags sequentially with a 1-second delay between each push. This ensures:
- Each tag push creates a separate GitHub webhook event
- Each workflow is triggered independently
- All tags still point to the same commit SHA (commit-level atomicity preserved)

**Trade-off**: Sequential push takes ~4 seconds longer than atomic push, but ensures workflows trigger correctly.

---

## User Experience

### Command Usage

```bash
$ make release.all
```

### Example Session

```
==========================================
       Unified Release: All Components
==========================================
Fetching latest changes from remote...

Fetching current versions...

Release Plan:
  ts-client:     0.1.10 -> 0.1.11
  app-bindings:  0.0.22 -> 0.0.23
  app:           0.0.42 -> 0.0.43
  docker:        0.0.8 -> 0.0.9

Checking existing tags...
Checking for existing tag ts-client/v0.1.11...
Checking for existing tag bodhi-app-bindings/v0.0.23...
Checking for existing tag app/v0.0.43...
Checking for existing tag docker/v0.0.9...

Create and push all release tags? [y/N] y

Creating tags on current commit...
Pushing tags sequentially (to trigger GitHub Actions)...
  Pushing ts-client/v0.1.11...
  ✓ ts-client/v0.1.11 pushed
  Pushing bodhi-app-bindings/v0.0.23...
  ✓ bodhi-app-bindings/v0.0.23 pushed
  Pushing app/v0.0.43...
  ✓ app/v0.0.43 pushed
  Pushing docker/v0.0.9...
  ✓ docker/v0.0.9 pushed

==========================================
           Release Complete
==========================================
Tags successfully pushed:
  ✓ ts-client/v0.1.11
  ✓ bodhi-app-bindings/v0.0.23
  ✓ app/v0.0.43
  ✓ docker/v0.0.9

Workflows triggered:
  - publish-ts-client.yml
  - publish-app-bindings.yml
  - release.yml
  - publish-docker.yml + publish-docker-multiplatform.yml
```

### Error Handling

**If some pushes fail** (partial failure example):
```
Pushing tags sequentially (to trigger GitHub Actions)...
  Pushing ts-client/v0.1.11...
  ✓ ts-client/v0.1.11 pushed
  Pushing bodhi-app-bindings/v0.0.23...
  ✗ Failed to push bodhi-app-bindings/v0.0.23
  Pushing app/v0.0.43...
  ✓ app/v0.0.43 pushed
  Pushing docker/v0.0.9...
  ✓ docker/v0.0.9 pushed

==========================================
     Release Completed with Failures
==========================================
Tags successfully pushed:
  ✓ ts-client/v0.1.11
  ✓ app/v0.0.43
  ✓ docker/v0.0.9

Tags that failed to push:
  ✗ bodhi-app-bindings/v0.0.23

Please manually push failed tags:
  git push origin bodhi-app-bindings/v0.0.23

make: *** [release.all] Error 1
```

**Key behaviors**:
- All tags are attempted (no early termination on first failure)
- Local tags are preserved for manual recovery
- Clear reporting of what succeeded and what failed
- Manual push commands provided for failed tags
- Successfully pushed tags trigger their workflows normally

---

## Files Modified

| File | Changes | Lines | Status |
|------|---------|-------|--------|
| `Makefile` | Added `release.all` target | 155-219 | ✅ Implemented |
| `Makefile` | Updated `.PHONY` | 414 | ✅ Implemented |
| `.github/actions/commit-and-push-with-retry/action.yml` | Added `bump-command` input | 23-26 | ✅ Implemented |
| `.github/actions/commit-and-push-with-retry/action.yml` | Enhanced retry logic | 125-171 | ✅ Implemented |
| `.github/workflows/publish-ts-client.yml` | Added `bump-command` | 146 | ✅ Implemented |
| `.github/workflows/publish-app-bindings.yml` | Added `bump-command` | 197 | ✅ Implemented |
| `.github/workflows/release.yml` | Added `bump-command` | 316-319 | ✅ Implemented |

**Files NOT Modified**:
- `scripts/release.mk` - All functions unchanged
- Individual release targets (`release-ts-client`, `release-app`, etc.) - Completely unchanged
- Any other Makefile targets - No modifications

---

## Benefits

### For Developers

1. **Single Command**: `make release.all` instead of 4 separate commands
2. **Consistency**: All tags on identical commit
3. **Safety**: Multiple confirmation prompts before irreversible operations
4. **Transparency**: Clear output showing what will happen
5. **Resilient Error Handling**: Continues pushing all tags even if some fail
6. **Easy Recovery**: Failed tags preserved locally with manual push commands provided
7. **Rollback**: Easy to revert (just delete the new target)

### For CI/CD

1. **No More Conflicts**: Workflows handle concurrent bumps gracefully
2. **Automatic Retry**: Up to 3 attempts with fresh version calculation
3. **Observability**: Clear logging of retry attempts
4. **Backwards Compatible**: Works with existing workflow structure

### For Project

1. **Zero Risk**: No changes to existing release process
2. **Gradual Adoption**: Teams can use `release.all` or individual targets
3. **Maintainable**: Uses existing functions, no code duplication
4. **Future-Proof**: Easy to extend to include more components

---

## Testing Strategy

### Manual Testing

1. **Dry run** (abort at confirmation): Verify version calculation
2. **Single component changed**: Confirm other versions still increment
3. **All components unchanged**: Verify idempotent behavior
4. **Existing tags**: Confirm deletion prompts work
5. **Network failure simulation**: Verify cleanup on push failure
6. **Concurrent workflows**: Trigger multiple workflows, verify no conflicts

### Integration Testing

1. Run `release.all` on test branch
2. Verify all 4 tags created on same commit
3. Confirm workflows triggered correctly
4. Validate version bump commits succeed without conflicts
5. Check final state: all components at next version with `-dev` suffix

---

## Limitations and Future Work

### Current Limitations

1. **4 Tag Prompts**: If tags exist, user gets 4 separate prompts (uses existing function)
2. **Manual Only**: No CI automation mode (always interactive)
3. **Fixed Components**: Cannot select subset of components to release
4. **No Dry Run**: Cannot preview without interactive prompts

### Future Enhancements (Optional)

1. **Batch Tag Prompt**: Single prompt for all existing tags
2. **Component Selection**: `COMPONENTS='ts-client,app' make release.all`
3. **Dry Run Mode**: `DRY_RUN=1 make release.all`
4. **CI Mode**: `CI=1 make release.all` (non-interactive)
5. **Parallel Version Fetch**: Fetch versions concurrently for speed

---

## Comparison with Refactoring Approach

| Aspect | Non-Refactoring (Implemented) | Refactoring (Deferred) |
|--------|------------------------------|------------------------|
| Existing targets | Unchanged | Modified to use new primitives |
| Risk level | Minimal | Medium (changes existing behavior) |
| Implementation time | ~1 hour | ~4-6 hours |
| Testing burden | Light | Heavy (validate all existing flows) |
| Rollback complexity | Trivial | Non-trivial |
| Code duplication | Some | Minimal |
| Long-term maintainability | Good | Better |
| Adoption | Optional | Forced (changes existing) |

**Decision**: Implemented non-refactoring approach to deliver value quickly with minimal risk. Refactoring can be considered later if needed.

---

## Migration Path

### From Individual Releases

**Before**:
```bash
make release-ts-client
# wait for workflow...
make release-app-bindings
# wait for workflow...
make release-app
# wait for workflow...
make release-docker
```

**After**:
```bash
make release.all
# All workflows trigger simultaneously
```

### Gradual Adoption

- Teams can continue using individual targets
- `release.all` available when needed
- No forced migration
- Both approaches work identically

---

## Conclusion

This implementation delivers a unified release target without modifying any existing functionality. The additive approach minimizes risk while solving the real pain points of manual coordination and concurrent version bump conflicts.

**Status**: ✅ Fully implemented and ready for use

---

## References

- Original requirements: User request for unified release on same commit
- Deferred refactoring spec: `ai-docs/specs/20260110-release-all-target/`
- Makefile implementation: `Makefile:155-219`
- Retry action: `.github/actions/commit-and-push-with-retry/action.yml`
- Version scripts: `scripts/get_npm_version.js`, `scripts/get_github_release_version.js`, `scripts/get_ghcr_version.py`
