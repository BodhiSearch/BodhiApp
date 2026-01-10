# Release Targets and Utilities
# This file contains release targets and shared release utility functions

.PHONY: release.ts-client release.app release.app-bindings \
	release.docker release.docker-dev release.all docker.version-check

release.ts-client: ## Release TypeScript client package
	@echo "Preparing to release ts-client package..."
	$(call check_git_branch)
	@CURRENT_VERSION=$$($(call get_npm_version,@bodhiapp/ts-client)) && \
	NEXT_VERSION=$$($(call increment_version,$$CURRENT_VERSION)) && \
	echo "Current version on npmjs: $$CURRENT_VERSION" && \
	echo "Next version to release: $$NEXT_VERSION" && \
	TAG_NAME="ts-client/v$$NEXT_VERSION" && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "Creating new tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "Tag $$TAG_NAME pushed. GitHub workflow will handle the release process."

release.app: ## Create and push tag for native app release
	@echo "Preparing to release native app..."
	$(call check_git_branch)
	@CURRENT_VERSION=$$($(call get_app_version)) && \
	NEXT_VERSION=$$($(call increment_version,$$CURRENT_VERSION)) && \
	echo "Current version from GitHub releases: $$CURRENT_VERSION" && \
	echo "Next version: $$NEXT_VERSION" && \
	TAG_NAME="app/v$$NEXT_VERSION" && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "Creating tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "Tag $$TAG_NAME pushed. GitHub workflow will handle the release."

release.app-bindings: ## Create and push tag for app-bindings package release
	@echo "Preparing to release @bodhiapp/app-bindings package..."
	$(call check_git_branch)
	@CURRENT_VERSION=$$($(call get_npm_version,@bodhiapp/app-bindings)) && \
	NEXT_VERSION=$$($(call increment_version,$$CURRENT_VERSION)) && \
	echo "Current version on npmjs: $$CURRENT_VERSION" && \
	echo "Next version to release: $$NEXT_VERSION" && \
	TAG_NAME="bodhi-app-bindings/v$$NEXT_VERSION" && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "Creating tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "Tag $$TAG_NAME pushed. GitHub workflow will handle the release process."

release.docker: ## Create and push tag for production Docker image release (use DOCKER_VERSION=docker/vX.Y.Z to override)
	@echo "Preparing to release production Docker image..."
	$(call check_git_branch)
	@if [ -n "$(DOCKER_VERSION)" ]; then \
		echo "Using manually specified version: $(DOCKER_VERSION)"; \
		if ! echo "$(DOCKER_VERSION)" | grep -qE '^docker/v[0-9]+\.[0-9]+\.[0-9]+$$'; then \
			echo "Error: DOCKER_VERSION must be in format docker/vX.Y.Z (e.g., docker/v0.0.5)"; \
			exit 1; \
		fi; \
		TAG_NAME="$(DOCKER_VERSION)"; \
		$(call delete_tag_if_exists,$$TAG_NAME); \
		echo "Creating production release tag $$TAG_NAME..."; \
		git tag "$$TAG_NAME"; \
		git push origin "$$TAG_NAME"; \
		echo "Production release tag $$TAG_NAME pushed. GitHub workflow will handle the Docker image build and publish."; \
	else \
		echo "Fetching latest production release version from GHCR..."; \
		CURRENT_VERSION=$$($(call get_ghcr_docker_version,production)); \
		if [ "$$CURRENT_VERSION" = "0.0.0" ]; then \
			echo "No existing production releases found in GHCR, starting with version 0.0.1"; \
		fi; \
		$(call create_docker_release_tag,production,$$CURRENT_VERSION); \
	fi

