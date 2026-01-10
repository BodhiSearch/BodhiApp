# Unified Release Target - Specification

**Date**: 2026-01-10
**Status**: DEFERRED

## Quick Links

- **[SPEC.md](./SPEC.md)** - High-level specification with context, rationale, and trade-offs
- **[IMPLEMENTATION-PLAN.md](./IMPLEMENTATION-PLAN.md)** - Detailed technical implementation plan

## Overview

This specification proposes a unified `make release.all` target that atomically releases all BodhiApp components (ts-client, app-bindings, native app, Docker) on the same commit with a single atomic git push.

## Status: DEFERRED

Implementation is deferred due to concerns about refactoring existing release targets. The plan documents the proposed approach for future consideration.

## Key Features

1. **Atomic multi-component releases** - All tags on same commit
2. **Decoupled primitives** - Maintainable, testable release functions
3. **Concurrent bump fix** - Resolves GitHub workflow conflicts
4. **Backwards compatible** - Preserves existing function names as aliases

## Problem Being Solved

### Current Pain Points

- Manual coordination required for full releases
- Tags created on different commits (timing skew)
- Duplicate validation logic across targets
- Concurrent version bump conflicts in CI

### Proposed Solution

- Single command: `make release.all`
- Refactored primitives: `check_git_sync`, `get_component_version`, etc.
- Atomic push: `git push origin tag1 tag2 tag3 tag4`
- Retry with re-bump: Workflows regenerate version on conflict

## Why Deferred?

Changes existing release targets which requires careful review:
- Risk assessment needed
- Testing strategy must be comprehensive
- Migration path should be planned
- Documentation updates required

## Components Covered

| Component | Current Target | Tag Format | Registry |
|-----------|----------------|------------|----------|
| TypeScript client | `release-ts-client` | `ts-client/vX.Y.Z` | npm |
| NAPI bindings | `release-app-bindings` | `bodhi-app-bindings/vX.Y.Z` | npm |
| Native app | `release-app` | `app/vX.Y.Z` | GitHub |
| Docker (prod) | `release-docker` | `docker/vX.Y.Z` | GHCR |

**Not included**: `release-docker-dev` (optional), `website.release` (special timing dependencies)

## Architecture

```
Primitive Layer (scripts/release.mk)
  ↓
Individual Releases (Makefile)
  ↓
Unified release.all (Makefile)
```

## Files That Would Be Modified

- `scripts/release.mk` - Add 8 decoupled primitives
- `Makefile` - Refactor 4 targets, add `release.all`
- `.github/actions/commit-and-push-with-retry/action.yml` - Add `bump-command`
- `.github/workflows/publish-ts-client.yml` - Use `bump-command`
- `.github/workflows/publish-app-bindings.yml` - Use `bump-command`
- `.github/workflows/release.yml` - Use `bump-command`

## Design Decisions

- **Bash compatibility**: POSIX-compatible (macOS bash 3.2)
- **Backwards compat**: Old function names kept as aliases
- **Version override**: Removed (auto-increment only)
- **Tag handling**: Abort entire operation if user declines delete
- **CI behavior**: Run CI on bump commits (no `[skip ci]`)
- **Working tree**: Require clean (abort if uncommitted changes)
- **Interactivity**: Always interactive (no force mode)

## Next Steps (When Revisited)

1. Review specification with stakeholders
2. Assess risk vs. benefit
3. Consider alternative approaches (additive vs. refactor)
4. Plan phased rollout if approved
5. Develop comprehensive test strategy

## Questions or Feedback?

This spec was created via detailed interview process covering:
- Versioning strategy (independent vs. synchronized)
- Tag atomicity (single push vs. sequential)
- Post-release bump handling (retry strategy)
- Component selection (all vs. configurable)
- Failure handling (cleanup, rollback)
- Compatibility requirements (POSIX, aliases)
- Security considerations (no force mode)

For questions or to reopen discussion, reference this spec directory.
