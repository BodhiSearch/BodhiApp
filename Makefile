.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test.backend:
	cargo test
	cargo test -p bodhi --features native

test.ui:
	cd crates/bodhi && npm install && npm test
	$(MAKE) -C crates/lib_bodhiserver_napi test.ui

test.napi:
	cd crates/lib_bodhiserver_napi && npm install && npm run test:all

test: test.backend test.ui test.napi

format: ## Format code in all projects (Rust, Node, Python)
	cargo fmt --all
	cd crates/bodhi && npm run format
	cd crates/lib_bodhiserver_napi && npm run format
	# cd openai-pysdk-compat && poetry run ruff format .

format.all: format ## Format code in all projects (Rust, Node, Python), and run Clippy
	cargo clippy --fix --allow-dirty --allow-staged

ci.clean: ## Clean all cargo packages
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo clean $$PACKAGES

ci.coverage: ## Run coverage in CI environment
	$(MAKE) coverage

clean.ui: ## Clean UI
	rm -rf crates/bodhi/out
	cargo clean -p lib_bodhiserver -p bodhi && rm -rf crates/lib_bodhiserver_napi/app-bindings.*.node

build.ui:
	cd crates/bodhi && npm run build
	cd crates/lib_bodhiserver_napi && npm run build

rebuild.ui: clean.ui build.ui

coverage: ## Generate code coverage report
	cargo build -p llama_server_proc; \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo llvm-cov test --no-fail-fast --all-features $$PACKAGES --lcov --output-path lcov.info

ci.update-version: ## Update version in all Cargo.toml files
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@for dir in crates/* crates/bodhi/src-tauri; do \
		if [ -f $$dir/Cargo.toml ]; then \
			sed -i.bak "s/^version = .*/version = \"$(VERSION)\"/" $$dir/Cargo.toml && \
			rm $$dir/Cargo.toml.bak; \
		fi \
	done

ci.build: ## Build the Tauri application
	cd crates/bodhi/src-tauri && \
	cargo tauri build $${TARGET:+--target $${TARGET}} --ci --config '{"tauri": {"updater": {"active": false}}}'

ci.app-npm: ## Install npm dependencies for the app
	cd crates/bodhi && npm install

ci.ui: ## Run UI tests with coverage
	cd crates/bodhi && npm run test
	$(MAKE) -C crates/lib_bodhiserver_napi test.ui

release-ts-client: ## Release TypeScript client package
	@echo "Preparing to release ts-client package..."
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
	fi && \
	cd ts-client && \
	CURRENT_VERSION=$$(npm view @bodhiapp/ts-client version 2>/dev/null || echo "0.0.0") && \
	IFS='.' read -r MAJOR MINOR PATCH <<< "$$CURRENT_VERSION" && \
	NEXT_VERSION="$$MAJOR.$$MINOR.$$((PATCH + 1))" && \
	echo "Current version on npmjs: $$CURRENT_VERSION" && \
	echo "Next version to release: $$NEXT_VERSION" && \
	TAG_NAME="ts-client/v$$NEXT_VERSION" && \
	echo "Cleaning up any existing tag $$TAG_NAME..." && \
	git tag -d "$$TAG_NAME" 2>/dev/null || true && \
	git push --delete origin "$$TAG_NAME" 2>/dev/null || true && \
	echo "Creating new tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "Tag $$TAG_NAME pushed. GitHub workflow will handle the release process."

ci.ts-client-check: ## Verify ts-client is up to date with openapi.yml
	@echo "==> Checking ts-client is up to date with openapi spec"
	@cargo run --package xtask openapi
	@cd ts-client && npm ci && npm run generate
	@if [ -n "$$(git status --porcelain ts-client/src)" ]; then \
		echo "Error: Found uncommitted changes in ts-client/src after generating OpenAPI spec and ts-client types."; \
		echo "Please run 'cargo run --package xtask openapi && cd ts-client && npm run generate' and commit the changes."; \
		git status ts-client/src; \
		exit 1; \
	fi
	@echo "✓ ts-client is up to date with OpenAPI spec"

ci.ts-client-test: ## Run ts-client tests
	@echo "==> Running ts-client tests"
	@cd ts-client && npm ci && npm run generate && npm test

ts-client: ## Build the TypeScript types package
	@echo "==> Building ts-client package"
	@cd ts-client && npm install && npm run build && npm run test && npm run bundle
	@echo "✓ ts-client package built successfully"

# Function to check git branch status
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

# Function to safely delete existing tag
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

