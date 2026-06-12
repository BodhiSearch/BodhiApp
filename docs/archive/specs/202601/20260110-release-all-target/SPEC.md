# Unified Release Target Specification

**Date**: 2026-01-10
**Status**: DEFERRED - Pending review of refactoring approach
**Author**: Claude (AI Assistant)

---

## Executive Summary

This specification proposes a unified `make release.all` target that atomically creates and pushes release tags for all BodhiApp components (ts-client, app-bindings, native app, Docker images) on the same commit. The proposal also refactors existing release targets into decoupled primitives for better maintainability and addresses concurrent version bump conflicts in GitHub Actions workflows.

**Key Decision**: Implementation deferred due to concerns about refactoring existing targets. Only the plan is documented here for future consideration.

---

## Problem Statement

### Current State

BodhiApp has 6 independent release targets:
- `release-ts-client` - TypeScript client (@bodhiapp/ts-client)
- `release-app-bindings` - NAPI bindings (@bodhiapp/app-bindings)
- `release-app` - Native desktop application
- `release-docker` - Production Docker images
- `release-docker-dev` - Development Docker images
- `website.release` - Website deployment (delegated to getbodhi.app)

### Issues

1. **Manual Coordination**: Releasing all components requires running multiple commands, increasing human error risk
2. **Timing Skew**: Tags created minutes apart on different commits (if main receives concurrent pushes)
3. **Duplicate Validation**: Each target independently validates git sync, branch status
4. **Code Duplication**: Similar logic repeated across targets (version fetching, tag creation, push)
5. **Concurrent Bump Conflicts**: GitHub workflows that bump versions after release conflict when triggered simultaneously

### Goals

1. Single command to release all components atomically
2. All tags created on identical commit
3. Single atomic git push operation
4. Decoupled primitives for maintainability
5. Fix concurrent version bump conflicts in CI

---

## Current Release System Analysis

### Release Targets

| Target | Version Source | Tag Format | Workflow Triggered |
|--------|----------------|------------|-------------------|
| `release-ts-client` | npm registry | `ts-client/vX.Y.Z` | `publish-ts-client.yml` |
| `release-app-bindings` | npm registry | `bodhi-app-bindings/vX.Y.Z` | `publish-app-bindings.yml` |
| `release-app` | GitHub releases | `app/vX.Y.Z` | `release.yml` |
| `release-docker` | GHCR | `docker/vX.Y.Z` | `publish-docker.yml` + `publish-docker-multiplatform.yml` |

### Current Flow

```
1. Developer runs: make release-ts-client
   - check_git_branch (validate main + sync)
   - get_npm_version (@bodhiapp/ts-client)
   - increment_version
   - delete_tag_if_exists (interactive prompt)
   - create tag locally
   - push tag
   - GitHub workflow: publish + bump version + commit

2. Developer runs: make release-app
   (same pattern, different version source)

3. Problem: If concurrent, version bumps conflict in CI
```

### Existing Primitives (scripts/release.mk)

- `check_git_branch` - Validate branch and sync (interactive)
- `delete_tag_if_exists` - Delete tag with confirmation
- `get_npm_version` - Fetch version from npm
- `get_app_version` - Fetch version from GitHub releases
- `get_ghcr_docker_version` - Fetch version from GHCR
- `increment_version` - Call increment_version.js script

### Issues with Current Primitives

1. **Monolithic validation**: `check_git_branch` does both branch check and sync
2. **No abstraction**: Version getters are separate functions, can't iterate
3. **Implicit state**: Tag deletion happens during check
4. **Remote check missing**: Only checks local tags, not remote
5. **No atomic push**: Each release pushes individually

---

## Proposed Solution

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Primitive Layer                      │
├─────────────────────────────────────────────────────────┤
│  check_git_sync          - Validate git state           │
│  get_component_version   - Unified version getter       │
│  check_tag_exists        - Check local + remote         │
│  delete_tag_remote/local - Separate delete ops          │
│  handle_existing_tag     - Interactive delete flow      │
│  create_tag              - Create local tag             │
│  push_tags_atomic        - Atomic multi-tag push        │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ Compose
                          │
        ┌─────────────────┴─────────────────┐
        │                                   │
