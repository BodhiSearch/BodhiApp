# crates/bodhiapp/src-tauri/scripts/copy_libs.win.ps1

$projectRoot = Join-Path $PSScriptRoot '..' '..' '..' '..'
$sourcePath = Join-Path $projectRoot 'llamacpp-sys' 'libs'
$destPath = Join-Path $PSScriptRoot '..' 'libs'

# Convert to absolute paths
$sourcePath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($sourcePath)
$destPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($destPath)

Write-Host "Source path (absolute): $sourcePath"
Write-Host "Destination path (absolute): $destPath"

if (-not $sourcePath -or -not (Test-Path $sourcePath)) {
    Write-Error "Source directory does not exist: $sourcePath"
    exit 1
}

if (-not (Test-Path $destPath)) {
    Write-Host "Creating destination directory: $destPath"
    New-Item -ItemType Directory -Path $destPath
}

Write-Host "Copying files from $sourcePath to $destPath"
Copy-Item -Path "$sourcePath\*" -Destination $destPath -Recurse -Force
Write-Host "Copy operation completed"