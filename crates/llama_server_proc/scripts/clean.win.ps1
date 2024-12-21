# scripts/clean.win.ps1

$ErrorActionPreference = "Stop"  # Stop on any error

# Remove build folders if they exist
Remove-Item -Path "llama.cpp/build" -Recurse -Force -ErrorAction SilentlyContinue

# Return success
exit 0
