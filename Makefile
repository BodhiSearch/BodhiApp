.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

test: ## Run all tests (Rust, Node, Python)
	cargo test
	cd crates/bodhi && npm test -- --run
	cd openai-pysdk-compat && poetry run pytest || true

format: ## Format code in all projects (Rust, Node, Python)
	cd crates/bodhi && npm run format && npm run lint
	cargo fmt --all
	cd openai-pysdk-compat && poetry run ruff format .

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

ci.app-pnpm: ## Install pnpm dependencies for the app
	cd crates/bodhi && pnpm install

ci.ui: ## Run UI tests with coverage
	cd crates/bodhi && pnpm run test run --coverage

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
	echo "Creating tag ts-client/v$$NEXT_VERSION..." && \
	git tag "ts-client/v$$NEXT_VERSION" && \
	git push origin "ts-client/v$$NEXT_VERSION" && \
	echo "Tag ts-client/v$$NEXT_VERSION pushed. GitHub workflow will handle the release process."

ci.ts-client-check: ## Verify ts-client is up to date with openapi.yml
	@echo "==> Checking ts-client is up to date with openapi spec"
	@cargo run --package xtask openapi
	@cd ts-client && npm ci && npm run generate
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "Error: Found uncommitted changes after generating OpenAPI spec and ts-client types."; \
		echo "Please run 'cargo run --package xtask openapi && cd ts-client && npm run generate' and commit the changes."; \
		git status; \
		exit 1; \
	fi
	@echo "✓ ts-client is up to date with OpenAPI spec"

ci.ts-client-test: ## Run ts-client tests
	@echo "==> Running ts-client tests"
	@cd ts-client && npm ci && npm run generate && npm test

ts-client: ## Build the TypeScript types package
	@echo "==> Building ts-client package"
	@cd ts-client && npm install && npm run build
	@echo "✓ ts-client package built successfully"

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.app-pnpm ci.ui ci.ts-client-check ci.ts-client-test ts-client help
