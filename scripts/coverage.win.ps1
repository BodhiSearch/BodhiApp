# scripts/coverage.win.ps1

Write-Host 'Getting package list...'
$packages = (cargo metadata --no-deps --format-version 1 | ConvertFrom-Json).packages |
  Where-Object { $_.name -ne 'integration-tests' } |
  Select-Object -ExpandProperty name

Write-Host 'Packages to test:' $packages
$crates = $packages | ForEach-Object { "-p $_".Trim() } | Join-String -Separator " "
$cmd = "cargo llvm-cov nextest --no-fail-fast --all-features --lcov --output-path lcov.info $crates"
Write-Host "Executing command: $cmd"
Invoke-Expression $cmd