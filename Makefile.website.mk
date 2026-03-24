# Website and Documentation Targets
# This file contains website build/deploy and documentation sync targets

.PHONY: docs.context-update docs.context-update-dry-run \
	website.update-releases website.update-releases-check website.release

# Documentation context management
docs.context-update: ## Update symlinks in ai-docs/context for CLAUDE.md and PACKAGE.md files
	@echo "Updating AI context symlinks..."
	@python3 scripts/update_context_symlinks.py

docs.context-update-dry-run: ## Preview changes that would be made to AI context symlinks
	@echo "Previewing AI context symlinks changes..."
	@python3 scripts/update_context_symlinks.py --dry-run --verbose

# Website release URLs
website.update-releases: ## Update website release URLs from latest releases
	$(MAKE) -C getbodhi.app update_releases

website.update-releases-check: ## Check latest releases (dry-run)
	$(MAKE) -C getbodhi.app update_releases.check

website.release: ## Create and push tag for website release
	$(MAKE) -C getbodhi.app release
