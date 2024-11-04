# scripts/clean.win.ps1

Write-Host 'Getting package list...'
$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages.name
$cmd = "cargo clean " + ($packages | ForEach-Object { "-p $_" } | Join-String -Separator " ")
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd
