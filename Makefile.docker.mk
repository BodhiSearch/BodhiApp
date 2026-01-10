# Docker Build and Runtime Targets
# This file contains Docker image building and runtime management targets

.PHONY: docker.dev.cpu docker.dev.cpu.amd64 docker.dev.cpu.arm64 docker.dev.cuda \
	docker.run.amd64 docker.run.arm64 docker.list docker.clean

# Docker build targets - delegated to devops/Makefile
docker.dev.cpu: ## Build CPU image for current platform (use PLATFORM to override)
	@$(MAKE) -C devops dev.cpu BUILD_VARIANT=$${BUILD_VARIANT:-development} PLATFORM=$${PLATFORM}

docker.dev.cpu.amd64: ## Build AMD64 CPU image for local testing
	@$(MAKE) -C devops dev.cpu.amd64 BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.dev.cpu.arm64: ## Build ARM64 CPU image for local testing
	@$(MAKE) -C devops dev.cpu.arm64 BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.dev.cuda: ## Build NVIDIA CUDA GPU image
	@$(MAKE) -C devops dev.cuda BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.amd64: ## Run locally built linux/amd64 Docker image
	@$(MAKE) -C devops run VARIANT=$${VARIANT:-cpu} ARCH=$${ARCH:-amd64} BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.run.arm64: ## Run locally built linux/arm64 Docker image
	@$(MAKE) -C devops run VARIANT=$${VARIANT:-cpu} ARCH=$${ARCH:-arm64} BUILD_VARIANT=$${BUILD_VARIANT:-development}

docker.list: ## List all locally built BodhiApp images
	@$(MAKE) -C devops list-images

docker.clean: ## Remove all locally built BodhiApp images
	@$(MAKE) -C devops clean
