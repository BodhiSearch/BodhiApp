# Include shared utilities and domain-specific targets
include Makefile.release.mk
include Makefile.ci.mk
include Makefile.docker.mk
include Makefile.website.mk

.DEFAULT_GOAL := help

.PHONY: help test test.backend test.ui test.napi test.coverage \
	test.deps.up test.deps.down \
	dev.deps.up dev.deps.down dev.deps.clear \
	build build.native build.ui build.ui-clean build.ui-rebuild build.ts-client \
	openapi.anthropic openapi.gemini \
	format format.all \
	run run.native app.clear app.run app.run.pg app.run.live app.run.live.stop \
	test.extension-download test.model-download \
	setup.worktree

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo '=== Testing ==='
	@awk 'BEGIN {FS = ":.*?## "} /^test[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Building ==='
	@awk 'BEGIN {FS = ":.*?## "} /^(build|openapi)[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== Formatting ==='
	@awk 'BEGIN {FS = ":.*?## "} /^format[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
	@echo ''
	@echo '=== App Runtime ==='
	@awk 'BEGIN {FS = ":.*?## "} /^(run|app|dev)[a-zA-Z0-9._-]*:.*?## / {printf "  \033[36m%-30s\033[0m %s \033[90m(%s)\033[0m\n", $$1, $$2, FILENAME}' $(MAKEFILE_LIST)
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
	docker compose -f docker/docker-compose.test.yml up -d --wait

test.deps.down: ## Stop test dependencies and remove volumes
	docker compose -f docker/docker-compose.test.yml down -v

dev.deps.up: ## Start dev PostgreSQL databases (ports 34320/34321)
	docker compose -f docker/docker-compose.dev.yml up -d --wait

dev.deps.down: ## Stop dev PostgreSQL databases (keep volumes)
	docker compose -f docker/docker-compose.dev.yml down

dev.deps.clear: ## Stop dev PostgreSQL databases and remove volumes
	docker compose -f docker/docker-compose.dev.yml down -v

test.backend: test.deps.up ## Run Rust backend tests (requires Docker for PostgreSQL)
	cargo test --no-fail-fast
	cargo test -p bodhi --features native --no-fail-fast

test.ui.unit: ## Run frontend unit tests
	cd crates/bodhi && npm install && npm test

test.ui: ## Run frontend and UI integration tests
	$(MAKE) test.ui.unit
	$(MAKE) -C crates/lib_bodhiserver_napi test.ui

test.napi.standalone: ## Run frontend and UI integration tests
	$(MAKE) -C crates/lib_bodhiserver_napi test.napi.standalone

test.napi.multi_tenant: ## Run frontend and UI integration tests
	$(MAKE) -C crates/lib_bodhiserver_napi test.napi.multi_tenant

test.napi: ## Run NAPI bindings tests
	cd crates/lib_bodhiserver_napi && npm install && npm run test:playwright

test: test.backend test.ui test.napi ## Run all tests (backend, UI, NAPI)

format: ## Format code in all projects (Rust, Node, Python)
	cargo fmt --all
	cd crates/bodhi && npm run format && npm run test:typecheck
	cd crates/lib_bodhiserver_napi && npm run format
	# cd openai-pysdk-compat && poetry run ruff format .
	$(MAKE) -C getbodhi.app format

format.all: format ## Format code in all projects (Rust, Node, Python), and run Clippy
	cargo clippy --fix --allow-dirty --allow-staged

build: ## Build command line app (server variant)
	cargo build -p bodhi

build.native: ## Build native app with system tray
	cd crates/bodhi/src-tauri && cargo tauri build --features native

build.ui: ## Build Vite frontend and NAPI bindings
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

openapi.anthropic: ## Download and filter Anthropic OpenAPI spec, generate types
	@echo "==> Syncing Anthropic OpenAPI spec"
	@cd ts-client && npm install && node scripts/sync-anthropic-openapi.mjs
	@echo "✓ Anthropic OpenAPI spec synced"

openapi.gemini: ## Download and filter Gemini OpenAPI spec, generate types
	@echo "==> Syncing Gemini OpenAPI spec"
	@cd ts-client && npm install && node scripts/sync-gemini-openapi.mjs
	@echo "✓ Gemini OpenAPI spec synced"

run: ## Run command line app
	cargo run --bin bodhi -- serve --port 1135

run.native: ## Run native app in development mode
	cd crates/bodhi/src-tauri && cargo tauri dev

app.clear: ## Clear the app data folders
	rm -rf ~/.bodhi-dev-makefile

app.clean-run:
	cd crates/bodhi && npm run build && cd ../../ && $(MAKE) app.run

