.PHONY: build_frontend

build_frontend:
	pwsh -NoProfile -NonInteractive -Command "Set-Location ..; pnpm install; pnpm run build"