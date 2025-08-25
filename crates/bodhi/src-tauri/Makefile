.PHONY: help build_frontend ci.check-macos-vars ci.sign-binaries
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo 'Tauri Desktop Application Build System'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build_frontend: ## Build Next.js frontend for Tauri
	cd .. && npm install && npm run build

# Required env vars for macOS signing
REQUIRED_MACOS_VARS := APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY KEYCHAIN_PASSWORD

ci.check-macos-vars: ## Verify macOS signing environment variables are set
	@for var in $(REQUIRED_MACOS_VARS); do \
		if [ -z "$${!var}" ]; then \
			echo "Error: Required environment variable $$var is not set"; \
			exit 1; \
		fi \
	done

ci.sign-binaries: ci.check-macos-vars ## Sign embedded binaries for macOS distribution
	@echo "Signing embedded binaries for macOS..."
	@echo "$$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
	@security create-keychain -p "$$KEYCHAIN_PASSWORD" build.keychain || (echo "Failed to create keychain" && exit 1)
	@security default-keychain -s build.keychain || (echo "Failed to set default keychain" && exit 1)
	@security unlock-keychain -p "$$KEYCHAIN_PASSWORD" build.keychain || (echo "Failed to unlock keychain" && exit 1)
	@security import certificate.p12 -k build.keychain -P "$$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign || (echo "Failed to import certificate" && exit 1)
	@security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$$KEYCHAIN_PASSWORD" build.keychain || (echo "Failed to set partition list" && exit 1)

	@echo "Signing metal binary..."
	@codesign --force --options runtime --sign "$$APPLE_SIGNING_IDENTITY" --timestamp bin/aarch64-apple-darwin/metal/llama-server || (echo "Failed to sign metal binary" && exit 1)

	@echo "Signing CPU binary..."
	@codesign --force --options runtime --sign "$$APPLE_SIGNING_IDENTITY" --timestamp bin/aarch64-apple-darwin/cpu/llama-server || (echo "Failed to sign CPU binary" && exit 1)

	@echo "Verifying signatures..."
	@codesign -vv --deep --strict bin/aarch64-apple-darwin/metal/llama-server || (echo "Failed to verify metal binary signature" && exit 1)
	@codesign -vv --deep --strict bin/aarch64-apple-darwin/cpu/llama-server || (echo "Failed to verify CPU binary signature" && exit 1)

	@echo "Cleaning up..."
	@rm certificate.p12
	@security delete-keychain build.keychain
	@echo "Binary signing completed successfully"
