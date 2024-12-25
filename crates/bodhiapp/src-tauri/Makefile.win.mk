.PHONY: build_frontend copy_bins

build_frontend:
	@pwsh -NoProfile -File scripts/build_frontend.win.ps1

copy_bins:
	@pwsh -NoProfile -File scripts/copy_bins.win.ps1
