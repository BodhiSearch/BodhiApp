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
	@pwsh -Command "$$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name; \
	cargo clean $($$packages | ForEach-Object { \"-p $_\" })"

ci.coverage:
	pwsh -Command "Write-Host 'Getting package list...'; \
		$$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages | \
			Where-Object { $$_.name -ne 'integration-tests' } | \
			Select-Object -ExpandProperty name; \
		Write-Host 'Packages to test:' $$packages; \
		$$crates = $$packages | ForEach-Object { \"-p $$_\".Trim() }; \
		$$cmd = \"cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info $$crates\"; \
		Write-Host 'Executing command:' $$cmd; \
		Invoke-Expression $$cmd"

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