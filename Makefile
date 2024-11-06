test:
	cargo test
	cd crates/bodhiapp && npm test -- --run
	cd openai-pysdk-compat && poetry run pytest || true

format:
	cd crates/bodhiapp && npm run format && npm run lint
	cargo fmt --all
	cd openai-pysdk-compat && poetry run ruff format .

ci.clean:
	cargo clean

ci.coverage:
	cargo llvm-cov clean
	$(MAKE) coverage

coverage:
	@CRATES=""; \
	for dir in crates/*; do \
		if [ -f $$dir/Cargo.toml ] && [ "$$(basename $$dir)" != "integration-tests" ]; then \
			CRATES="$$CRATES -p $$(basename $$dir)"; \
		fi \
	done; \
	cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info $$CRATES

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@for dir in crates/* crates/bodhiapp/src-tauri; do \
		if [ -f $$dir/Cargo.toml ]; then \
			sed -i.bak "s/^version = .*/version = \"$(VERSION)\"/" $$dir/Cargo.toml && \
			rm $$dir/Cargo.toml.bak; \
		fi \
	done

ci.build:
	cd crates/bodhiapp/src-tauri && \
	cargo tauri build $${TARGET:+--target $${TARGET}} --ci --config '{"tauri": {"updater": {"active": false}}}'

ci.setup-vercel-ai:
	cd vercel-ai && pnpm recursive install --frozen-lockfile
	cd vercel-ai/packages/core && pnpm install --frozen-lockfile
	cd vercel-ai && pnpm run build --filter=ai...

ci.app-pnpm:
	cd crates/bodhiapp && pnpm install

.PHONY: test format ci.clean ci.coverage ci.update-version ci.build ci.setup-vercel-ai
