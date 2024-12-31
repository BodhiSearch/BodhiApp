# scripts/coverage.win.ps1

# Stop on first error
$ErrorActionPreference = "Stop"

try {
  $cmd = "cargo build -p llama_server_proc"
  Write-Host "Executing command: $cmd"
  Invoke-Expression $cmd
  if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
  }

  $packages = (cargo metadata --no-deps --format-version 1 |
    ConvertFrom-Json).packages.name |
    ForEach-Object { "-p $_" }

  $cmd = "cargo llvm-cov test --no-fail-fast --all-features $packages --lcov --output-path lcov.info"
  Write-Host "Executing command: $cmd"
  Invoke-Expression $cmd
  if ($LASTEXITCODE -ne 0) {
    throw "cargo llvm-cov test failed with exit code $LASTEXITCODE"
  }
} catch {
  Write-Host "Error: $_" -ForegroundColor Red
  exit 1
}
