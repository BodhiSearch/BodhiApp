# scripts/copy_libs.win.ps1

$sourcePath = Join-Path '..' '..' '..' 'llamacpp-sys' 'libs'
Write-Host "Checking source directory: $sourcePath"

if (-not (Test-Path $sourcePath)) {
  Write-Error "Source directory not found: $sourcePath"
  exit 1
}

if (-not (Test-Path libs)) {
  Write-Host "Creating libs directory"
  New-Item -ItemType Directory -Path libs
}

Write-Host "Copying files from $sourcePath to libs"
Copy-Item -Path "$sourcePath\*" -Destination 'libs' -Recurse -Force