# scripts/coverage.win.ps1

$cmd = "cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd