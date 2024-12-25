.PHONY: build_frontend copy_libs

build_frontend:
	cd .. && pnpm install && pnpm run build

copy_bins:
	@if [ ! -d "../../llama_server_proc" ]; then \
		echo "Error: Source directory '../../llama_server_proc' not found"; \
		exit 1; \
	fi
	@echo "Creating bin directory"
	mkdir -p bin
	@echo "Copying files from ../../llama_server_proc/bin to bin"
	cp -r ../../llama_server_proc/bin/* bin/