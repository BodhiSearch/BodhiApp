# Plan: Refactor Release System + Unified `release.all`

## Overview

Refactor existing release targets into decoupled primitives, then compose them for both individual releases and unified `release.all`.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Bash compatibility | POSIX-compatible (no bash 4+ features) |
| Old function names | Keep as thin aliases for compatibility |
| Version override | Removed (auto-increment only) |
| Existing tag handling | Abort entire operation on decline |
| [skip ci] in bumps | No - run CI on bump commits |
| Dirty working tree | Require clean (abort if uncommitted changes) |
| Interactive prompts | Always (no RELEASE_FORCE bypass) |
| Logging | None (terminal output only) |
| Tag check order | All checks upfront, single confirmation for deletes |

---

## Phase refactor-primitives: Extract Reusable Primitives

**File**: `scripts/release.mk`

### Primitive 1: `check_git_sync`
```makefile
# Validate: on main, synced with origin, clean working tree
define check_git_sync
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Working tree has uncommitted changes"; \
		git status --short; \
		exit 1; \
	fi && \
	CURRENT_BRANCH=$$(git branch --show-current) && \
	if [ "$$CURRENT_BRANCH" != "main" ]; then \
		read -p "Warning: Not on main (current: $$CURRENT_BRANCH). Continue? [y/N] " confirm && \
		[ "$$confirm" = "y" ] || { echo "Aborted."; exit 1; }; \
	fi && \
	echo "Fetching origin/main..." && \
	git fetch origin main && \
	LOCAL=$$(git rev-parse HEAD) && \
	REMOTE=$$(git rev-parse origin/main) && \
	if [ "$$LOCAL" != "$$REMOTE" ]; then \
		echo "Warning: Local differs from origin/main" && \
		echo "  Local:  $$LOCAL" && \
		echo "  Remote: $$REMOTE" && \
		read -p "Continue? [y/N] " confirm && \
		[ "$$confirm" = "y" ] || { echo "Aborted."; exit 1; }; \
	fi
endef
```

### Primitive 2: `get_component_version`
```makefile
# Usage: $(call get_component_version,TYPE,ARG)
define get_component_version
$(if $(filter npm,$(1)),./scripts/get_npm_version.js $(2),\
$(if $(filter github-release,$(1)),./scripts/get_github_release_version.js $(2),\
$(if $(filter ghcr,$(1)),./scripts/get_ghcr_version.py $(2),\
$(error Unknown version type: $(1)))))
endef

# Backwards-compatible aliases
define get_npm_version
$(call get_component_version,npm,$(1))
endef

define get_app_version
$(call get_component_version,github-release,BodhiSearch/BodhiApp)
endef

define get_ghcr_docker_version
$(call get_component_version,ghcr,$(1))
endef
```

### Primitive 3: `check_tag_exists`
```makefile
# Returns: "local", "remote", "both", or empty
define check_tag_exists
LOCAL_EXISTS=""; REMOTE_EXISTS=""; \
git rev-parse "$(1)" >/dev/null 2>&1 && LOCAL_EXISTS="yes"; \
git ls-remote --tags origin "refs/tags/$(1)" 2>/dev/null | grep -q . && REMOTE_EXISTS="yes"; \
if [ -n "$$LOCAL_EXISTS" ] && [ -n "$$REMOTE_EXISTS" ]; then echo "both"; \
elif [ -n "$$LOCAL_EXISTS" ]; then echo "local"; \
elif [ -n "$$REMOTE_EXISTS" ]; then echo "remote"; fi
endef
```

### Primitive 4-5: `delete_tag_remote` and `delete_tag_local`
```makefile
define delete_tag_remote
echo "Deleting remote tag $(1)..." && \
git push --delete origin "$(1)" 2>/dev/null || true
endef

define delete_tag_local
echo "Deleting local tag $(1)..." && \
git tag -d "$(1)" 2>/dev/null || true
endef
```

### Primitive 6: `handle_existing_tag` (POSIX-compatible)
```makefile
define handle_existing_tag
TAG_STATE=$$($(call check_tag_exists,$(1))); \
if [ -n "$$TAG_STATE" ]; then \
	echo "Tag $(1) exists ($$TAG_STATE)"; \
	read -p "Delete and recreate? [y/N] " confirm; \
	if [ "$$confirm" = "y" ]; then \
		if [ "$$TAG_STATE" = "both" ] || [ "$$TAG_STATE" = "remote" ]; then \
			$(call delete_tag_remote,$(1)); \
		fi; \
		if [ "$$TAG_STATE" = "both" ] || [ "$$TAG_STATE" = "local" ]; then \
			$(call delete_tag_local,$(1)); \
		fi; \
	else \
		echo "Aborted."; exit 1; \
	fi; \
fi
endef
```

