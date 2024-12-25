# Get package list from cargo metadata
$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name | ForEach-Object { "-p $_" }
$cleanCmd = "cargo clean $($packages -join ' ')"

Write-Host "Executing command: $cleanCmd"
Invoke-Expression $cleanCmd

# Remove build folders if they exist
Remove-Item -Path "crates/llama_server_proc/llama.cpp/build" -Recurse -Force -ErrorAction SilentlyContinue
