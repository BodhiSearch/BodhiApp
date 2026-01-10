# CI and Workflow Trigger Targets
# This file contains CI-specific targets and workflow trigger targets

.PHONY: ci.clean ci.coverage ci.build-only ci.build ci.app-npm \
	ci.ui.unit ci.ui.integration ci.ui ci.update-version \
	ci.ts-client-check ci.ts-client-test \
	trigger.docker trigger.docker-dev trigger.ts-client \
	trigger.bodhi-app-bindings trigger.app trigger.website

ci.clean: ## Clean all cargo packages
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo clean $$PACKAGES

ci.coverage: ## Run coverage in CI environment
	$(MAKE) test.coverage

ci.build-only: ## Build without running tests for faster CI
	cargo build -p llama_server_proc; \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo build --all-features $$PACKAGES

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
