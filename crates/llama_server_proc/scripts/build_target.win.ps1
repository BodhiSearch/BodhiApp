param(
    [Parameter(Mandatory=$true)]
    [string]$CmakeFlags,

    [Parameter(Mandatory=$true)]
    [string]$TargetArch,

    [Parameter(Mandatory=$true)]
    [string]$BuildType
)

# Constants moved from Makefile
$BUILD_DIR = "llama.cpp/build"
$CMAKE_COMMON_FLAGS = "-DLLAMA_FATAL_WARNINGS=OFF -DLLAMA_CURL=OFF -DBUILD_LLAMA_SERVER=ON"
# Change to llama.cpp directory from the script's location
Push-Location (Join-Path $PSScriptRoot "../llama.cpp")

# Add at the start of the file
$ErrorActionPreference = "Stop"  # Stop on any error

try {
    # Debug output to verify flags
    Write-Host "CMAKE_COMMON_FLAGS: $CMAKE_COMMON_FLAGS"
    Write-Host "CmakeFlags: $CmakeFlags"

    # Combine CMAKE flags - flatten the arrays properly
    $allFlags = ($CMAKE_COMMON_FLAGS.Split(" ") + $CmakeFlags.Split(" ")) | Where-Object { $_ }
    $CMAKE_FULL_FLAGS = $allFlags -join " "

    Write-Host "CMAKE_FULL_FLAGS: $CMAKE_FULL_FLAGS"
    Write-Host '=== Running CMake Configure ==='
    $cmakeCommand = "cmake -S . -B build $CMAKE_FULL_FLAGS"
    Write-Host "Executing command: $cmakeCommand"
    Invoke-Expression $cmakeCommand

    if ($LASTEXITCODE -eq 0) {
        Write-Host '=== Running CMake Build ==='
        $numCpu = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
        cmake --build build --config Release --target llama-server -j $numCpu
        if ($LASTEXITCODE -eq 0) {
            Write-Host '=== Copying Build Artifacts ==='
            $destPath = Join-Path $PSScriptRoot '..' 'bin'
            $destPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destPath)
            $sourceDir = Join-Path (Get-Location) 'build'
            $sourceDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($sourceDir)
            $targetDir = Join-Path $destPath $TargetArch $BuildType

            # Get all DLL files recursively in the build directory and its subdirectories
            Get-ChildItem -Path $sourceDir -Filter "llama-server.exe" -Recurse | ForEach-Object {
                $fileName = $_.Name
                # Create target directory if it doesn't exist
                New-Item -ItemType Directory -Force $targetDir | Out-Null

                # Determine destination path
                $destination = Join-Path $targetDir $fileName
                Write-Host "Copying $fileName to $destination"
                Copy-Item -Path $_.FullName -Destination $destination -Force -ErrorVariable copyError
                if ($copyError) {
                    Write-Error "Copy failed for $fileName`: $copyError"
                    exit 1
                }
            }
        } else {
            Write-Error 'Error: Build failed during cmake build step'
            exit 1
        }
    } else {
        Write-Error 'Error: Build failed during cmake configure step'
        exit 1
    }
} finally {
    # Always return to the original directory
    Pop-Location
}
