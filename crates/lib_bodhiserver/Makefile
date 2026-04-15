# E2E (Playwright) suite for BodhiApp running against the bodhiserver_dev binary.
.PHONY: help install-js test test.e2e test.e2e.standalone test.e2e.multi_tenant format download-extension clean
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install-js: ## Install JavaScript dependencies
	npm install

test: test.e2e ## Default target: run full Playwright matrix

test.e2e: install-js ## Run Playwright tests against both projects
	npm run test:playwright

test.e2e.standalone: install-js ## Run only the standalone (SQLite) project
	npm run test:playwright:standalone

test.e2e.multi_tenant: install-js ## Run only the multi_tenant (Postgres) project
	npm run test:playwright:multi_tenant

format: install-js ## Format JS sources
	npm run format

download-extension: install-js ## Download the Bodhi browser extension (FORCE=1 to refresh)
	@node scripts/download-extension.mjs $(if $(FORCE),--force,)

clean: ## Remove JS build artifacts
	rm -rf node_modules test-results playwright-report coverage .nyc_output
