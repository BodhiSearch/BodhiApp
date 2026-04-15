# Node.js Bindings for BodhiServer (npm-publish only).
# E2E + Playwright targets live in crates/lib_bodhiserver/Makefile.
.PHONY: help all install-js build build-debug build-release test format clean
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo 'Node.js Bindings for BodhiServer'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

all: build test ## Build the npm package and run Rust tests

install-js: ## Install JavaScript dependencies
	npm install

build: install-js ## Build NAPI bindings (debug)
	npm run build

build-debug: build ## Alias for `build`

build-release: install-js ## Build NAPI bindings (release/optimized)
	npm run build:release

test: ## Run Rust tests for the NAPI crate
	cargo test -p lib_bodhiserver_napi

format: ## Format Rust code
	cargo fmt

clean: ## Remove built native artifacts
	rm -rf node_modules app-bindings.*.node