### Primitive 7-8: `create_tag` and `push_tags_atomic`
```makefile
define create_tag
git tag "$(1)"
endef

define push_tags_atomic
git push origin $(1)
endef
```

---

## Phase refactor-individual: Refactor Individual Release Targets

**File**: `Makefile`

```makefile
release-ts-client: ## Release TypeScript client package
	@echo "=== Release: ts-client ==="
	$(call check_git_sync)
	@CURRENT=$$($(call get_component_version,npm,@bodhiapp/ts-client)) && \
	NEXT=$$($(call increment_version,$$CURRENT)) && \
	TAG="ts-client/v$$NEXT" && \
	echo "Version: $$CURRENT -> $$NEXT" && \
	$(call handle_existing_tag,$$TAG) && \
	$(call create_tag,$$TAG) && \
	$(call push_tags_atomic,$$TAG) && \
	echo "Tag $$TAG pushed."

release-app: ## Release native desktop app
	@echo "=== Release: app ==="
	$(call check_git_sync)
	@CURRENT=$$($(call get_component_version,github-release,BodhiSearch/BodhiApp)) && \
	NEXT=$$($(call increment_version,$$CURRENT)) && \
	TAG="app/v$$NEXT" && \
	echo "Version: $$CURRENT -> $$NEXT" && \
	$(call handle_existing_tag,$$TAG) && \
	$(call create_tag,$$TAG) && \
	$(call push_tags_atomic,$$TAG) && \
	echo "Tag $$TAG pushed."

release-app-bindings: ## Release app-bindings package
	@echo "=== Release: app-bindings ==="
	$(call check_git_sync)
	@CURRENT=$$($(call get_component_version,npm,@bodhiapp/app-bindings)) && \
	NEXT=$$($(call increment_version,$$CURRENT)) && \
	TAG="bodhi-app-bindings/v$$NEXT" && \
	echo "Version: $$CURRENT -> $$NEXT" && \
	$(call handle_existing_tag,$$TAG) && \
	$(call create_tag,$$TAG) && \
	$(call push_tags_atomic,$$TAG) && \
	echo "Tag $$TAG pushed."

release-docker: ## Release production Docker image
	@echo "=== Release: docker ==="
	$(call check_git_sync)
	@CURRENT=$$($(call get_component_version,ghcr,production)) && \
	NEXT=$$($(call increment_version,$$CURRENT)) && \
	TAG="docker/v$$NEXT" && \
	echo "Version: $$CURRENT -> $$NEXT" && \
	$(call handle_existing_tag,$$TAG) && \
	$(call create_tag,$$TAG) && \
	$(call push_tags_atomic,$$TAG) && \
	echo "Tag $$TAG pushed."
```

---

## Phase release-all: Unified Release Target

**File**: `Makefile`

