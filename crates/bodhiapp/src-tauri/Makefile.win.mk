.PHONY: build_frontend copy_libs

build_frontend:
	pwsh -NoProfile -NonInteractive -Command "Set-Item env:PRETTIER_DISABLE '1'; Set-Location ..; pnpm install; pnpm run build"

copy_libs:
	pwsh -NoProfile -NonInteractive -Command "\
		$$sourcePath = Join-Path -Path '..' -ChildPath '..' -AdditionalChildPath '..', 'llamacpp-sys', 'libs'; \
		Write-Host \"Checking source directory: $$sourcePath\"; \
		if (-not (Test-Path $$sourcePath)) { \
			Write-Error \"Source directory not found: $$sourcePath\"; \
			exit 1 \
		} \
	"
	pwsh -NoProfile -NonInteractive -Command "\
		if (-not (Test-Path libs)) { \
			Write-Host \"Creating libs directory\"; \
			New-Item -ItemType Directory -Path libs \
		} \
	"
	pwsh -NoProfile -NonInteractive -Command "\
		$$sourcePath = Join-Path -Path '..' -ChildPath '..' -AdditionalChildPath '..', 'llamacpp-sys', 'libs'; \
		Write-Host \"Copying files from $$sourcePath to libs\"; \
		Copy-Item -Path \"$$sourcePath\*\" -Destination 'libs' -Recurse -Force \
	"