# scripts/coverage.win.ps1

$cmd = "cargo build -p llama_server_proc"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd

$packages = (cargo metadata --no-deps --format-version 1 |
  ConvertFrom-Json).packages.name |
  ForEach-Object { "-p $_" }

$cmd = "cargo llvm-cov test --no-fail-fast --all-features $packages --lcov --output-path lcov.info"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd
