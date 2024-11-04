# scripts/build_frontend.win.ps1

$env:PRETTIER_DISABLE = '1'
Set-Location ..

if (Test-Path .next) {
  Write-Host 'Cleaning up .next directory...'
  Remove-Item -Recurse -Force .next
}

Write-Host 'Installing dependencies...'
pnpm install

Write-Host 'Building frontend...'
pnpm run build