release.docker-dev: ## Create and push tag for development Docker image release (use DOCKER_DEV_VERSION=docker-dev/vX.Y.Z to override)
	@echo "Preparing to release development Docker image..."
	$(call check_git_branch)
	@if [ -n "$(DOCKER_DEV_VERSION)" ]; then \
		echo "Using manually specified version: $(DOCKER_DEV_VERSION)"; \
		if ! echo "$(DOCKER_DEV_VERSION)" | grep -qE '^docker-dev/v[0-9]+\.[0-9]+\.[0-9]+$$'; then \
			echo "Error: DOCKER_DEV_VERSION must be in format docker-dev/vX.Y.Z (e.g., docker-dev/v0.0.5)"; \
			exit 1; \
		fi; \
		TAG_NAME="$(DOCKER_DEV_VERSION)"; \
		$(call delete_tag_if_exists,$$TAG_NAME); \
		echo "Creating development release tag $$TAG_NAME..."; \
		git tag "$$TAG_NAME"; \
		git push origin "$$TAG_NAME"; \
		echo "Development release tag $$TAG_NAME pushed. GitHub workflow will handle the Docker image build and publish."; \
	else \
		echo "Fetching latest development release version from GHCR..."; \
		CURRENT_VERSION=$$($(call get_ghcr_docker_version,development)); \
		if [ "$$CURRENT_VERSION" = "0.0.0" ]; then \
			echo "No existing development releases found in GHCR, starting with version 0.0.1"; \
		fi; \
		$(call create_docker_release_tag,development,$$CURRENT_VERSION); \
	fi

docker.version-check: ## Check latest versions of both production and development Docker images from GHCR
	@echo "=== Latest Docker Release Versions (from GHCR) ==="
	@echo "Production releases (bodhiapp - all variants: cpu, cuda, rocm, vulkan):"
	@PROD_VERSION=$$($(call get_ghcr_docker_version,production)) && \
	if [ "$$PROD_VERSION" = "0.0.0" ]; then \
		echo "  Latest: No releases found"; \
	else \
		echo "  Latest: $$PROD_VERSION (across all variants)"; \
	fi
	@echo ""
	@echo "Development releases (bodhiapp - all variants: cpu, cuda, rocm, vulkan):"
	@DEV_VERSION=$$($(call get_ghcr_docker_version,development)) && \
	if [ "$$DEV_VERSION" = "0.0.0" ]; then \
		echo "  Latest: No releases found"; \
	else \
		echo "  Latest: $$DEV_VERSION (across all variants)"; \
	fi
	@echo "==============================="

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
	echo "Pushing tags sequentially (to trigger GitHub Actions)..." && \
	FAILED_TAGS="" && \
	SUCCEEDED_TAGS="" && \
	for tag in $$ALL_TAGS; do \
		echo "  Pushing $$tag..." && \
		if git push origin "$$tag"; then \
			echo "  ✓ $$tag pushed" && \
			SUCCEEDED_TAGS="$$SUCCEEDED_TAGS $$tag"; \
		else \
			echo "  ✗ Failed to push $$tag" && \
			FAILED_TAGS="$$FAILED_TAGS $$tag"; \
		fi && \
		sleep 1; \
	done && \
	echo "" && \
	echo "==========================================" && \
	if [ -z "$$FAILED_TAGS" ]; then \
		echo "           Release Complete" && \
		echo "==========================================" && \
		echo "Tags successfully pushed:" && \
		for tag in $$SUCCEEDED_TAGS; do \
			echo "  ✓ $$tag"; \
		done && \
		echo "" && \
		echo "Workflows triggered:" && \
		echo "  - publish-ts-client.yml" && \
		echo "  - publish-app-bindings.yml" && \
		echo "  - release.yml" && \
		echo "  - publish-docker.yml + publish-docker-multiplatform.yml"; \
	else \
		echo "     Release Completed with Failures" && \
		echo "==========================================" && \
		if [ -n "$$SUCCEEDED_TAGS" ]; then \
			echo "Tags successfully pushed:" && \
			for tag in $$SUCCEEDED_TAGS; do \
				echo "  ✓ $$tag"; \
			done && \
			echo ""; \
		fi && \
		echo "Tags that failed to push:" && \
		for tag in $$FAILED_TAGS; do \
			echo "  ✗ $$tag"; \
		done && \
		echo "" && \
		echo "Please manually push failed tags:" && \
		for tag in $$FAILED_TAGS; do \
			echo "  git push origin $$tag"; \
		done && \
		echo "" && \
		exit 1; \
	fi

