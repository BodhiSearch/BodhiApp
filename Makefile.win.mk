# Windows-specific Makefile for PowerShell

test:
	cargo test
	cd crates/bodhiapp && npm test -- --run
	cd openai-pysdk-compat && poetry run pytest || true

format:
	cd crates/bodhiapp && npm run format && npm run lint
	cargo fmt --all
	cd openai-pysdk-compat && poetry run ruff format .

ci.clean:
	@powershell -Command "$$CRATES='llamacpp-sys'; \
	Get-ChildItem -Path crates/* -Directory | ForEach-Object { \
		if (Test-Path \"$$_/Cargo.toml\") { \
			$$CRATES=\"$$CRATES $$(Split-Path -Leaf $$_)\"; \
		} \
	}; \
	$$CRATES_ARRAY = $$CRATES -split ' '; \
	cargo clean $$($$CRATES_ARRAY | ForEach-Object { \"-p $$_\" })"

ci.coverage:
	cargo llvm-cov clean
	$(MAKE) -f Makefile.win.mk coverage SHELL=pwsh

coverage:
	powershell -Command "$$CRATES=''; \
	Get-ChildItem -Path crates/* -Directory | ForEach-Object { \
		if ((Test-Path \"$$_/Cargo.toml\") -and ((Split-Path -Leaf $$_) -ne 'integration-tests')) { \
			$$CRATES=\"$$CRATES -p $$(Split-Path -Leaf $$_)\"; \
		} \
	}; \
	cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info $$CRATES"

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@powershell -Command "\
	Get-ChildItem -Path crates/*,crates/bodhiapp/src-tauri -Directory | ForEach-Object { \
		$$cargoFile = Join-Path $$_ 'Cargo.toml'; \
		if (Test-Path $$cargoFile) { \
			(Get-Content $$cargoFile) -replace '^version = .*', 'version = \"$(VERSION)\"' | Set-Content $$cargoFile; \
		} \
	}"

ci.build:
	cd crates/bodhiapp/src-tauri && \
	if ($$env:TARGET) { \
		cargo tauri build --target $$env:TARGET --ci --config '{"tauri": {"updater": {"active": false}}}' \
	} else { \
		cargo tauri build --ci --config '{"tauri": {"updater": {"active": false}}}' \
	}

ci.setup-vercel-ai:
	cd vercel-ai && pnpm recursive install --frozen-lockfile
	cd vercel-ai/packages/core && pnpm install --frozen-lockfile
	cd vercel-ai && pnpm run build --filter=ai...

ci.app-pnpm:
	cd crates/bodhiapp && pnpm install

.PHONY: test format ci.clean ci.coverage ci.update-version ci.build ci.setup-vercel-ai ci.app-pnpm