# Node.js Bindings for BodhiServer
.PHONY: help all build build-debug build-release test test-js test-js-config test-js-integration test-js-live test-playwright test-playwright-ui test-playwright-headed test-playwright-ci clean format-js dev install-js download-extension
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo 'Node.js Bindings for BodhiServer'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

all: build test ## Build and test the Node.js bindings

# Install JavaScript dependencies
install-js: ## Install JavaScript dependencies
	npm install

build: install-js ## Build Node.js bindings (development)
	npm run build

build-debug: install-js ## Build Node.js bindings in debug mode
	npm run build

build-release: install-js ## Build Node.js bindings (release/optimized)
	npm run build:release

test: test-rust test-js test-playwright ## Run all tests (Rust, JS, Playwright)

test.ui: build-debug install-js
	npm run test:all 

test-rust: ## Run Rust unit tests
	cargo test

test-js: build-debug install-js ## Run JavaScript unit tests
	npm run test:run

test-js-config: build-debug install-js ## Run JavaScript configuration tests
	npm run test:config

test-js-integration: build-debug install-js ## Run JavaScript integration tests
	npm run test:integration

test-js-live: build-debug install-js ## Run JavaScript live server tests
	npm run test:live

test-playwright: build-debug install-js ## Run Playwright browser tests
	npm run test:playwright

test-playwright-ui: build-debug install-js ## Run Playwright tests with UI mode
	npm run test:playwright:ui -- --reporter=line

test-playwright-headed: build-debug install-js ## Run Playwright tests in headed mode (visible browser)
	npm run test:playwright:headed -- --reporter=line

test-playwright-ci: build-debug install-js ## Run Playwright tests in CI mode
	npm run test:playwright:ci

format: format-rust format-js ## Format all code (Rust and JavaScript)

format-rust: ## Format Rust code
	cargo fmt

format-js: install-js ## Format JavaScript/TypeScript code
	npm run format

dev: format-rust build-debug test-js-config ## Quick development cycle

clean: clean-rust clean-js ## Clean all build artifacts

clean-rust: ## Clean Rust build artifacts
	$(MAKE) -C ../.. build.ui-clean

clean-js: ## Clean JavaScript build artifacts and dependencies
	rm -rf node_modules test-results playwright-report coverage .nyc_output app-bindings.*.node

download-extension: ## Download Bodhi browser extension (use FORCE=1 to check for updates)
	@node scripts/download-extension.mjs $(if $(FORCE),--force,)