# scripts/coverage.win.ps1

$packages = (cargo metadata --no-deps --format-version 1 |
  ConvertFrom-Json).packages.name |
  Where-Object { $_ -ne "llamacpp_sys" } |
  ForEach-Object { "-p $_" }

$cmd = "cargo llvm-cov nextest --no-fail-fast --all-features $packages --lcov --output-path lcov.info"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd
