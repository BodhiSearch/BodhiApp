.PHONY: build_frontend copy_libs

build_frontend:
	cd .. && pnpm install && pnpm run build

copy_libs:
	@if [ ! -d "../../../llamacpp-sys/libs" ]; then \
		echo "Error: Source directory '../../../llamacpp-sys/libs' not found"; \
		exit 1; \
	fi
	@echo "Checking source directory: ../../../llamacpp-sys/libs"
	@echo "Creating libs directory"
	mkdir -p libs
	@echo "Copying files from ../../../llamacpp-sys/libs to libs"
	cp -r ../../../llamacpp-sys/libs/* libs/
