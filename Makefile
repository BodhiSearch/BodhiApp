.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test: ## Run all tests (Rust, Node, Python)
	cargo test
	cargo test -p bodhi --features native
	cd crates/bodhi && npm install && npm test
	cd crates/lib_bodhiserver_napi && npm install && npm run test
# 	cd openai-pysdk-compat && poetry run pytest || true

format: ## Format code in all projects (Rust, Node, Python)
	cargo fmt --all
	cd crates/bodhi && npm run format
	cd openai-pysdk-compat && poetry run ruff format .

format.all: format ## Format code in all projects (Rust, Node, Python), and run Clippy
	cargo clippy --fix --allow-dirty --allow-staged

ci.clean: ## Clean all cargo packages
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo clean $$PACKAGES

ci.coverage: ## Run coverage in CI environment
	$(MAKE) coverage

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

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.app-npm ci.ui ci.ts-client-check ci.ts-client-test ts-client release-app-bindings ui.test help
