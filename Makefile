test:
	cargo test
	cd bodhi-ui && npm run test
	cd openai-pysdk-compat && poetry run pytest || true

format:
	cd bodhi-ui && npm run format && npm run lint
	cargo fmt -p bodhicore -p bodhi
	cd openai-pysdk-compat && poetry run ruff format .

.PHONY: test format
