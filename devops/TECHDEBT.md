# devops — TECHDEBT

## Multi-Tenant Base Image

The `multi-tenant.Dockerfile` and `multi-tenant-arm64.Dockerfile` currently use
`ghcr.io/bodhisearch/llama.cpp:latest-cpu` as base image for structural consistency
with other variants. However, multi-tenant mode uses `MultitenantInferenceService`
which proxies to external LLM APIs — it never spawns a local llama.cpp process.

**Improvement**: Switch to `debian:bookworm-slim` as the base image. This would:
- Reduce image size by ~200MB (no llama-server binary or CPU-optimized libraries)
- Remove unused BODHI_EXEC_* and BODHI_LLAMACPP_* settings from defaults.yaml
- Require creating the `llama` user manually (currently provided by the llama.cpp base)
