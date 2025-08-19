# Makefile for lib_bodhiserver_napi
.PHONY: all build build-debug build-release test test-js test-js-config test-js-integration test-js-live test-playwright test-playwright-ui test-playwright-headed test-playwright-ci clean format-js dev install-js

# Default target
all: build test

# Install JavaScript dependencies
install-js:
	npm install

build: install-js
	npm run build

build-release: install-js
	npm run build:release

# Test targets
test: test-rust test-js test-playwright

test-rust:
	cargo test

test-js: build-debug install-js
	npm run test:run

test-js-config: build-debug install-js
	npm run test:config

test-js-integration: build-debug install-js
	npm run test:integration

test-js-live: build-debug install-js
	npm run test:live

# Run Playwright browser tests
test-playwright: build-debug install-js
	npm run test:playwright -- --reporter=line

# Run Playwright tests with UI mode
test-playwright-ui: build-debug install-js
	npm run test:playwright:ui -- --reporter=line

# Run Playwright tests in headed mode (visible browser)
test-playwright-headed: build-debug install-js
	npm run test:playwright:headed -- --reporter=line

# Run Playwright tests in CI mode
test-playwright-ci: build-debug install-js
	npm run test:playwright:ci

# Format all code
format: format-rust format-js

format-rust:
	cargo fmt

format-js: install-js
	npm run format

# Quick development cycle
dev: format-rust build-debug test-js-config

# Clean targets
clean: clean-rust clean-js

clean-rust:
	cargo clean

clean-js:
	rm -rf node_modules test-results playwright-report coverage .nyc_output 