# Plan: Add `release.all` Target (Non-Refactoring)

## Overview

Add a new `make release.all` target that creates and pushes tags for all components atomically, WITHOUT modifying existing release targets.

## Design Decisions

| Decision | Choice |
|----------|--------|
| Approach | Additive only - no changes to existing targets |
| Function reuse | Use existing release.mk functions |
| Validation | Use check_git_branch once at start |
| Tag prompts | Use delete_tag_if_exists per tag (4 prompts) |
| Final confirmation | Yes, before creating/pushing |
| Failure cleanup | Local tags only |
| Bump fix | Include workflow changes |

## Components Released

- `ts-client/vX.Y.Z` - TypeScript client
- `bodhi-app-bindings/vX.Y.Z` - NAPI bindings
- `app/vX.Y.Z` - Native desktop app
- `docker/vX.Y.Z` - Production Docker

**Excluded**: docker-dev, website

---

## Phase add-release-all: Add Unified Target

**File**: `Makefile`

```makefile
release.all: ## Release all components: ts-client, app-bindings, app, docker
	@echo "=========================================="
	@echo "       Unified Release: All Components"
	@echo "=========================================="
	$(call check_git_branch)
	@echo ""
	@echo "Fetching current versions..."
	@TS_CUR=$$($(call get_npm_version,@bodhiapp/ts-client)) && \
	AB_CUR=$$($(call get_npm_version,@bodhiapp/app-bindings)) && \
	APP_CUR=$$($(call get_app_version)) && \
	DOCK_CUR=$$($(call get_ghcr_docker_version,production)) && \
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
	echo "Checking existing tags..." && \
	$(call delete_tag_if_exists,$$TS_TAG) && \
	$(call delete_tag_if_exists,$$AB_TAG) && \
	$(call delete_tag_if_exists,$$APP_TAG) && \
	$(call delete_tag_if_exists,$$DOCK_TAG) && \
	echo "" && \
	read -p "Create and push all release tags? [y/N] " confirm && \
	[ "$$confirm" = "y" ] || { echo "Aborted."; exit 1; } && \
	echo "" && \
	echo "Creating tags on current commit..." && \
	git tag "$$TS_TAG" && \
	git tag "$$AB_TAG" && \
	git tag "$$APP_TAG" && \
	git tag "$$DOCK_TAG" && \
	ALL_TAGS="$$TS_TAG $$AB_TAG $$APP_TAG $$DOCK_TAG" && \
	echo "Pushing all tags atomically..." && \
	if ! git push origin $$ALL_TAGS; then \
		echo "Push failed! Cleaning up local tags..." && \
		git tag -d "$$TS_TAG" 2>/dev/null || true && \
		git tag -d "$$AB_TAG" 2>/dev/null || true && \
		git tag -d "$$APP_TAG" 2>/dev/null || true && \
		git tag -d "$$DOCK_TAG" 2>/dev/null || true && \
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

**Also add to `.PHONY`**:
```makefile
.PHONY: ... release.all ...
```

---

## Phase bump-command: Fix Concurrent Version Bumps

**File**: `.github/actions/commit-and-push-with-retry/action.yml`

Add new input:
```yaml
inputs:
  # ... existing inputs ...
  bump-command:
    description: "Command to regenerate changes on retry (runs after rebase)"
    required: false
    default: ""
```

Replace "Push with retry logic" step:
```yaml
- name: Push with retry logic
  shell: bash
  run: |
    MAX_RETRIES=${{ inputs.max-retries }}
    BUMP_CMD="${{ inputs.bump-command }}"

    for i in $(seq 1 $MAX_RETRIES); do
      echo "Push attempt $i/$MAX_RETRIES"

      if git push origin ${{ inputs.branch }}; then
        echo "Successfully pushed to origin/${{ inputs.branch }}"
        exit 0
      fi

      if [[ $i -lt $MAX_RETRIES ]]; then
        echo "Push failed, rebasing and retrying..."

        # Reset the commit we just made
        git reset --hard HEAD~1

        # Sync with upstream
        if ! git pull --rebase origin ${{ inputs.branch }}; then
          echo "Error: Failed to rebase after failed push"
          exit 1
        fi

        # Re-run bump command if provided
        if [[ -n "$BUMP_CMD" ]]; then
          echo "Re-running bump command..."
          eval "$BUMP_CMD"
        fi

        # Re-add and commit
        git add ${{ inputs.files }}
        if ! git diff --cached --quiet; then
          git commit -m "${{ inputs.commit-message }}"
        else
          echo "No changes after re-bump, nothing to push"
          exit 0
        fi
      fi
    done

    echo "Error: Failed to push after $MAX_RETRIES attempts"
    exit 1
```

---

## Phase workflow-ts-client: Update publish-ts-client.yml

**File**: `.github/workflows/publish-ts-client.yml`

Change the "Commit and push version bump" step:
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump ts-client version to ${{ steps.version.outputs.next_version }} after release"
    files: "ts-client/package.json ts-client/package-lock.json"
    bump-command: "cd ts-client && npm version ${{ steps.version.outputs.next_version }} --no-git-tag-version"
```

---

## Phase workflow-app-bindings: Update publish-app-bindings.yml

**File**: `.github/workflows/publish-app-bindings.yml`

Change the "Commit and push version bump" step:
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump app-bindings version to ${{ steps.version.outputs.next_version }} after release"
    files: "crates/lib_bodhiserver_napi/package.json crates/lib_bodhiserver_napi/package-lock.json"
    bump-command: "cd crates/lib_bodhiserver_napi && npm version ${{ steps.version.outputs.next_version }} --no-git-tag-version && npm install"
```

---

## Phase workflow-release: Update release.yml

**File**: `.github/workflows/release.yml`

Change the "Commit and push version bump" step:
```yaml
- name: Commit and push version bump
  uses: ./.github/actions/commit-and-push-with-retry
  with:
    branch: main
    commit-message: "chore: bump version to ${{ needs.create-release.outputs.VERSION }}-dev after release"
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
| `Makefile` | Add `release.all` target, update `.PHONY` |
| `.github/actions/commit-and-push-with-retry/action.yml` | Add `bump-command` input, modify retry logic |
| `.github/workflows/publish-ts-client.yml` | Add `bump-command` to commit step |
| `.github/workflows/publish-app-bindings.yml` | Add `bump-command` to commit step |
| `.github/workflows/release.yml` | Add `bump-command` to commit step |

---

## Implementation Order

1. `Makefile` - Add `release.all` target
2. `.github/actions/commit-and-push-with-retry/action.yml` - Add bump-command
3. `.github/workflows/publish-ts-client.yml` - Update commit step
4. `.github/workflows/publish-app-bindings.yml` - Update commit step
5. `.github/workflows/release.yml` - Update commit step

---

## User Flow

```
$ make release.all

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
Pushing all tags atomically...

==========================================
           Release Complete
==========================================
Tags pushed:
  - ts-client/v0.1.11
  - bodhi-app-bindings/v0.0.23
  - app/v0.0.43
  - docker/v0.0.9

Workflows triggered:
  - publish-ts-client.yml
  - publish-app-bindings.yml
  - release.yml
  - publish-docker.yml + publish-docker-multiplatform.yml
```
