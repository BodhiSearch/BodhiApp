test:
	cargo test
	cd bodhi-ui && npm test -- --run
	cd openai-pysdk-compat && poetry run pytest || true

format:
	cd bodhi-ui && npm run format && npm run lint
	cargo fmt --all
	cd openai-pysdk-compat && poetry run ruff format .

ci.clean:
	@CRATES="bodhi llama-server-bindings"; \
	for dir in crates/*; do \
		if [ -f $$dir/Cargo.toml ]; then \
			CRATES="$$CRATES $$(basename $$dir)"; \
		fi \
	done; \
	cargo clean $$(for crate in $$CRATES; do echo -n " -p $$crate"; done)

ci.coverage:
	cargo llvm-cov clean
	@CRATES=""; \
	for dir in crates/*; do \
		if [ -f $$dir/Cargo.toml ]; then \
			CRATES="$$CRATES -p $$(basename $$dir)"; \
		fi \
	done; \
	cargo llvm-cov nextest --all-features --lcov --output-path lcov.info $$CRATES

ci.update-version:
	@echo "Updating version to $(VERSION) in Cargo.toml files"
	@for dir in crates/* app/bodhi; do \
		if [ -f $$dir/Cargo.toml ]; then \
			sed -i.bak "s/^version = .*/version = \"$(VERSION)\"/" $$dir/Cargo.toml && \
			rm $$dir/Cargo.toml.bak; \
		fi \
	done

.PHONY: test format ci.clean ci.coverage
