.PHONY: build_frontend ci.check-macos-vars ci.sign-binaries

build_frontend:
	cd .. && pnpm install && pnpm run build

# Required env vars for macOS signing
REQUIRED_MACOS_VARS := APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY
# Generate a random keychain password
KEYCHAIN_PASSWORD := $(shell openssl rand -base64 24)

ci.check-macos-vars:
	@for var in $(REQUIRED_MACOS_VARS); do \
		if [ -z "$${!var}" ]; then \
			echo "Error: Required environment variable $$var is not set"; \
			exit 1; \
		fi \
	done

ci.sign-binaries: ci.check-macos-vars
	@echo "Signing embedded binaries for macOS..."
	@echo "$$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
	@security create-keychain -p "$(KEYCHAIN_PASSWORD)" build.keychain || (echo "Failed to create keychain" && exit 1)
	@security default-keychain -s build.keychain || (echo "Failed to set default keychain" && exit 1)
	@security unlock-keychain -p "$(KEYCHAIN_PASSWORD)" build.keychain || (echo "Failed to unlock keychain" && exit 1)
	@security import certificate.p12 -k build.keychain -P "$$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign || (echo "Failed to import certificate" && exit 1)
	@security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$(KEYCHAIN_PASSWORD)" build.keychain || (echo "Failed to set partition list" && exit 1)

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
