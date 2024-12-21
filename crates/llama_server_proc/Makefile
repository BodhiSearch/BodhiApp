# Makefile
.PHONY: all clean clean_build clean_target configure build copy_lib

BUILD_DIR := llama.cpp/build
BIN_DIR := bin

# Common CMake flags
CMAKE_COMMON_FLAGS := -DLLAMA_FATAL_WARNINGS=OFF -DLLAMA_CURL=OFF -DBUILD_LLAMA_SERVER=ON

# Platform-specific variables
ifeq ($(shell uname -s),Darwin)
  NCPU := $(shell sysctl -n hw.logicalcpu)
	BIN_EXT := ""
else
  NCPU := $(shell nproc)
	BIN_EXT := .exe
endif

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
	$(call build_target,-DGGML_METAL_EMBED_LIBRARY=ON,,aarch64-apple-darwin,metal)

build-aarch64-apple-darwin-cpu:
	$(call build_target,-DGGML_METAL_EMBED_LIBRARY=OFF -DGGML_METAL=OFF,,aarch64-apple-darwin,cpu)

build-aarch64-unknown-linux-gnu-cpu:
	$(call build_target,-DGGML_NATIVE=OFF,UNAME_S=Linux UNAME_P=unknown UNAME_M=arm64,aarch64-unknown-linux-gnu,cpu)

build-x86_64-unknown-linux-gnu-cpu:
	$(call build_target,-DGGML_NATIVE=OFF,UNAME_S=Linux UNAME_P=unknown UNAME_M=x86_64,x86_64-unknown-linux-gnu,cpu)
