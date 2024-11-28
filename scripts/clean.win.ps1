# Get package list from cargo metadata
$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name | ForEach-Object { "-p $_" }
$cleanCmd = "cargo clean $($packages -join ' ')"

Write-Host "Executing command: $cleanCmd"
Invoke-Expression $cleanCmd

# Remove build folders if they exist
Remove-Item -Path "llamacpp-sys/build" -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -Path "llamacpp-sys/llama.cpp/build" -Recurse -Force -ErrorAction SilentlyContinue
