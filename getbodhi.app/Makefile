.PHONY: help
help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

sync.docs: ## Sync docs from embedded app
	@npm run sync:docs

sync.docs.check: ## Check if docs are in sync
	@npm run sync:docs:check

update_releases: ## Update .env.release_urls from latest GitHub releases
	@npm run update_releases

update_releases.check: ## Check latest releases (dry-run)
	@npm run update_releases:check

build: ## Build website (auto-syncs docs first via prebuild)
	@npm run build

format: ## Format code
	@npm run format

lint: ## Lint code
	@npm run lint

lint.fix: ## Lint code and fix errors
	@npm run lint:fix

.PHONY: help \
	sync.docs sync.docs.check \
	update_releases update_releases.check \
	build \
	format format.fix \
	lint lint.fix
