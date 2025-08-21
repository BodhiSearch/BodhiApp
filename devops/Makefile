# BodhiApp Docker Build Targets
# This file contains all Docker build logic for local development
# Called from root Makefile for delegation

.PHONY: help build.cpu.amd64 build.cpu.arm64 build.cuda build.rocm build.vulkan run clean list-images
.DEFAULT_GOAL := help

# Default build variant (can be overridden)
BUILD_VARIANT ?= development
PROJECT_ROOT := $(shell dirname $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST)))))

help: ## Show Docker build help
	@echo 'Docker Build Targets:'
	@echo ''
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z0-9._-]+:.*?## / {printf "  \033[36m%-25s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ''
	@echo 'Environment Variables:'
	@echo '  BUILD_VARIANT    Build variant: development (default) or production'
	@echo '  GH_PAT          GitHub Personal Access Token (required for ARM64 builds)'
	@echo '  VARIANT         Image variant to run: cpu (default), cuda, rocm, vulkan'
	@echo '  ARCH            Architecture to run: amd64 (default), arm64'
	@echo ''
	@echo 'Examples:'
	@echo '  make build.cpu.amd64                    # Build AMD64 CPU development image'
	@echo '  make build.cpu.arm64 GH_PAT=token      # Build ARM64 CPU image'
	@echo '  make build.cuda BUILD_VARIANT=production  # Build production CUDA image'
	@echo '  make run VARIANT=cuda                  # Run CUDA image'

build.cpu.amd64: ## Build AMD64 CPU image for local testing
	@echo "Building AMD64 CPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/cpu.Dockerfile \
		-t bodhiapp:local-cpu-amd64-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-cpu-amd64-$(BUILD_VARIANT)"

build.cpu.arm64: ## Build ARM64 CPU image (requires GH_PAT)
	@if [ -z "$(GH_PAT)" ]; then \
		echo "Error: GH_PAT must be set for ARM64 builds"; \
		echo "Usage: make build.cpu.arm64 GH_PAT=your_token"; \
		exit 1; \
	fi
	@echo "Building ARM64 CPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--build-arg GH_PAT=$(GH_PAT) \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/cpu-arm64.Dockerfile \
		-t bodhiapp:local-cpu-arm64-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-cpu-arm64-$(BUILD_VARIANT)"

build.cuda: ## Build NVIDIA CUDA GPU image (AMD64 only)
	@echo "Building CUDA GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/cuda.Dockerfile \
		-t bodhiapp:local-cuda-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-cuda-$(BUILD_VARIANT)"

build.rocm: ## Build AMD ROCm GPU image (AMD64 only)
	@echo "Building ROCm GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/rocm.Dockerfile \
		-t bodhiapp:local-rocm-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-rocm-$(BUILD_VARIANT)"

build.vulkan: ## Build Vulkan GPU image (AMD64 only)
	@echo "Building Vulkan GPU image ($(BUILD_VARIANT) variant)..."
	@cd $(PROJECT_ROOT) && docker buildx build --load \
		--build-arg BUILD_VARIANT=$(BUILD_VARIANT) \
		-f devops/vulkan.Dockerfile \
		-t bodhiapp:local-vulkan-$(BUILD_VARIANT) .
	@echo "✓ Built: bodhiapp:local-vulkan-$(BUILD_VARIANT)"

run: ## Run locally built image (specify VARIANT and ARCH)
	@VARIANT=$${VARIANT:-cpu} && \
	ARCH=$${ARCH:-amd64} && \
	BUILD_VAR=$${BUILD_VARIANT:-$(BUILD_VARIANT)} && \
	IMAGE_NAME="bodhiapp:local-$$VARIANT"; \
	if [ "$$VARIANT" = "cpu" ]; then \
		IMAGE_NAME="$$IMAGE_NAME-$$ARCH"; \
	fi; \
	IMAGE_NAME="$$IMAGE_NAME-$$BUILD_VAR"; \
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

list-images: ## List all locally built BodhiApp images
	@echo "Local BodhiApp images:"
	@docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}" --filter=reference="bodhiapp:local-*"

clean: ## Remove all locally built BodhiApp images
	@echo "Removing local BodhiApp images..."
	@docker images --format "{{.Repository}}:{{.Tag}}" --filter=reference="bodhiapp:local-*" | xargs -r docker rmi
	@echo "✓ Cleaned up local images"

# Development convenience targets
build.all.cpu: build.cpu.amd64 build.cpu.arm64 ## Build both CPU architectures (requires GH_PAT for ARM64)

build.all.gpu: build.cuda build.rocm build.vulkan ## Build all GPU variants

# Quick development targets (development variant)
dev.cpu.amd64: ## Quick development build for AMD64 CPU
	@$(MAKE) build.cpu.amd64 BUILD_VARIANT=development

dev.cpu.arm64: ## Quick development build for ARM64 CPU (requires GH_PAT)
	@$(MAKE) build.cpu.arm64 BUILD_VARIANT=development

dev.cuda: ## Quick development build for CUDA
	@$(MAKE) build.cuda BUILD_VARIANT=development

# Production targets
prod.cpu.amd64: ## Production build for AMD64 CPU
	@$(MAKE) build.cpu.amd64 BUILD_VARIANT=production

prod.cpu.arm64: ## Production build for ARM64 CPU (requires GH_PAT)
	@$(MAKE) build.cpu.arm64 BUILD_VARIANT=production

prod.cuda: ## Production build for CUDA
	@$(MAKE) build.cuda BUILD_VARIANT=production