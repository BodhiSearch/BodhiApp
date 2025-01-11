test:
	cargo test
	cd crates/bodhi && npm test -- --run
	cd openai-pysdk-compat && poetry run pytest || true

format:
	cd crates/bodhi && npm run format && npm run lint
	cargo fmt --all
	cd openai-pysdk-compat && poetry run ruff format .

ci.clean:
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo clean $$PACKAGES

ci.coverage:
	$(MAKE) coverage

coverage:
	cargo build -p llama_server_proc; \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo llvm-cov test --no-fail-fast --all-features $$PACKAGES --lcov --output-path lcov.info

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@for dir in crates/* crates/bodhi/src-tauri; do \
		if [ -f $$dir/Cargo.toml ]; then \
			sed -i.bak "s/^version = .*/version = \"$(VERSION)\"/" $$dir/Cargo.toml && \
			rm $$dir/Cargo.toml.bak; \
		fi \
	done

ci.build:
	cd crates/bodhi/src-tauri && \
	cargo tauri build $${TARGET:+--target $${TARGET}} --ci --config '{"tauri": {"updater": {"active": false}}}'

ci.app-pnpm:
	cd crates/bodhi && pnpm install

ci.ui:
	cd crates/bodhi && pnpm run test run --coverage

.PHONY: test format coverage ci.clean ci.coverage ci.update-version ci.build ci.ui
