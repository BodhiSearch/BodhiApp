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
	@pwsh -NoProfile -File scripts/clean.win.ps1

ci.coverage:
	pwsh -NoProfile -File scripts/coverage.win.ps1

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