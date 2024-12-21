# Windows-specific builds for pwsh
.PHONY: all clean clean_build build-x86_64-pc-windows-msvc-cpu build-x86_64-pc-windows-msvc-cuda ci.clean ci.coverage

# Common variables
BUILD_DIR := llama.cpp/build

build-x86_64-pc-windows-msvc-cpu:
	pwsh scripts/build_target.win.ps1 -CmakeFlags "-DGGML_NATIVE=OFF" -TargetArch "x86_64-pc-windows-msvc" -BuildType "cpu" || exit 1

clean:
	@pwsh -NoProfile -Command "$$ErrorActionPreference='Stop'; & scripts/clean.win.ps1" || exit 1
