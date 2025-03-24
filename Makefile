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
	@cd ts-client && \
	VERSION=$$(node -p "require('./package.json').version.replace(/-dev$$/,'')") && \
	echo "Creating tag ts-client/v$$VERSION..." && \
	git tag "ts-client/v$$VERSION" && \
	git push origin "ts-client/v$$VERSION" && \
	echo "Tag ts-client/v$$VERSION pushed. GitHub workflow will handle the release process."

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

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.app-pnpm ci.ui ci.ts-client-check ci.ts-client-test help
