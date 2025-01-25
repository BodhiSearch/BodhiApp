param(
    [Parameter(Mandatory=$true)]
    [string]$CudaVersion,

    [Parameter(Mandatory=$true)]
    [string]$TargetArch,

    [Parameter(Mandatory=$true)]
    [string]$BuildType
)

$ErrorActionPreference = "Stop"  # Stop on any error

# Constants
$BUILD_DIR = "llama.cpp/build"
$VS_PATH = "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvars64.bat"

# Change to llama.cpp directory from the script's location
Push-Location (Join-Path $PSScriptRoot "../llama.cpp")

try {
    # Initialize Visual Studio environment
    Write-Host "Initializing Visual Studio environment..."
    $vsInit = "`"$VS_PATH`" && set"
    $vsVars = cmd /c "$vsInit" | Where-Object { $_ -match '=' } | ForEach-Object {
        $parts = $_ -split '=', 2
        @{$parts[0] = $parts[1]}
    }
    foreach ($var in $vsVars) {
        $var.GetEnumerator() | ForEach-Object {
            Set-Item "env:$($_.Key)" $_.Value
        }
    }

    # Clean any existing CMake cache
    if (Test-Path "build") {
        Write-Host "Cleaning existing build directory..."
        Remove-Item -Path "build" -Recurse -Force
    }

    # Debug output
    Write-Host "=== Build Configuration ==="
    Write-Host "CUDA Version: $CudaVersion"
    Write-Host "Target Architecture: $TargetArch"
    Write-Host "Build Type: $BuildType"

    # Ensure Ninja is available
    if (-not (Get-Command "ninja.exe" -ErrorAction SilentlyContinue)) {
        Write-Host "Installing Ninja..."
        choco install ninja -y
        $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    }

    # CMake configuration
    Write-Host '=== Running CMake Configure ==='
    $cmakeFlags = @(
        "-G `"Ninja`"",
        "-DLLAMA_BUILD_SERVER=ON",
        "-DGGML_NATIVE=OFF",
        "-DGGML_CUDA=ON",
        "-DGGML_RPC=ON"
    )
    $cmakeCommand = "cmake -S . -B build $($cmakeFlags -join ' ')"
    Write-Host "Executing command: $cmakeCommand"
    $cmakeResult = Invoke-Expression $cmakeCommand
    if ($LASTEXITCODE -ne 0) {
        Write-Host "CMake output:"
        Write-Host $cmakeResult
        throw "CMake configure failed with exit code $LASTEXITCODE"
    }

    # Calculate number of jobs for ninja
    $numCpu = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
    $ninjaJobs = [Math]::Max(1, $numCpu - 1)
    Write-Host "Using $ninjaJobs ninja jobs"

    # Build ggml target first
    Write-Host '=== Building ggml target ==='
    $buildGgmlCommand = "cmake --build build --config Release -j $ninjaJobs -t ggml"
    Write-Host "Executing command: $buildGgmlCommand"
    $ggmlResult = Invoke-Expression $buildGgmlCommand
    if ($LASTEXITCODE -ne 0) {
        Write-Host "GGML build output:"
        Write-Host $ggmlResult
        throw "GGML build failed with exit code $LASTEXITCODE"
    }

    # Build main release
    Write-Host '=== Building main release ==='
    $buildCommand = "cmake --build build --config Release -j $ninjaJobs"
    Write-Host "Executing command: $buildCommand"
    $buildResult = Invoke-Expression $buildCommand
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build output:"
        Write-Host $buildResult
        throw "Main build failed with exit code $LASTEXITCODE"
    }

    # Copy artifacts
    Write-Host '=== Copying Build Artifacts ==='
    $destPath = Join-Path $PSScriptRoot '..' 'bin'
    $destPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destPath)
    $sourceDir = Join-Path (Get-Location) 'build'
    $sourceDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($sourceDir)
    $targetDir = Join-Path $destPath $TargetArch "cuda-$CudaVersion"

    # Create target directory
    New-Item -ItemType Directory -Force $targetDir | Out-Null

    # Copy server executable and required DLLs
    Get-ChildItem -Path $sourceDir -Filter "llama-server.exe" -Recurse | ForEach-Object {
        $destination = Join-Path $targetDir $_.Name
        Write-Host "Copying $($_.Name) to $destination"
        Copy-Item -Path $_.FullName -Destination $destination -Force
    }

    # Copy CUDA runtime DLLs if needed
    $cudaPath = $env:CUDA_PATH
    if ($cudaPath) {
        $cudaDlls = @(
            "cudart64_*.dll",
            "cublas64_*.dll",
            "cublasLt64_*.dll"
        )
        foreach ($dll in $cudaDlls) {
            Get-ChildItem -Path "$cudaPath\bin" -Filter $dll | ForEach-Object {
                $destination = Join-Path $targetDir $_.Name
                Write-Host "Copying CUDA DLL $($_.Name) to $destination"
                Copy-Item -Path $_.FullName -Destination $destination -Force
            }
        }
    }

} catch {
    Write-Host "Error details:"
    Write-Host $_.Exception.Message
    Write-Host $_.ScriptStackTrace
    Write-Error "Build failed: $_"
    exit 1
} finally {
    # Return to original directory
    Pop-Location
} 