release-app-bindings: ## Create and push tag for app-bindings package release
	@echo "Preparing to release @bodhiapp/app-bindings package..."
	$(call check_git_branch)
	@CURRENT_VERSION=$$(npm view @bodhiapp/app-bindings version 2>/dev/null || echo "0.0.0") && \
	IFS='.' read -r MAJOR MINOR PATCH <<< "$$CURRENT_VERSION" && \
	NEXT_VERSION="$$MAJOR.$$MINOR.$$((PATCH + 1))" && \
	echo "Current version on npmjs: $$CURRENT_VERSION" && \
	echo "Next version to release: $$NEXT_VERSION" && \
	TAG_NAME="bodhi-app-bindings/v$$NEXT_VERSION" && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "Creating tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "Tag $$TAG_NAME pushed. GitHub workflow will handle the release process."

ui.test:
	cd crates/bodhi && npm run test

# Docker build targets - delegated to devops/Makefile
docker.dev.cpu: ## Build CPU image for current platform (use PLATFORM to override)
	@$(MAKE) -C devops dev.cpu BUILD_VARIANT=$${BUILD_VARIANT:-development} PLATFORM=$${PLATFORM}

docker.dev.cpu.amd64: ## Build AMD64 CPU image for local testing
	@$(MAKE) -C devops dev.cpu.amd64 BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.dev.cpu.arm64: ## Build ARM64 CPU image for local testing
	@$(MAKE) -C devops dev.cpu.arm64 BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.dev.cuda: ## Build NVIDIA CUDA GPU image
	@$(MAKE) -C devops dev.cuda BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.amd64: ## Run locally built linux/amd64 Docker image
	@$(MAKE) -C devops run VARIANT=$${VARIANT:-cpu} ARCH=$${ARCH:-amd64} BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.arm64: ## Run locally built linux/arm64 Docker image
	@$(MAKE) -C devops run VARIANT=$${VARIANT:-cpu} ARCH=$${ARCH:-arm64} BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.list: ## List all locally built BodhiApp images
	@$(MAKE) -C devops list-images

docker.clean: ## Remove all locally built BodhiApp images
	@$(MAKE) -C devops clean

run.app: ## Run the BodhiApp
	BODHI_ENCRYPTION_KEY=dummy-key \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.cache/bodhi-dev-20250820 \
		cargo run --bin bodhi -- serve --port 1135

# Note: CI/Release targets below are for CI pipeline use only
# For local Docker builds, use the docker.build.* targets above

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

# Function to check git branch status
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

# Function to safely delete existing tag
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

release-docker: ## Create and push tag for production Docker image release
	@echo "Preparing to release production Docker image..."
	$(call check_git_branch)
	@echo "Fetching latest production release version from GHCR..."
	@CURRENT_VERSION=$$($(call get_ghcr_docker_version,production)) && \
	if [ "$$CURRENT_VERSION" = "0.0.0" ]; then \
		echo "No existing production releases found in GHCR, starting with version 0.0.1"; \
	fi && \
	$(call create_docker_release_tag,production,$$CURRENT_VERSION)

release-docker-dev: ## Create and push tag for development Docker image release
	@echo "Preparing to release development Docker image..."
	$(call check_git_branch)
	@echo "Fetching latest development release version from GHCR..."
	@CURRENT_VERSION=$$($(call get_ghcr_docker_version,development)) && \
	if [ "$$CURRENT_VERSION" = "0.0.0" ]; then \
		echo "No existing development releases found in GHCR, starting with version 0.0.1"; \
	fi && \
	$(call create_docker_release_tag,development,$$CURRENT_VERSION)

check-docker-versions: ## Check latest versions of both production and development Docker images from GHCR
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

update-context-symlinks: ## Update symlinks in ai-docs/context for CLAUDE.md and PACKAGE.md files
	@echo "Updating AI context symlinks..."
	@python3 scripts/update_context_symlinks.py

update-context-symlinks-dry-run: ## Preview changes that would be made to AI context symlinks
	@echo "Previewing AI context symlinks changes..."
	@python3 scripts/update_context_symlinks.py --dry-run --verbose

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.app-npm ci.ui ci.ts-client-check ci.ts-client-test ts-client release-app-bindings ui.test docker.dev.cpu docker.dev.cpu.amd64 docker.dev.cpu.arm64 docker.dev.cuda docker.run docker.list docker.clean release-docker release-docker-dev check-docker-versions update-context-symlinks update-context-symlinks-dry-run help