┌───────▼────────┐              ┌───────────▼────────┐
│ Individual     │              │  release.all       │
│ Releases       │              │  (Unified)         │
│                │              │                    │
│ - ts-client    │              │  All components    │
│ - app-bindings │              │  atomically        │
│ - app          │              │                    │
│ - docker       │              │                    │
└────────────────┘              └────────────────────┘
```

### New Primitives

1. **check_git_sync**: Validate branch (main), sync with origin, clean working tree
2. **get_component_version**: Unified getter supporting npm/github-release/ghcr types
3. **check_tag_exists**: Return "local", "remote", "both", or empty
4. **delete_tag_remote/local**: Separate operations (remote first for safety)
5. **handle_existing_tag**: Interactive prompt and deletion coordination
6. **create_tag**: Create local tag (no push)
7. **push_tags_atomic**: Push multiple tags in single operation

### Refactored Individual Releases

Each release target becomes:
```makefile
release-ts-client:
    $(call check_git_sync)
    CURRENT=$(get_component_version,npm,@bodhiapp/ts-client)
    NEXT=$(increment_version,$CURRENT)
    TAG=ts-client/v$NEXT
    $(call handle_existing_tag,$TAG)
    $(call create_tag,$TAG)
    $(call push_tags_atomic,$TAG)
```

**Benefits**: Clear flow, minimal duplication, easy to test/debug

### Unified release.all

```makefile
release.all:
    1. check_git_sync (once)
    2. Fetch all current versions (4 components)
    3. Calculate next versions (4 components)
    4. Check all tags upfront (batch)
    5. Single prompt: "Delete all existing?"
    6. Delete tags (remote first, then local)
    7. Create all tags (on same commit)
    8. Atomic push: git push origin tag1 tag2 tag3 tag4
    9. On failure: cleanup local tags
```

### Concurrent Bump Fix

**Problem**: When `release.all` triggers 4 workflows, each tries to bump version and push to main simultaneously.

**Current behavior**:
```
Workflow A: bump ts-client  → push main (conflicts)
Workflow B: bump app-bindings → push main (conflicts)
Workflow C: bump app          → push main (conflicts)
Workflow D: bump docker       → (never completes)
```

**Solution**: Add `bump-command` input to retry action
```yaml
- name: Commit and push version bump
  with:
    bump-command: "cd ts-client && npm version $VERSION --no-git-tag-version"

