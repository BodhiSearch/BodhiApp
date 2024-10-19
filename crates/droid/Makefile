# Makefile for Android build process

# Variables
ANDROID_HOME ?= $(shell echo $$ANDROID_HOME)
NDK_VERSION := 27.1.12297006
BUILD_TOOLS_VERSION := 34.0.0

# Ensure ANDROID_HOME is set
ifeq ($(ANDROID_HOME),)
$(error ANDROID_HOME is not set. Please set it to your Android SDK path)
endif

# Build Android app
.PHONY: build-android
build-android:
	@echo "Building Android app..."
	NDK_HOME=$(ANDROID_HOME)/ndk/$(NDK_VERSION) cargo tauri android build -v

# Sign APK
.PHONY: sign-apk
sign-apk:
	@echo "Signing APK..."
	@if [ -z "$(ANDROID_KEYSTORE_PATH)" ]; then \
		echo "ANDROID_KEYSTORE_PATH is not set. Please set it to your keystore path"; \
		exit 1; \
	fi
	@if [ -z "$(ANDROID_KEY_ALIAS)" ]; then \
		echo "ANDROID_KEY_ALIAS is not set. Please set it"; \
		exit 1; \
	fi
	@if [ -z "$(ANDROID_KEYSTORE_PASSWORD)" ]; then \
		echo "ANDROID_KEYSTORE_PASSWORD is not set. Please set it"; \
		exit 1; \
	fi
	$(ANDROID_HOME)/build-tools/$(BUILD_TOOLS_VERSION)/apksigner sign --ks $(ANDROID_KEYSTORE_PATH) \
		--ks-key-alias $(ANDROID_KEY_ALIAS) \
		--ks-pass pass:$(ANDROID_KEYSTORE_PASSWORD) \
		--key-pass pass:$(ANDROID_KEYSTORE_PASSWORD) \
		--out src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-signed.apk \
		src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

# Rename APK
.PHONY: rename-apk
rename-apk:
	@echo "Renaming APK..."
	$(eval VERSION := $(shell node -p "require('./package.json').version"))
	mv ./src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-signed.apk \
	   ./src-tauri/gen/android/app/build/outputs/apk/universal/release/bodhidroid-$(VERSION).apk