```makefile
release.all: ## Release all components atomically
	@echo "=========================================="
	@echo "       Unified Release: All Components"
	@echo "=========================================="
	$(call check_git_sync)
	@echo ""
	@echo "Fetching current versions..."
	@TS_CUR=$$($(call get_component_version,npm,@bodhiapp/ts-client)) && \
	AB_CUR=$$($(call get_component_version,npm,@bodhiapp/app-bindings)) && \
	APP_CUR=$$($(call get_component_version,github-release,BodhiSearch/BodhiApp)) && \
	DOCK_CUR=$$($(call get_component_version,ghcr,production)) && \
	TS_NEXT=$$($(call increment_version,$$TS_CUR)) && \
	AB_NEXT=$$($(call increment_version,$$AB_CUR)) && \
	APP_NEXT=$$($(call increment_version,$$APP_CUR)) && \
	DOCK_NEXT=$$($(call increment_version,$$DOCK_CUR)) && \
	TS_TAG="ts-client/v$$TS_NEXT" && \
	AB_TAG="bodhi-app-bindings/v$$AB_NEXT" && \
	APP_TAG="app/v$$APP_NEXT" && \
	DOCK_TAG="docker/v$$DOCK_NEXT" && \
	echo "" && \
	echo "Release Plan:" && \
	echo "  ts-client:     $$TS_CUR -> $$TS_NEXT" && \
	echo "  app-bindings:  $$AB_CUR -> $$AB_NEXT" && \
	echo "  app:           $$APP_CUR -> $$APP_NEXT" && \
	echo "  docker:        $$DOCK_CUR -> $$DOCK_NEXT" && \
	echo "" && \
	echo "Checking for existing tags..." && \
	EXISTING_TAGS="" && \
	TS_STATE=$$($(call check_tag_exists,$$TS_TAG)) && \
	AB_STATE=$$($(call check_tag_exists,$$AB_TAG)) && \
	APP_STATE=$$($(call check_tag_exists,$$APP_TAG)) && \
	DOCK_STATE=$$($(call check_tag_exists,$$DOCK_TAG)) && \
	if [ -n "$$TS_STATE" ]; then EXISTING_TAGS="$$EXISTING_TAGS $$TS_TAG($$TS_STATE)"; fi && \
	if [ -n "$$AB_STATE" ]; then EXISTING_TAGS="$$EXISTING_TAGS $$AB_TAG($$AB_STATE)"; fi && \
	if [ -n "$$APP_STATE" ]; then EXISTING_TAGS="$$EXISTING_TAGS $$APP_TAG($$APP_STATE)"; fi && \
	if [ -n "$$DOCK_STATE" ]; then EXISTING_TAGS="$$EXISTING_TAGS $$DOCK_TAG($$DOCK_STATE)"; fi && \
	if [ -n "$$EXISTING_TAGS" ]; then \
		echo "Existing tags found:$$EXISTING_TAGS" && \
		read -p "Delete all and recreate? [y/N] " confirm && \
		if [ "$$confirm" = "y" ]; then \
			if [ -n "$$TS_STATE" ]; then \
				if [ "$$TS_STATE" = "both" ] || [ "$$TS_STATE" = "remote" ]; then $(call delete_tag_remote,$$TS_TAG); fi; \
				if [ "$$TS_STATE" = "both" ] || [ "$$TS_STATE" = "local" ]; then $(call delete_tag_local,$$TS_TAG); fi; \
			fi && \
			if [ -n "$$AB_STATE" ]; then \
				if [ "$$AB_STATE" = "both" ] || [ "$$AB_STATE" = "remote" ]; then $(call delete_tag_remote,$$AB_TAG); fi; \
				if [ "$$AB_STATE" = "both" ] || [ "$$AB_STATE" = "local" ]; then $(call delete_tag_local,$$AB_TAG); fi; \
			fi && \
			if [ -n "$$APP_STATE" ]; then \
				if [ "$$APP_STATE" = "both" ] || [ "$$APP_STATE" = "remote" ]; then $(call delete_tag_remote,$$APP_TAG); fi; \
				if [ "$$APP_STATE" = "both" ] || [ "$$APP_STATE" = "local" ]; then $(call delete_tag_local,$$APP_TAG); fi; \
			fi && \
			if [ -n "$$DOCK_STATE" ]; then \
				if [ "$$DOCK_STATE" = "both" ] || [ "$$DOCK_STATE" = "remote" ]; then $(call delete_tag_remote,$$DOCK_TAG); fi; \
				if [ "$$DOCK_STATE" = "both" ] || [ "$$DOCK_STATE" = "local" ]; then $(call delete_tag_local,$$DOCK_TAG); fi; \
			fi; \
		else \
			echo "Aborted."; exit 1; \
		fi; \
	fi && \
	echo "" && \
	read -p "Create and push all release tags? [y/N] " confirm && \
	[ "$$confirm" = "y" ] || { echo "Aborted."; exit 1; } && \
	echo "Creating tags on current commit..." && \
	$(call create_tag,$$TS_TAG) && \
	$(call create_tag,$$AB_TAG) && \
	$(call create_tag,$$APP_TAG) && \
	$(call create_tag,$$DOCK_TAG) && \
	ALL_TAGS="$$TS_TAG $$AB_TAG $$APP_TAG $$DOCK_TAG" && \
	echo "Pushing all tags atomically..." && \
	if ! $(call push_tags_atomic,$$ALL_TAGS); then \
		echo "Push failed! Cleaning up local tags..." && \
		$(call delete_tag_local,$$TS_TAG); \
		$(call delete_tag_local,$$AB_TAG); \
		$(call delete_tag_local,$$APP_TAG); \
		$(call delete_tag_local,$$DOCK_TAG); \
		exit 1; \
	fi && \
	echo "" && \
	echo "==========================================" && \
	echo "           Release Complete" && \
	echo "==========================================" && \
	echo "Tags pushed:" && \
	echo "  - $$TS_TAG" && \
	echo "  - $$AB_TAG" && \
	echo "  - $$APP_TAG" && \
	echo "  - $$DOCK_TAG" && \
	echo "" && \
	echo "Workflows triggered:" && \
	echo "  - publish-ts-client.yml" && \
	echo "  - publish-app-bindings.yml" && \
	echo "  - release.yml" && \
	echo "  - publish-docker.yml + publish-docker-multiplatform.yml"
```

