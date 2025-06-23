# crates/bodhi/src-tauri/scripts/build_frontend.win.ps1

$env:PRETTIER_DISABLE = '1'
Set-Location ..

if (Test-Path out) {
  Write-Host 'Cleaning up out directory...'
  Remove-Item -Recurse -Force out
}

Write-Host 'Installing dependencies...'
npm install

Write-Host 'Building frontend...'
npm run build