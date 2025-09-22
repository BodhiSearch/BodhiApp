# Windows-specific builds for pwsh
.PHONY: all clean clean_build build-x86_64-pc-windows-msvc-cpu build-x86_64-pc-windows-msvc-cuda-11.7 build-x86_64-pc-windows-msvc-cuda-12.4 ci.clean ci.coverage

# Common variables
BUILD_DIR := llama.cpp/build

build-x86_64-pc-windows-msvc-cpu:
	pwsh scripts/build_target.win.ps1 -CmakeFlags "-DGGML_NATIVE=OFF" -TargetArch "x86_64-pc-windows-msvc" -BuildType "cpu" || exit 1

build-x86_64-pc-windows-msvc-cuda-11.7:
	pwsh scripts/build_target_cuda.win.ps1 -CudaVersion "11.7" -TargetArch "x86_64-pc-windows-msvc" -BuildType "cuda" || exit 1

build-x86_64-pc-windows-msvc-cuda-12.4:
	pwsh scripts/build_target_cuda.win.ps1 -CudaVersion "12.4" -TargetArch "x86_64-pc-windows-msvc" -BuildType "cuda" || exit 1

clean:
	@pwsh -NoProfile -Command "& scripts/clean.win.ps1" || exit 1
