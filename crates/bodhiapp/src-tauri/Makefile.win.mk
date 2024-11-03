.PHONY: build_frontend

build_frontend:
	pwsh -NoProfile -NonInteractive -Command "Set-Item env:PRETTIER_DISABLE '1'; Set-Location ..; pnpm install; pnpm run build"