.PHONY: sync.docs sync.docs.check build help

sync.docs: ## Sync docs from embedded app
	@npm run sync:docs

sync.docs.check: ## Check if docs are in sync
	@npm run sync:docs:check

build: ## Build website (auto-syncs docs first via prebuild)
	@npm run build

help: ## Show this help
	@grep -E '^[a-zA-Z_.-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'
