# scripts/clean.win.ps1

$cmd = "cargo clean"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd
