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

update_releases: ## Update .env.release_urls and releases.json from latest GitHub releases (desktop + Docker variants)
	@npm run update_releases

update_releases.check: ## Check latest releases (dry-run) - validates desktop and Docker variant discovery
	@npm run update_releases:check

build: ## Build website (auto-syncs docs first via prebuild)
	@npm run build

format: ## Format code
	@npm run format

lint: ## Lint code
	@npm run lint

lint.fix: ## Lint code and fix errors
	@npm run lint:fix

# Include shared release utilities from parent directory
include ../Makefile.release.mk

release.precheck: sync.docs.check ## Pre-release validation checks
	@echo "==> Running release prechecks..."
	@echo "  ✓ Docs sync check passed"
	@echo ""
	@echo "==> Checking release URLs are up-to-date..."
	@if ! $(MAKE) update_releases.check >/dev/null 2>&1; then \
		echo "⚠ Release URLs check failed. Latest app releases may not be reflected."; \
		read -p "Continue with release anyway? [y/N] " confirm; \
		if [ "$$confirm" != "y" ]; then \
			echo "Aborting release. Run 'make update_releases' to update URLs first."; \
			exit 1; \
		fi \
	fi
	@echo "  ✓ Release URLs check passed (or user confirmed to proceed)"
	@echo ""
	@echo "✓ All prechecks passed"

release: release.precheck ## Create and push tag for website release
	@echo "==> Preparing website release..."
	$(call check_git_branch)
	@echo ""
	@echo "==> Fetching current deployed version from https://getbodhi.app/version.json..."
	@CURRENT_VERSION=$$(node scripts/get-version.js) && \
	NEXT_VERSION=$$(node ../scripts/increment_version.js $$CURRENT_VERSION) && \
	echo "Current deployed version: $$CURRENT_VERSION" && \
	echo "Next version to release: $$NEXT_VERSION" && \
	echo "" && \
	TAG_NAME="getbodhi.app/v$$NEXT_VERSION" && \
	$(call delete_tag_if_exists,$$TAG_NAME) && \
	echo "" && \
	echo "Creating tag $$TAG_NAME..." && \
	git tag "$$TAG_NAME" && \
	git push origin "$$TAG_NAME" && \
	echo "" && \
	echo "✓ Tag $$TAG_NAME pushed successfully!" && \
	echo "" && \
	echo "Next steps:" && \
	echo "  1. Monitor workflow: https://github.com/BodhiSearch/BodhiApp/actions/workflows/deploy-website.yml" && \
	echo "  2. After deploy completes, verify: https://getbodhi.app/version.json" && \
	echo "  3. The workflow will automatically bump main branch to $$NEXT_VERSION-dev"

.PHONY: help \
	sync.docs sync.docs.check \
	update_releases update_releases.check \
	release.precheck release \
	build \
	format format.fix \
	lint lint.fix
