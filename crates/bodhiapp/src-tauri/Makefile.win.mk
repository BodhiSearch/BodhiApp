.PHONY: build_frontend copy_libs

build_frontend:
	pwsh -NoProfile -NonInteractive -Command " \
		Set-Item env:PRETTIER_DISABLE '1'; \
		Set-Location ..; \
		if (Test-Path .next) { \
			Write-Host 'Cleaning up .next directory...'; \
			Remove-Item -Recurse -Force .next; \
		}; \
		pnpm install; \
		pnpm run build \
	"

copy_libs:
	pwsh -NoProfile -NonInteractive -Command " \
		$$sourcePath = Join-Path '..' '..' '..' 'llamacpp-sys' 'libs'; \
		Write-Host \"Checking source directory: $$sourcePath\"; \
		if (-not (Test-Path $$sourcePath)) { \
			Write-Error \"Source directory not found: $$sourcePath\"; \
			exit 1; \
		}; \
		if (-not (Test-Path libs)) { \
			Write-Host \"Creating libs directory\"; \
			New-Item -ItemType Directory -Path libs; \
		}; \
		Write-Host \"Copying files from $$sourcePath to libs\"; \
		Copy-Item -Path \"$$sourcePath\\*\" -Destination 'libs' -Recurse -Force \
	" 
