# Include shared utilities and domain-specific targets
include Makefile.release.mk
include Makefile.ci.mk
include Makefile.docker.mk
include Makefile.website.mk

.DEFAULT_GOAL := help

.PHONY: help test test.backend test.ui test.napi test.coverage \
	test.deps.up test.deps.down \
	build build.native build.ui build.ui-clean build.ui-rebuild build.ts-client \
	format format.all \
	run run.native app.clear app.run \
	test.extension-download test.model-download

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo '=== Testing ==='
	@awk 'BEGIN {FS = ":.*?## "} /^test[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Building ==='
	@awk 'BEGIN {FS = ":.*?## "} /^build[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Formatting ==='
	@awk 'BEGIN {FS = ":.*?## "} /^format[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== App Runtime ==='
	@awk 'BEGIN {FS = ":.*?## "} /^(run|app)[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Release ==='
	@awk 'BEGIN {FS = ":.*?## "} /^release[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Docker ==='
	@awk 'BEGIN {FS = ":.*?## "} /^docker[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Documentation ==='
	@awk 'BEGIN {FS = ":.*?## "} /^docs[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Website ==='
	@awk 'BEGIN {FS = ":.*?## "} /^website[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== CI/Workflow ==='
	@awk 'BEGIN {FS = ":.*?## "} /^(ci|trigger)[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo 'Run "make [target]" to execute a specific target'

test.deps.up: ## Start test dependencies (PostgreSQL via docker-compose)
	docker compose -f docker-compose-test-deps.yml up -d --wait

test.deps.down: ## Stop test dependencies and remove volumes
	docker compose -f docker-compose-test-deps.yml down -v

test.backend: test.deps.up ## Run Rust backend tests (requires Docker for PostgreSQL)
	cargo test
	cargo test -p bodhi --features native

test.ui.unit: ## Run frontend unit tests
	cd crates/bodhi && npm install && npm test

test.ui: ## Run frontend and UI integration tests
	$(MAKE) test.ui.unit
	$(MAKE) -C crates/lib_bodhiserver_napi test.ui

test.napi: ## Run NAPI bindings tests
	cd crates/lib_bodhiserver_napi && npm install && npm run test:all

test: test.backend test.ui test.napi ## Run all tests (backend, UI, NAPI)

format: ## Format code in all projects (Rust, Node, Python)
	cargo fmt --all
	cd crates/bodhi && npm run format
	cd crates/lib_bodhiserver_napi && npm run format
	# cd openai-pysdk-compat && poetry run ruff format .
	$(MAKE) -C getbodhi.app format

format.all: format ## Format code in all projects (Rust, Node, Python), and run Clippy
	cargo clippy --fix --allow-dirty --allow-staged

build: ## Build command line app (server variant)
	cargo build -p bodhi

build.native: ## Build native app with system tray
	cd crates/bodhi/src-tauri && cargo tauri build --features native

build.ui: ## Build Next.js frontend and NAPI bindings
	cd crates/bodhi && npm run build
	cd crates/lib_bodhiserver_napi && npm run build

build.ui-clean: ## Clean UI build artifacts
	rm -rf crates/bodhi/out
	cargo clean -p lib_bodhiserver -p bodhi && rm -rf crates/lib_bodhiserver_napi/app-bindings.*.node

build.ui-rebuild: build.ui-clean build.ui ## Clean and rebuild UI

test.coverage: ## Generate code coverage report
	cargo build -p llama_server_proc; \
	PACKAGES=$$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[].name' | sed 's/^/-p /'); \
	cargo llvm-cov test --no-fail-fast --all-features $$PACKAGES --lcov --output-path lcov.info

build.ts-client: ## Build the TypeScript client package
	@echo "==> Building ts-client package"
	@cd ts-client && npm install && npm run build && npm run test && npm run bundle
	@echo "✓ ts-client package built successfully"

run: ## Run command line app
	cargo run --bin bodhi -- serve --port 1135

run.native: ## Run native app in development mode
	cd crates/bodhi/src-tauri && cargo tauri dev

app.clear: ## Clear the app data folders
	rm -rf ~/.cache/bodhi-dev-makefile

app.clean-run:
	cd crates/bodhi && npm run build && cd ../../ && $(MAKE) app.run

app.run: ## Run the BodhiApp
	BODHI_ENCRYPTION_KEY=dummy-key \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.cache/bodhi-dev-makefile \
		cargo run --bin bodhi -- serve --port 1135

test.extension-download: ## Download Bodhi browser extension for testing (use FORCE=1 to check for updates)
	@$(MAKE) -C crates/lib_bodhiserver_napi download-extension FORCE=$(FORCE)

test.model-download: ## Download test model for integration tests (Qwen3-1.7B Q8_0 GGUF)
	@echo "==> Downloading test model for integration tests"
	@command -v hf >/dev/null 2>&1 || { echo "Error: 'hf' command not found. Install with: pip install -U huggingface_hub[cli]"; exit 1; }
	@hf download --revision daeb8e2d528a760970442092f6bf1e55c3b659eb ggml-org/Qwen3-1.7B-GGUF Qwen3-1.7B-Q8_0.gguf
	@hf download --revision 4bcbc666d2f0d2b04d06f046d6baccdab79eac61 afrideva/Llama-68M-Chat-v1-GGUF llama-68m-chat-v1.q8_0.gguf
	@echo "✓ Test model downloaded successfully"