# Retry logic:
# 1. Push attempt
# 2. On failure: git reset HEAD~1
# 3. Pull --rebase origin main
# 4. Re-run bump-command (calculates fresh version)
# 5. Commit and retry push
```

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bash compatibility | POSIX-compatible | macOS default bash is 3.2 (no bash 4 features) |
| Old function names | Keep as aliases | Backwards compatibility, gradual migration |
| Version override | Remove | Simplifies logic, auto-increment is standard |
| Tag existence | Abort if decline | All-or-nothing consistency |
| [skip ci] | No | Run CI on all commits for validation |
| Dirty working tree | Abort | Prevents accidental uncommitted changes |
| Interactive prompts | Always | Critical operation requires human approval |
| Logging | None | Terminal output sufficient |
| Tag check order | Upfront batch | Single confirmation UX |

---

## Trade-offs and Concerns

### Advantages

1. **Atomic releases**: All components on same commit
2. **Reduced human error**: Single command
3. **Better UX**: Clear flow, upfront confirmation
4. **Maintainability**: Decoupled primitives
5. **Testability**: Small composable functions
6. **Concurrent fix**: Workflows won't conflict

### Concerns (Why Deferred)

1. **Breaking changes**: Refactors existing targets (user discomfort)
2. **Testing burden**: Must validate all existing release flows still work
3. **Rollback complexity**: If issues found, reverting is non-trivial
4. **Documentation gap**: Need to update all release docs
5. **Migration risk**: Teams might have scripts depending on current behavior

### Alternative Approaches

1. **Additive only**: Add `release.all` without refactoring (creates duplication)
2. **Parallel refactor**: Keep old targets unchanged, add new namespaced targets
3. **Gradual migration**: Refactor one target at a time, measure impact
4. **Script-based**: Bash script instead of Makefile (different trade-offs)

---

## Implementation Plan (Deferred)

### Phase 1: Refactor Primitives
- File: `scripts/release.mk`
- Add 8 new primitives
- Keep old functions as thin aliases
- No breaking changes

### Phase 2: Refactor Individual Targets
- File: `Makefile`
- Update 4 release targets to use new primitives
- Maintain identical behavior
- Extensive testing

### Phase 3: Add release.all
- File: `Makefile`
- New unified target
- Uses same primitives
- Comprehensive error handling

### Phase 4: Fix Concurrent Bumps
- File: `.github/actions/commit-and-push-with-retry/action.yml`
- Add `bump-command` input
- Update retry logic
- Test with concurrent workflows

### Phase 5: Update Workflows
- Files: `publish-ts-client.yml`, `publish-app-bindings.yml`, `release.yml`
- Add `bump-command` parameter
- Validate no regressions

---

## Files to Modify

| File | Changes | Risk Level |
|------|---------|------------|
| `scripts/release.mk` | Add primitives + aliases | Low (aliases preserve behavior) |
| `Makefile` | Refactor 4 targets + add release.all | Medium (changes existing targets) |
| `.github/actions/commit-and-push-with-retry/action.yml` | Add bump-command | Low (new optional input) |
| `.github/workflows/publish-ts-client.yml` | Add bump-command | Low (backward compatible) |
| `.github/workflows/publish-app-bindings.yml` | Add bump-command | Low (backward compatible) |
| `.github/workflows/release.yml` | Add bump-command | Low (backward compatible) |

---

## Testing Strategy (When Implemented)

### Unit Tests (Primitives)
- Test each primitive in isolation
- Mock git commands
- Verify error handling

### Integration Tests (Individual Releases)
1. Test `release-ts-client` on test branch
2. Verify tag creation, push, workflow trigger
3. Confirm version bump works
4. Repeat for all components

### System Tests (release.all)
1. Create test branches
2. Run `release.all` end-to-end
3. Verify:
   - All 4 tags on same commit
   - Atomic push (all or nothing)
   - Concurrent workflow execution
   - Version bumps don't conflict
   - Cleanup on failure

### Regression Tests
- Confirm existing release scripts still work
- Validate CI pipelines unchanged
- Check documentation examples

---

## Rollout Strategy (Future)

### Phase 1: Internal Testing
- Deploy to test repository
- Run through release cycles
- Gather feedback

### Phase 2: Opt-in Rollout
- Announce new target availability
- Keep old targets unchanged
- Document migration path

### Phase 3: Gradual Adoption
- Teams adopt at their pace
- Monitor for issues
- Iterate based on feedback

### Phase 4: Standardization (Optional)
- Once proven stable
- Deprecate old individual workflows
- Update all documentation

---

## Appendix: Detailed Implementation

See plan file for complete Makefile/workflow code:
- Primitive implementations
- Refactored release targets
- release.all implementation
- Workflow modifications

---

## Conclusion

This specification documents a comprehensive refactoring of BodhiApp's release system to enable atomic multi-component releases while improving maintainability through decoupled primitives. The proposal addresses real pain points (concurrent bumps, manual coordination) but requires careful consideration due to changes to existing workflows.

**Status**: Deferred pending stakeholder review and risk assessment. Plan preserved here for future reference and potential iterative implementation.

---

## References

- Current release system: `Makefile` lines 92-153
- Release utilities: `scripts/release.mk`
- Workflows: `.github/workflows/publish-*.yml`, `release.yml`
- Version scripts: `scripts/get_npm_version.js`, `scripts/get_github_release_version.js`, `scripts/get_ghcr_version.py`
