# Windows-specific Makefile for pwsh

test:
	cargo test
	pwsh -Command "Push-Location crates/bodhi; npm test -- --run; Pop-Location"
	pwsh -Command "Push-Location openai-pysdk-compat; poetry run pytest; exit 0"

format:
	pwsh -Command "Push-Location crates/bodhi; npm run format; npm run lint; Pop-Location"
	cargo fmt --all
	pwsh -Command "Push-Location openai-pysdk-compat; poetry run ruff format .; Pop-Location"

ci.clean:
	@pwsh -NoProfile -File scripts/clean.win.ps1

ci.coverage:
	pwsh -NoProfile -File scripts/coverage.win.ps1

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@pwsh -Command "\
	Get-ChildItem -Path crates/*,crates/bodhi/src-tauri -Directory | ForEach-Object { \
		$$cargoFile = Join-Path $$_ 'Cargo.toml'; \
		if (Test-Path $$cargoFile) { \
			(Get-Content $$cargoFile) -replace '^version = .*', 'version = \"$(VERSION)\"' | Set-Content $$cargoFile; \
		} \
	}"

ci.build:
	pwsh -Command "Push-Location crates/bodhi/src-tauri; \
	if ($$env:TARGET) { \
		cargo tauri build --target $$env:TARGET --ci --config '{\"tauri\": {\"updater\": {\"active\": false}}}'; \
	} else { \
		cargo tauri build --ci --config '{\"tauri\": {\"updater\": {\"active\": false}}}'; \
	}"

ci.app-pnpm:
	pwsh -Command "Push-Location crates/bodhi; pnpm install; Pop-Location"

ci.ui:
	pwsh -Command "Push-Location crates/bodhi; pnpm run test run --coverage; Pop-Location"

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.app-pnpm ci.ui