# BodhiApp Docker Build Targets
# This file contains all Docker build logic for local development
# Called from root Makefile for delegation

.PHONY: help build.app-binary build.cpu build.cpu.amd64 build.cpu.arm64 build.cpu.multi build.cuda build.rocm build.vulkan build.multi-tenant build.multi-tenant-arm64 run run.multi-tenant clean list-images dev.multi-tenant dev.multi-tenant.amd64 prod.multi-tenant prod.multi-tenant.amd64
.DEFAULT_GOAL := help

# Default build variant (can be overridden)
BUILD_VARIANT ?= development
PROJECT_ROOT := $(shell dirname $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))

# App binary image tag (for local builds, we use a local tag)
APP_BINARY_TAG ?= bodhiapp:local-app-binary-$(BUILD_VARIANT)

help: ## Show this help message
	@echo 'BodhiApp Docker Build System'
	@echo ''
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@echo ''
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ''
	@echo 'Environment Variables:'
	@echo '  BUILD_VARIANT    Build variant: development (default) or production'
	@echo '  PLATFORM        Docker platform: linux/amd64 (default), linux/arm64'
	@echo '  VARIANT         Image variant to run: cpu (default), cuda, rocm, vulkan, multi-tenant, multi-tenant-arm64'
	@echo ''
	@echo 'Examples:'
	@echo '  make build.cpu                          # Build CPU image for current platform (AMD64)'
	@echo '  make build.cpu.arm64                    # Build ARM64 CPU image explicitly'
	@echo '  make build.cpu PLATFORM=linux/arm64    # Build CPU image for specific platform'
	@echo '  make build.cuda BUILD_VARIANT=production  # Build production CUDA image'
	@echo '  make run VARIANT=cpu                   # Run CPU image'

build.app-binary: ## Build app-binary base image (required for all variant builds)
	@PLATFORM=$${PLATFORM:-linux/amd64} && \
	echo "Building app-binary image for platform $$PLATFORM ($(BUILD_VARIANT) variant)..." && \
	cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform $$PLATFORM \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/app-binary.Dockerfile \
		-t $(APP_BINARY_TAG) .
	@echo "✓ Built: $(APP_BINARY_TAG)"

build.app-binary.multi: ## Build multi-platform app-binary image (for local registry testing)
	@echo "Building multi-platform app-binary image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/app-binary.Dockerfile \
		-t $(APP_BINARY_TAG) .
	@echo "✓ Built: $(APP_BINARY_TAG) (multi-platform: AMD64 + ARM64)"

build.cpu: build.app-binary ## Build CPU image for current platform (use PLATFORM to override)
	@PLATFORM=$${PLATFORM:-linux/amd64} && \
	echo "Building CPU image for platform $$PLATFORM ($(BUILD_VARIANT) variant)..." && \
	cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform $$PLATFORM \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/cpu.Dockerfile \
		-t bodhiapp:local-cpu-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-cpu-$(BUILD_VARIANT)"

build.cpu.amd64: ## Build AMD64 CPU image explicitly
	@$(MAKE) build.cpu PLATFORM=linux/amd64

build.cpu.arm64: ## Build ARM64 CPU image explicitly  
	@$(MAKE) build.cpu PLATFORM=linux/arm64

build.cpu.multi: ## Build multi-platform CPU image (for CI/push to registry)
	@echo "Building multi-platform CPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build \
		--platform linux/amd64,linux/arm64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/cpu.Dockerfile \
		-t bodhiapp:cpu-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:cpu-$(BUILD_VARIANT) (multi-platform: AMD64 + ARM64)"


build.cuda: build.app-binary ## Build NVIDIA CUDA GPU image (AMD64 only)
	@echo "Building CUDA GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/amd64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/cuda.Dockerfile \
		-t bodhiapp:local-cuda-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-cuda-$(BUILD_VARIANT)"

build.rocm: build.app-binary ## Build AMD ROCm GPU image (AMD64 only)
	@echo "Building ROCm GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/amd64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/rocm.Dockerfile \
		-t bodhiapp:local-rocm-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-rocm-$(BUILD_VARIANT)"

build.vulkan: build.app-binary ## Build Vulkan GPU image (AMD64 only)
	@echo "Building Vulkan GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/amd64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/vulkan.Dockerfile \
		-t bodhiapp:local-vulkan-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-vulkan-$(BUILD_VARIANT)"

build.multi-tenant: build.app-binary ## Build multi-tenant image (AMD64, for cloud)
	@echo "Building multi-tenant AMD64 image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/amd64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/multi-tenant.Dockerfile \
		-t bodhiapp:local-multi-tenant-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-multi-tenant-$(BUILD_VARIANT)"

build.multi-tenant-arm64: ## Build multi-tenant image (ARM64, for macOS dev)
	@$(MAKE) build.app-binary PLATFORM=linux/arm64
	@echo "Building multi-tenant ARM64 image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--platform linux/arm64 \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		--build-arg APP_BINARY_IMAGE=$(APP_BINARY_TAG) \
		-f devops/multi-tenant-arm64.Dockerfile \
		-t bodhiapp:local-multi-tenant-arm64-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-multi-tenant-arm64-$(BUILD_VARIANT)"

