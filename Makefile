test:
	cargo test
	cd bodhi-ui && npm run test
	cd openai-pysdk-compat && poetry run pytest || true

.PHONY: test
