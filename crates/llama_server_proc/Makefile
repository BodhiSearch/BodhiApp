# Makefile
.PHONY: all clean clean_build clean_target configure build copy_lib

BUILD_DIR := llama.cpp/build
BIN_DIR := bin

# Common CMake flags
CMAKE_COMMON_FLAGS := -DLLAMA_FATAL_WARNINGS=OFF -DLLAMA_CURL=OFF -DBUILD_SHARED_LIBS=OFF -DCMAKE_BUILD_TYPE=Release

# Platform-specific variables
ifeq ($(shell uname -s),Darwin)
  NCPU := $(shell sysctl -n hw.logicalcpu)
else
  NCPU := $(shell nproc)
endif
BIN_EXT :=

# Define the variable for the build arguments
CUDA_BUILD_FLAGS := -DCMAKE_CUDA_ARCHITECTURES=89-real -DCMAKE_EXE_LINKER_FLAGS=-Wl,--allow-shlib-undefined -DGGML_NATIVE=OFF -DGGML_CUDA=ON

all: configure build copy_exec

clean:
	rm -rf $(BUILD_DIR)
	mkdir -p $(BUILD_DIR)

# Generic build function
define build_target
	$(eval PROJECT_ROOT := $(shell pwd))
	cd llama.cpp && \
		rm -rf build && \
		$(2) cmake -S . -B build $(1) $(CMAKE_COMMON_FLAGS) && \
		cmake --build build --config Release -j $(NCPU) --target llama-server && \
		rm -rf $(PROJECT_ROOT)/$(BIN_DIR)/$(3)/$(4) && \
		mkdir -p $(PROJECT_ROOT)/$(BIN_DIR)/$(3)/$(4) && \
		find $(PROJECT_ROOT)/llama.cpp/build -type f -name "llama-server$(BIN_EXT)" -exec sh -c '\
			filename=$$(basename "{}"); \
			cp "{}" "$(PROJECT_ROOT)/$(BIN_DIR)/$(3)/$(4)/$$filename"; \
		' \;
endef

# Target-specific builds
build-aarch64-apple-darwin-metal:
	$(call build_target,-DGGML_METAL_EMBED_LIBRARY=ON,UNAME_S=Darwin UNAME_P=arm64 UNAME_M=arm64,aarch64-apple-darwin,metal)

build-aarch64-apple-darwin-cpu:
	$(call build_target,-DGGML_METAL_EMBED_LIBRARY=OFF -DGGML_METAL=OFF,UNAME_S=Darwin UNAME_P=arm64 UNAME_M=arm64,aarch64-apple-darwin,cpu)

build-aarch64-unknown-linux-gnu-cpu:
	$(call build_target,,UNAME_S=Linux UNAME_P=unknown UNAME_M=arm64,aarch64-unknown-linux-gnu,cpu)

build-x86_64-unknown-linux-gnu-cpu:
	$(call build_target,,UNAME_S=Linux UNAME_P=unknown UNAME_M=x86_64,x86_64-unknown-linux-gnu,cpu)

build-x86_64-unknown-linux-gnu-cuda-12_6:
	$(call build_target,"$(CUDA_BUILD_FLAGS)",UNAME_S=Linux UNAME_P=unknown UNAME_M=x86_64,x86_64-unknown-linux-gnu,cuda-12_6)
