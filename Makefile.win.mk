# Windows-specific Makefile for pwsh

test:
	cargo test
	pwsh -Command "Push-Location crates/bodhiapp; npm test -- --run; Pop-Location"
	pwsh -Command "Push-Location openai-pysdk-compat; poetry run pytest; exit 0"

format:
	pwsh -Command "Push-Location crates/bodhiapp; npm run format; npm run lint; Pop-Location"
	cargo fmt --all
	pwsh -Command "Push-Location openai-pysdk-compat; poetry run ruff format .; Pop-Location"

ci.clean:
	@pwsh -Command "$$CRATES='llamacpp-sys'; \
	Get-ChildItem -Path crates/* -Directory | ForEach-Object { \
		if (Test-Path \"$$_/Cargo.toml\") { \
			$$CRATES=\"$$CRATES $$(Split-Path -Leaf $$_)\"; \
		} \
	}; \
	$$CRATES_ARRAY = $$CRATES -split ' '; \
	cargo clean $$($$CRATES_ARRAY | ForEach-Object { \"-p $$_\" })"

ci.coverage:
	pwsh -Command "cargo llvm-cov clean"
	pwsh -Command "$$CRATES=''; \
	Get-ChildItem -Path crates/* -Directory | ForEach-Object { \
		if ((Test-Path \"$$_/Cargo.toml\") -and ((Split-Path -Leaf $$_) -ne 'integration-tests')) { \
			$$CRATES=\"$$CRATES -p $$(Split-Path -Leaf $$_)\"; \
		} \
	}; \
	cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info $$CRATES"

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@pwsh -Command "\
	Get-ChildItem -Path crates/*,crates/bodhiapp/src-tauri -Directory | ForEach-Object { \
		$$cargoFile = Join-Path $$_ 'Cargo.toml'; \
		if (Test-Path $$cargoFile) { \
			(Get-Content $$cargoFile) -replace '^version = .*', 'version = \"$(VERSION)\"' | Set-Content $$cargoFile; \
		} \
	}"

ci.build:
	pwsh -Command "Push-Location crates/bodhiapp/src-tauri; \
	if ($$env:TARGET) { \
		cargo tauri build --target $$env:TARGET --ci --config '{\"tauri\": {\"updater\": {\"active\": false}}}'; \
	} else { \
		cargo tauri build --ci --config '{\"tauri\": {\"updater\": {\"active\": false}}}'; \
	}"

ci.setup-vercel-ai:
	pwsh -Command "Push-Location vercel-ai; pnpm recursive install --frozen-lockfile; \
	Push-Location packages/core; pnpm install --frozen-lockfile; Pop-Location; \
	pnpm run build --filter=ai...; Pop-Location"

ci.app-pnpm:
	pwsh -Command "Push-Location crates/bodhiapp; pnpm install; Pop-Location"

.PHONY: test format ci.clean ci.coverage ci.update-version ci.build ci.setup-vercel-ai ci.app-pnpm