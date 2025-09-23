# scripts/release.mk - Shared release utilities for BodhiApp
# This file contains common functions used across all release targets

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