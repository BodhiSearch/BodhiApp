# llama.cpp Context Parameters

Conceptual reference for the low-level llama-server context parameters that control how a model loads and generates. These are llama.cpp server knobs, not BodhiApp domain types.

> **Current ownership**: server-side context flags are passed as raw llama-server CLI args (e.g. via `BODHI_LLAMACPP_ARGS_*`; see `devops/cuda.Dockerfile` and `docs/conventions/cuda-dockerfile-optimizations.md`) and handled by the `llama_server_proc` crate. Per-request generation params (temperature, max tokens, etc.) live in `OAIRequestParams` on the alias — see `crates/services/src/models/model_objs.rs` and `crates/services/CLAUDE.md`. The old per-alias `n_ctx`/`n_seed`/`n_keep` context-params struct and its JSON config blocks have been removed; this doc is kept for the parameter explanations only.

## Core Parameters

| Param | llama.cpp flag | Default | Effect |
|-------|----------------|---------|--------|
| Context window | `--ctx-size` (`n_ctx`) | model-dependent | Prompt+generation window size; drives KV-cache memory and max conversation length. |
| Seed | `--seed` (`n_seed`) | random | Fixed seed → reproducible outputs (same seed + settings ≈ same response). |
| Threads | `--threads` (`n_threads`) | system CPU count | CPU threads for compute; match to cores, watch other load. |
| Parallel sequences | `--parallel` (`n_parallel`) | 1 | Concurrent request slots; raise for multi-user, balance against VRAM/RAM. |
| Predict limit | `--n-predict` (`n_predict`) | -1 (unbounded) | Max tokens to generate; -1 lets the model decide. |
| Keep tokens | `--keep` (`n_keep`) | 0 | Initial-prompt tokens retained when context is recycled; preserves system instructions. |

## Tuning Guidance

- **Memory pressure** → lower `--ctx-size` and `--parallel`; consider KV-cache quantization (`--cache-type-k/v q8_0`).
- **Slow responses** → raise `--threads` (up to core count), check system load.
- **Inconsistent outputs** → set a fixed `--seed` for testing; vary it in production for diversity.
- **Multi-user** → raise `--parallel` with `--cont-batching`, monitoring stability.

Start from model defaults, change one knob at a time, and measure tokens/sec and VRAM. For GPU-specific tuning of these same flags, see `docs/conventions/cuda-dockerfile-optimizations.md`.
