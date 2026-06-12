# Unified Release Target - No-Refactor Implementation

**Date**: 2026-01-10
**Status**: ✅ IMPLEMENTED

## Quick Links

- **[SPEC.md](./SPEC.md)** - Complete specification with context, implementation details, and user guide
- **[IMPLEMENTATION-PLAN.md](./IMPLEMENTATION-PLAN.md)** - Detailed technical implementation plan used during development

---

## Overview

This directory documents the **implemented** unified `make release.all` target that atomically releases all BodhiApp components on the same commit. This is the **non-refactoring** approach that was chosen and implemented.

## Key Characteristics

- ✅ **Implemented and Working**
- ✅ **Additive Only** - No changes to existing release targets
- ✅ **Zero Risk** - All existing functionality unchanged
- ✅ **Concurrent Bump Fix** - Workflows handle conflicts gracefully

---

## Quick Start

```bash
# Release all components atomically
make release.all
```

This single command:
1. Validates git state
2. Fetches versions for ts-client, app-bindings, app, docker
3. Shows release plan (current→next versions)
4. Prompts for confirmation
5. Creates all 4 tags on current commit
6. Pushes all tags atomically
7. Triggers all 4 release workflows

---

## What Was Implemented

### 1. New `release.all` Target
- **File**: `Makefile` (lines 155-219)
- **Status**: ✅ Implemented
- Releases 4 components atomically: ts-client, app-bindings, app, docker

### 2. Concurrent Version Bump Fix
- **File**: `.github/actions/commit-and-push-with-retry/action.yml`
- **Added**: `bump-command` input parameter
- **Enhanced**: Retry logic with fresh version recalculation

### 3. Workflow Updates
- ✅ `publish-ts-client.yml` - Added `bump-command`
- ✅ `publish-app-bindings.yml` - Added `bump-command`
- ✅ `release.yml` - Added `bump-command`

---

## Quick Start

```bash
# Release all components atomically
make release.all
```

The command will:
1. Validate git state (on main, synced)
2. Show current→next versions
3. Check/prompt for existing tags
4. Final confirmation
5. Create all 4 tags on same commit
6. Push atomically
7. Trigger all workflows

---

## Key Differences from Refactoring Approach

| Feature | This Implementation | Deferred Refactoring |
|---------|-------------------|---------------------|
| Existing targets | ✅ Unchanged | ❌ Modified |
| Risk | ✅ Minimal | ⚠️ Medium |
| Implementation time | ✅ 1 hour | 4-6 hours |
| Testing required | ✅ Light | Heavy |
| Rollback | ✅ Trivial | Complex |
| Code duplication | Some | None |
| Adoption | Optional | Forced |

**Decision**: Implemented non-refactoring approach for quick delivery with zero risk.

---

See [SPEC.md](./SPEC.md) for complete details.