run: ## Run locally built image (specify VARIANT)
	@VARIANT=$${VARIANT:-cpu} && \
	BUILD_VAR=$${BUILD_VARIANT:-$(BUILD_VARIANT)} && \
	IMAGE_NAME="bodhiapp:local-$$VARIANT-$$BUILD_VAR"; \
	echo "Running BodhiApp container: $$IMAGE_NAME"; \
	mkdir -p $(PROJECT_ROOT)/docker-data/bodhi_home $(PROJECT_ROOT)/docker-data/hf_home && \
	docker run --rm -it \
		-e BODHI_LOG_STDOUT=true \
		-e BODHI_LOG_LEVEL=debug \
		-e BODHI_ENCRYPTION_KEY=local-dev-key \
		-e BODHI_PUBLIC_HOST=localhost \
		-e BODHI_PUBLIC_PORT=8080 \
		-p 8080:8080 \
		-v $(PROJECT_ROOT)/docker-data/bodhi_home:/data/bodhi_home \
		-v $(PROJECT_ROOT)/docker-data/hf_home:/data/hf_home \
		$$IMAGE_NAME

run.cuda: ## Run CUDA image with GPU support
	@$(MAKE) run VARIANT=cuda EXTRA_ARGS="--gpus all"

run.rocm: ## Run ROCm image with AMD GPU support
	@$(MAKE) run VARIANT=rocm EXTRA_ARGS="--device=/dev/kfd --device=/dev/dri --group-add video"

run.vulkan: ## Run Vulkan image with GPU support
	@$(MAKE) run VARIANT=vulkan EXTRA_ARGS="--device=/dev/dri"

run.multi-tenant: ## Run multi-tenant image (defaults to ARM64 for macOS dev)
	@ENV_FILE="$(CURDIR)/.env.local"; \
	if [ ! -f "$$ENV_FILE" ]; then \
		echo "ERROR: $$ENV_FILE not found. Copy .env.example to .env.local and fill in values:"; \
		echo "  cp devops/.env.example devops/.env.local"; \
		exit 1; \
	fi; \
	VARIANT=$${VARIANT:-multi-tenant-arm64} && \
	BUILD_VAR=$${BUILD_VARIANT:-$(BUILD_VARIANT)} && \
	IMAGE_NAME="bodhiapp:local-$$VARIANT-$$BUILD_VAR"; \
	echo "Running BodhiApp multi-tenant container: $$IMAGE_NAME"; \
	mkdir -p $(PROJECT_ROOT)/docker-data/mt_bodhi_home $(PROJECT_ROOT)/docker-data/mt_hf_home && \
	docker run --rm -it \
		--env-file "$$ENV_FILE" \
		-p 1135:1135 \
		-v $(PROJECT_ROOT)/docker-data/mt_bodhi_home:/data/bodhi_home \
		-v $(PROJECT_ROOT)/docker-data/mt_hf_home:/data/hf_home \
		$$IMAGE_NAME

list-images: ## List all locally built BodhiApp images
	@echo "Local BodhiApp images:"
	@docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" --filter=reference="bodhiapp:local-*"

clean: ## Remove all locally built BodhiApp images
	@echo "Removing local BodhiApp images..."
	@docker images --format "{{.Repository}}:{{.Tag}}" --filter=reference="bodhiapp:local-*" | xargs -r docker rmi
	@echo "✓ Cleaned up local images"

# Development convenience targets
build.all.cpu: build.cpu.amd64 build.cpu.arm64 ## Build CPU images for both platforms

build.all.gpu: build.cuda build.rocm build.vulkan ## Build all GPU variants

# Quick development targets (development variant)
dev.cpu: ## Quick development build for CPU (current platform)
	@$(MAKE) build.cpu BUILD_VARIANT=development

dev.cpu.amd64: ## Quick development build for AMD64 CPU
	@$(MAKE) build.cpu.amd64 BUILD_VARIANT=development

dev.cpu.arm64: ## Quick development build for ARM64 CPU
	@$(MAKE) build.cpu.arm64 BUILD_VARIANT=development

dev.cuda: ## Quick development build for CUDA
	@$(MAKE) build.cuda BUILD_VARIANT=development

# Production targets  
prod.cpu: ## Production build for CPU (current platform)
	@$(MAKE) build.cpu BUILD_VARIANT=production

prod.cpu.amd64: ## Production build for AMD64 CPU
	@$(MAKE) build.cpu.amd64 BUILD_VARIANT=production

prod.cpu.arm64: ## Production build for ARM64 CPU
	@$(MAKE) build.cpu.arm64 BUILD_VARIANT=production

prod.cuda: ## Production build for CUDA
	@$(MAKE) build.cuda BUILD_VARIANT=production

dev.multi-tenant: ## Quick development build for multi-tenant (ARM64, macOS dev)
	@$(MAKE) build.multi-tenant-arm64 BUILD_VARIANT=development

dev.multi-tenant.amd64: ## Quick development build for multi-tenant (AMD64, cloud)
	@$(MAKE) build.multi-tenant BUILD_VARIANT=development

prod.multi-tenant: ## Production build for multi-tenant (ARM64)
	@$(MAKE) build.multi-tenant-arm64 BUILD_VARIANT=production

prod.multi-tenant.amd64: ## Production build for multi-tenant (AMD64, cloud)
	@$(MAKE) build.multi-tenant BUILD_VARIANT=production
