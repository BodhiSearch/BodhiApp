.PHONY: build_frontend

build_frontend:
	pwsh -NoProfile -NonInteractive -Command "Set-Location ..; pnpm install; $$env:PRETTIER_DISABLE='1'; pnpm run build"