# Shared Release Utility Functions
# These define blocks provide reusable functions for release targets above

# Check git branch status (ensure on main and synced)
define check_git_branch
	@CURRENT_BRANCH=$$(git branch --show-current) && \
	if [ "$$CURRENT_BRANCH" != "main" ]; then \
		read -p "Warning: You are not on main branch (current: $$CURRENT_BRANCH). Continue? [y/N] " confirm && \
		if [ "$$confirm" != "y" ]; then \
			echo "Aborting release." && exit 1; \
		fi \
	fi && \
	echo "Fetching latest changes from remote..." && \
	git fetch origin main && \
	LOCAL_HEAD=$$(git rev-parse HEAD) && \
	REMOTE_HEAD=$$(git rev-parse origin/main) && \
	if [ "$$LOCAL_HEAD" != "$$REMOTE_HEAD" ]; then \
		echo "Warning: Your local main branch is different from origin/main" && \
		echo "Local:  $$LOCAL_HEAD" && \
		echo "Remote: $$REMOTE_HEAD" && \
		read -p "Continue anyway? [y/N] " confirm && \
		if [ "$$confirm" != "y" ]; then \
			echo "Aborting release." && exit 1; \
		fi \
	fi
endef

# Delete existing tag if exists
define delete_tag_if_exists
	echo "Checking for existing tag $(1)..." && \
	if git rev-parse "$(1)" >/dev/null 2>&1; then \
		read -p "Tag $(1) already exists. Delete and recreate? [y/N] " confirm && \
		if [ "$$confirm" = "y" ]; then \
			echo "Deleting existing tag $(1)..." && \
			git tag -d "$(1)" 2>/dev/null || true && \
			git push --delete origin "$(1)" 2>/dev/null || true; \
		else \
			echo "Aborting release." && exit 1; \
		fi \
	fi
endef

# Get NPM package version using existing get-npm-version.js pattern
define get_npm_version
	./scripts/get_npm_version.js $(1) 2>/dev/null || echo "0.0.0"
endef

# Get latest git tag version with prefix using bash
define get_git_tag_version
	LATEST_TAG=$$(git tag -l "$(1)*" --sort=-version:refname | head -n1 2>/dev/null); if [ -z "$$LATEST_TAG" ]; then echo "0.0.0"; else echo "$$LATEST_TAG" | sed "s/^$(1)//"; fi
endef

# Get latest app version from GitHub releases (handles v* to app/v* migration)
define get_app_version
	./scripts/get_github_release_version.js BodhiSearch/BodhiApp
endef

# Increment patch version using existing increment-version.js pattern
define increment_version
	./scripts/increment_version.js $(1)
endef

# Function to get current version from GHCR for Docker images across all variants
# Usage: $(call get_ghcr_docker_version,variant)
# variant: production or development
# Uses Python script for better maintainability
define get_ghcr_docker_version
	./scripts/get_ghcr_version.py $(1)
endef

# Function to increment version and create Docker release tag
# Usage: $(call create_docker_release_tag,variant,current-version)
# variant: production or development
# current-version: X.Y.Z format
define create_docker_release_tag
	if [ "$(2)" = "null" ] || [ -z "$(2)" ]; then \
		CURRENT_VERSION="0.0.0"; \
	else \
		CURRENT_VERSION="$(2)"; \
	fi && \
	IFS='.' read -r MAJOR MINOR PATCH <<< "$$CURRENT_VERSION" && \
	NEXT_VERSION="$$MAJOR.$$MINOR.$$((PATCH + 1))" && \
	echo "Current $(1) version (from GHCR): $$CURRENT_VERSION" && \
	echo "Next $(1) version: $$NEXT_VERSION" && \
	if [ "$(1)" = "production" ]; then \
		TAG_NAME="docker/v$$NEXT_VERSION"; \
	else \
		TAG_NAME="docker-dev/v$$NEXT_VERSION"; \
	fi && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "Creating $(1) release tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "$(1) release tag $$TAG_NAME pushed. GitHub workflow will handle the Docker image build and publish."
endef