app.run: ## Run the BodhiApp
	BODHI_ENCRYPTION_KEY=dummy-key \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.bodhi-dev-makefile \
		cargo run --bin bodhi -- serve --port 1135

app.run.pg: dev.deps.up ## Run the BodhiApp with PostgreSQL dev databases
	BODHI_ENCRYPTION_KEY=dummy-key \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.bodhi-dev-makefile \
		BODHI_APP_DB_URL=postgres://bodhi_dev:bodhi_dev@localhost:34320/bodhi_app \
		BODHI_SESSION_DB_URL=postgres://bodhi_dev:bodhi_dev@localhost:34321/bodhi_sessions \
		cargo run --bin bodhi -- serve --port 1135

app.run.live: ## Run BodhiApp with live Vite dev server (HMR enabled)
	@echo "==> Starting Vite dev server..."
	@cd crates/bodhi && npm run dev &
	@echo "==> Waiting for Vite dev server at http://localhost:3000/ui/..."
	@until curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/ui/ | grep -q '200'; do \
		sleep 1; \
	done
	@echo "Vite dev server is ready"
	@echo "==> Starting Rust server with BODHI_DEV_PROXY_UI=true..."
	BODHI_ENCRYPTION_KEY=dummy-key \
		BODHI_LOG_LEVEL=info \
		BODHI_LOG_STDOUT=true \
		BODHI_HOME=~/.bodhi-dev-makefile \
		BODHI_DEV_PROXY_UI=true \
		cargo run --bin bodhi -- serve --port 1135

app.run.live.stop: ## Stop live dev servers (Vite + Rust)
	@pkill -f "vite" 2>/dev/null || true
	@echo "Dev servers stopped"

test.extension-download: ## Download Bodhi browser extension for testing (use FORCE=1 to check for updates)
	@$(MAKE) -C crates/lib_bodhiserver_napi download-extension FORCE=$(FORCE)

test.model-download: ## Download test model for integration tests (Qwen3-1.7B Q8_0 GGUF)
	@echo "==> Downloading test model for integration tests"
	@command -v hf >/dev/null 2>&1 || { echo "Error: 'hf' command not found. Install with: pip install -U huggingface_hub[cli]"; exit 1; }
	@hf download --revision daeb8e2d528a760970442092f6bf1e55c3b659eb ggml-org/Qwen3-1.7B-GGUF Qwen3-1.7B-Q8_0.gguf
	@hf download --revision 4bcbc666d2f0d2b04d06f046d6baccdab79eac61 afrideva/Llama-68M-Chat-v1-GGUF llama-68m-chat-v1.q8_0.gguf
	@echo "✓ Test model downloaded successfully"

setup.worktree: ## Setup git worktree in working state
	@echo "==> Initializing git submodules"
	git submodule update --init --recursive
	@echo "✓ Submodules initialized"
	@echo "==> Copying .env.test files from main project"
	cp ../../BodhiApp/crates/auth_middleware/tests/.env.test crates/auth_middleware/tests/.env.test
	cp ../../BodhiApp/crates/server_app/tests/resources/.env.test crates/server_app/tests/resources/.env.test
	cp ../../BodhiApp/crates/lib_bodhiserver_napi/tests-js/.env.test crates/lib_bodhiserver_napi/tests-js/.env.test
	cp ../../BodhiApp/crates/services/.env.test crates/services/.env.test
	@echo "✓ .env.test files copied"
	@echo "==> Installing Python dependencies for objs tests"
	pip install -r crates/objs/tests/scripts/requirements.txt
	@echo "✓ Python dependencies installed"
	@echo "==> Installing npm dependencies for test-oauth-app"
	cd crates/lib_bodhiserver_napi/test-oauth-app && npm install
	@echo "✓ test-oauth-app dependencies installed"
	@echo "==> Downloading test models for integration tests"
	@command -v hf >/dev/null 2>&1 || { echo "Error: 'hf' command not found. Install with: pip install -U huggingface_hub[cli]"; exit 1; }
	@hf download --revision daeb8e2d528a760970442092f6bf1e55c3b659eb ggml-org/Qwen3-1.7B-GGUF Qwen3-1.7B-Q8_0.gguf
	@hf download --revision 4bcbc666d2f0d2b04d06f046d6baccdab79eac61 afrideva/Llama-68M-Chat-v1-GGUF llama-68m-chat-v1.q8_0.gguf
	@echo "✓ Test models downloaded"
	@echo "==> Rebuilding NAPI bindings with latest code"
	cd crates/lib_bodhiserver_napi && npm run build
	@echo "✓ NAPI bindings rebuilt"
