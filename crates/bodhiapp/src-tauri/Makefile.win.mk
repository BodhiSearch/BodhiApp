.PHONY: build_frontend copy_libs

build_frontend:
	@pwsh -NoProfile -File scripts/build_frontend.win.ps1

copy_libs:
	@pwsh -NoProfile -File scripts/copy_libs.win.ps1