---

## Phase workflow-bump-retry: Fix Concurrent Version Bump

**File**: `.github/actions/commit-and-push-with-retry/action.yml`

Add `bump-command` input:
```yaml
inputs:
  bump-command:
    description: "Command to regenerate changes on retry (runs after rebase)"
    required: false
    default: ""
```

Modify retry loop:
```yaml
- name: Push with retry logic
  shell: bash
  run: |
    MAX_RETRIES=${{ inputs.max-retries }}
    BUMP_CMD="${{ inputs.bump-command }}"

    for i in $(seq 1 $MAX_RETRIES); do
      echo "Push attempt $i/$MAX_RETRIES"

      if git push origin ${{ inputs.branch }}; then
        echo "Successfully pushed"
        exit 0
      fi

      if [[ $i -lt $MAX_RETRIES ]]; then
        echo "Push failed. Rebasing and regenerating..."
        git reset --hard HEAD~1
        git pull --rebase origin ${{ inputs.branch }}

        if [[ -n "$BUMP_CMD" ]]; then
          echo "Re-running bump command..."
          eval "$BUMP_CMD"
        fi

        git add ${{ inputs.files }}
        if ! git diff --cached --quiet; then
          git commit -m "${{ inputs.commit-message }}"
        else
          echo "No changes after re-bump"
          exit 0
        fi
      fi
    done

    echo "Failed after $MAX_RETRIES attempts"
    exit 1
```

---

## Phase workflow-updates: Update Workflows

### `publish-ts-client.yml`
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump ts-client version to ${{ steps.version.outputs.next_version }} after release"
    files: "ts-client/package.json ts-client/package-lock.json"
    bump-command: "cd ts-client && npm version ${{ steps.version.outputs.next_version }} --no-git-tag-version"
```

### `publish-app-bindings.yml`
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump app-bindings version to ${{ steps.version.outputs.next_version }} after release"
    files: "crates/lib_bodhiserver_napi/package.json crates/lib_bodhiserver_napi/package-lock.json"
    bump-command: "cd crates/lib_bodhiserver_napi && npm version ${{ steps.version.outputs.next_version }} --no-git-tag-version && npm install"
```

### `release.yml`
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump version to next dev after release"
    files: "-A"
    bump-command: |
      NEXT_VERSION=$(./scripts/increment_version.js "${{ needs.create-release.outputs.VERSION }}" patch dev)
      make ci.update-version VERSION="$NEXT_VERSION"
      cargo update --workspace
```

---

## Files to Modify

| File | Changes |
|------|---------|
| `scripts/release.mk` | Replace with 8 decoupled primitives + backwards-compat aliases |
| `Makefile` | Refactor 4 release targets, add `release.all`, update `.PHONY` |
| `.github/actions/commit-and-push-with-retry/action.yml` | Add `bump-command`, modify retry |
| `.github/workflows/publish-ts-client.yml` | Add `bump-command` |
| `.github/workflows/publish-app-bindings.yml` | Add `bump-command` |
| `.github/workflows/release.yml` | Add `bump-command` |

---

## Implementation Order

1. `scripts/release.mk` - Add primitives + aliases
2. `Makefile` - Refactor individual release targets
3. Test individual targets
4. `Makefile` - Add `release.all`
5. `.github/actions/commit-and-push-with-retry` - Add bump-command
6. Workflow files - Update to use bump-command
7. End-to-end test

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                       release.all                            │
├──────────────────────────────────────────────────────────────┤
│  1. check_git_sync (once) - branch + remote + clean tree    │
│  2. get_component_version × 4                                │
│  3. increment_version × 4                                    │
│  4. check_tag_exists × 4 (upfront)                           │
│  5. Single prompt: "Delete all existing? [y/N]"              │
│  6. delete_tag_remote/local for existing (remote first)      │
│  7. create_tag × 4 (on same commit)                          │
│  8. push_tags_atomic (all together)                          │
│  9. On failure: delete_tag_local × 4 (cleanup)               │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                  release-{component}                         │
├──────────────────────────────────────────────────────────────┤
│  1. check_git_sync                                           │
│  2. get_component_version                                    │
│  3. increment_version                                        │
│  4. handle_existing_tag (check + prompt + delete)            │
│  5. create_tag                                               │
│  6. push_tags_atomic                                         │
└──────────────────────────────────────────────────────────────┘
```
