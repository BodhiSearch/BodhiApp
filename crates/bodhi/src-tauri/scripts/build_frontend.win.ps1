# crates/bodhi/src-tauri/scripts/build_frontend.win.ps1

$env:PRETTIER_DISABLE = '1'
Set-Location ..

if (Test-Path dist) {
  Write-Host 'Cleaning up dist directory...'
  Remove-Item -Recurse -Force dist
}

Write-Host 'Installing dependencies...'
pnpm install

Write-Host 'Building frontend...'
pnpm run build