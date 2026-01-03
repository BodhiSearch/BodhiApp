# Include shared release utilities
include scripts/release.mk

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
	$(MAKE) -C getbodhi.app format

format.all: format ## Format code in all projects (Rust, Node, Python), and run Clippy
	cargo clippy --fix --allow-dirty --allow-staged

ci.clean: ## Clean all cargo packages
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo clean $$PACKAGES

ci.coverage: ## Run coverage in CI environment
	$(MAKE) coverage

ci.build-only: ## Build without running tests for faster CI
	cargo build -p llama_server_proc; \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo build --all-features $$PACKAGES

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

ci.update-version: ## Update version in all Cargo.toml, package.json, and tauri.conf.json files
	@echo "Updating version to $(VERSION)"
	@echo "  - Updating Cargo.toml files..."
	@for dir in crates/* crates/bodhi/src-tauri; do \
		if [ -f $$dir/Cargo.toml ]; then \
			sed -i.bak "s/^version = .*/version = \"$(VERSION)\"/" $$dir/Cargo.toml && \
			rm $$dir/Cargo.toml.bak; \
		fi \
	done
	@echo "  - Updating package.json..."
	@cd crates/bodhi && npm version $(VERSION) --no-git-tag-version --allow-same-version
	@echo "  - Updating tauri.conf.json..."
	@jq '.version = "$(VERSION)"' crates/bodhi/src-tauri/tauri.conf.json > crates/bodhi/src-tauri/tauri.conf.json.tmp && \
		mv crates/bodhi/src-tauri/tauri.conf.json.tmp crates/bodhi/src-tauri/tauri.conf.json
	@echo "✓ Version updated to $(VERSION) in all files"

ci.build: ## Build the Tauri application
	cd crates/bodhi/src-tauri && \
	cargo tauri build $${TARGET:+--target $${TARGET}} --ci --config '{"tauri": {"updater": {"active": false}}}'

ci.app-npm: ## Install npm dependencies for the app
	cd crates/bodhi && npm install

ci.ui.unit: ## Run UI tests with coverage
	cd crates/bodhi && npm run test

ci.ui.integration: ## Run UI tests with coverage
	$(MAKE) -C crates/lib_bodhiserver_napi test.ui

ci.ui: ci.ui.unit ci.ui.integration

release-ts-client: ## Release TypeScript client package
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

release-app: ## Create and push tag for native app release
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

release-app-bindings: ## Create and push tag for app-bindings package release
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

app.clear: ## Clear the app data folders
	rm -rf ~/.cache/bodhi-dev-makefile

app.run: ## Run the BodhiApp
	BODHI_ENCRYPTION_KEY=dummy-key \
		HF_HOME=~/.cache/huggingface \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.cache/bodhi-dev-makefile \
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


release-docker: ## Create and push tag for production Docker image release (use DOCKER_VERSION=docker/vX.Y.Z to override)
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

release-docker-dev: ## Create and push tag for development Docker image release (use DOCKER_DEV_VERSION=docker-dev/vX.Y.Z to override)
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

trigger.docker: ## Trigger Docker workflow for latest docker/v* tag
	@./scripts/trigger_workflow.js docker

trigger.docker-dev: ## Trigger Docker workflow for latest docker-dev/v* tag
	@./scripts/trigger_workflow.js docker-dev

trigger.ts-client: ## Trigger ts-client workflow for latest tag
	@./scripts/trigger_workflow.js ts-client

trigger.bodhi-app-bindings: ## Trigger app-bindings workflow for latest tag
	@./scripts/trigger_workflow.js bodhi-app-bindings

trigger.app: ## Trigger app release workflow for latest tag
	@./scripts/trigger_workflow.js app

trigger.website: ## Trigger website deploy workflow for latest tag
	@./scripts/trigger_workflow.js website

update-context-symlinks: ## Update symlinks in ai-docs/context for CLAUDE.md and PACKAGE.md files
	@echo "Updating AI context symlinks..."
	@python3 scripts/update_context_symlinks.py

update-context-symlinks-dry-run: ## Preview changes that would be made to AI context symlinks
	@echo "Previewing AI context symlinks changes..."
	@python3 scripts/update_context_symlinks.py --dry-run --verbose

extension.download: ## Download Bodhi browser extension for testing (use FORCE=1 to check for updates)
	@$(MAKE) -C crates/lib_bodhiserver_napi download-extension FORCE=$(FORCE)

# Website documentation sync
sync.docs: ## Sync documentation from embedded app to website
	$(MAKE) -C getbodhi.app sync.docs

sync.docs.check: ## Check if website docs are in sync (release gate)
	$(MAKE) -C getbodhi.app sync.docs.check

# Website release URLs
website.update_releases: ## Update website release URLs from latest releases
	$(MAKE) -C getbodhi.app update_releases

website.update_releases.check: ## Check latest releases (dry-run)
	$(MAKE) -C getbodhi.app update_releases.check

website.release: ## Create and push tag for website release
	$(MAKE) -C getbodhi.app release

.PHONY: test format coverage \
	ci.clean ci.coverage ci.update-version ci.build ci.app-npm ci.ui \
	ci.ts-client-check ci.ts-client-test ts-client \
	release-app release-app-bindings \
	ui.test \
	docker.dev.cpu docker.dev.cpu.amd64 docker.dev.cpu.arm64 docker.dev.cuda \
	docker.run docker.list docker.clean \
	release-docker release-docker-dev check-docker-versions \
	trigger.docker trigger.docker-dev trigger.ts-client trigger.bodhi-app-bindings trigger.app trigger.website \
	update-context-symlinks update-context-symlinks-dry-run \
	extension.download \
	sync.docs sync.docs.check \
	website.update_releases website.update_releases.check website.release \
	help
