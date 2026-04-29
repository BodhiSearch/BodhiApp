---
title: 'Inference Stack'
description: 'How Bodhi App invokes llama.cpp: variants, GGUF resolution, runtime arguments, and the keep-alive timer'
order: 2
---

# Inference Stack

Local inference in Bodhi App is llama.cpp. When you start a chat against a local alias, Bodhi spawns the `llama-server` binary as a child process, talks to it over HTTP on a private port, and shuts it down after an idle timeout. This page is the operator's view of that pipeline: which binary gets picked, how it's configured, and which knobs to turn.

If you're choosing between hardware backends, [Performance Tuning](/docs/advanced/performance-tuning) is the next page. If you just want to point an alias at a downloaded GGUF, [Features → Model Aliases](/docs/features/models/model-alias) covers the user-facing configuration.

## llama.cpp variants

llama.cpp doesn't ship as a single binary — it's compiled per-backend. Bodhi's Docker images each bundle one or more variants of `llama-server`, and the desktop installer ships the variants appropriate for your platform. The supported set includes:

| Variant               | Target hardware                                                     |
| --------------------- | ------------------------------------------------------------------- |
| `cpu` (avx2 / avx512) | Any x86_64 with the corresponding CPU feature flags                 |
| `metal`               | Apple Silicon (M1/M2/M3/M4) — uses the unified-memory Metal backend |
| `cuda`                | NVIDIA GPUs (consumer + datacenter)                                 |
| `rocm`                | AMD GPUs on Linux                                                   |
| `vulkan`              | Cross-vendor GPU backend (AMD, Intel, NVIDIA fallback)              |
| `musa`                | Moore Threads GPUs                                                  |
| `sycl`                | Intel iGPU / Arc / Data Center GPU Max                              |
| `cann`                | Huawei Ascend NPUs                                                  |

Each variant maps to a separate `llama-server` build — the binary name and ABI are the same, but the linked compute kernels are different. The Docker tags reflect the variant (`bodhi:cpu`, `bodhi:cuda`, `bodhi:rocm`, etc.); the desktop builds ship with the variant matching the host (Metal on Apple Silicon, AVX2/AVX512 + Vulkan on Windows, CUDA-enabled builds on supported Linux).

## Selecting the variant at runtime

Four environment variables control how Bodhi finds and picks the right `llama-server` binary:

| Variable                 | Purpose                               | Typical value                         |
| ------------------------ | ------------------------------------- | ------------------------------------- |
| `BODHI_EXEC_LOOKUP_PATH` | Directory tree to search for binaries | Set by the installer / Docker image   |
| `BODHI_EXEC_VARIANT`     | Which variant to invoke               | `cpu_avx2`, `cuda`, `metal`, ...      |
| `BODHI_EXEC_NAME`        | Binary filename                       | Usually `llama-server`                |
| `BODHI_EXEC_TARGET`      | OS/arch sub-folder                    | e.g. `linux-x86_64`, `darwin-aarch64` |

`BODHI_EXEC_VARIANT` is also one of the small set of settings you can change from the UI at runtime (see [Reference → Settings](/docs/reference/settings)). The other three are typically baked in by the install method and shouldn't need touching.

The available variants on a given install are exposed via `BODHI_EXEC_VARIANTS` (a comma-separated list); the settings page reads from this when offering a dropdown.

## What happens when an alias starts

When a request resolves to a local alias and there's no llama-server already running for it:

1. Bodhi composes the command line: `llama-server --port <private-port> --model <gguf-path> ...`
2. It appends the alias's `context_params` (per-alias overrides — context size, GPU layers, threads, anything llama-server's CLI accepts).
3. It appends `BODHI_LLAMACPP_ARGS` (global defaults that apply to _every_ alias).
4. The child process is spawned with stdout/stderr captured into the app's log stream.
5. Bodhi polls the child's `/health` endpoint until it reports ready, then forwards the original request.

Cold start is dominated by GGUF load time — usually tens of seconds for a 7B Q4_K_M, longer for larger or less-quantized models. After warm-up, requests are served immediately.

## `BODHI_LLAMACPP_ARGS` — global llama-server defaults

`BODHI_LLAMACPP_ARGS` is a string passed verbatim to every spawned `llama-server`. Anything llama-server's CLI accepts is fair game. Common uses:

```bash
# Larger default context, half-precision KV cache, 4 parallel slots
BODHI_LLAMACPP_ARGS="--ctx-size 8192 --cache-type-k f16 --cache-type-v f16 --parallel 4"

# Offload everything to GPU, 8 threads on the host CPU for non-GPU work
BODHI_LLAMACPP_ARGS="--n-gpu-layers 99 --threads 8"

# Smaller default context to save VRAM on a shared 8GB GPU
BODHI_LLAMACPP_ARGS="--ctx-size 2048 --n-gpu-layers 32"
```

Per-alias `context_params` (configured in [Model Aliases](/docs/features/models/model-alias)) win when they collide with `BODHI_LLAMACPP_ARGS` — the alias is the more specific configuration.

The full flag list comes from the upstream project: see [llama.cpp server documentation](https://github.com/ggml-org/llama.cpp/tree/master/tools/server). Be aware that flag names occasionally change between llama.cpp versions; pin to the version Bodhi ships against rather than copy/pasting from blog posts.

## `BODHI_KEEP_ALIVE_SECS` — the unload timer

After a llama-server process serves a request, Bodhi keeps it warm for `BODHI_KEEP_ALIVE_SECS` seconds (default `300`, i.e. 5 minutes). If no further requests arrive before the timer expires, the process is terminated and its memory is returned to the OS.

Why this matters:

- **Lower values** keep RAM/VRAM free, which is critical on machines that run multiple aliases or share the GPU with other workloads. A 7B Q4_K_M model holds ~4-5 GB; an unused process holding that hostage hurts.
- **Higher values** save cold-start time. If your team chats sporadically throughout the day, bumping this to `1800` (30 min) or `3600` (1 hr) keeps the most-used model warm without you babysitting it.
- **Setting it to `0`** is not "disabled"; it's "unload immediately after every request." Don't.

Like `BODHI_EXEC_VARIANT`, this is editable from the settings UI at runtime — no restart required. See [Reference → Settings](/docs/reference/settings) for the editable subset.

## GGUF — the model file format

Local models in Bodhi are GGUF files: a self-contained binary format that bundles model weights, tokenizer, and metadata into a single file. Bodhi resolves them out of your HuggingFace cache (default `$BODHI_HOME/hf_home`), and the alias points at one repo + filename.

### Quantization at a glance

The same model is published in multiple quantization levels. Lower bits = smaller file = faster inference = more quality loss. Common levels:

| Level    | Bits/weight (avg) | Note                                                           |
| -------- | ----------------- | -------------------------------------------------------------- |
| `F16`    | 16                | Half-precision; full quality, big file. Mostly for embeddings. |
| `Q8_0`   | 8                 | Near-full quality. Good baseline if you have the disk.         |
| `Q6_K`   | 6                 | Strong quality, modest savings.                                |
| `Q5_K_M` | 5                 | Common sweet spot for quality-conscious users.                 |
| `Q4_K_M` | 4                 | The popular default — best size/quality balance.               |
| `Q4_K_S` | 4                 | Smaller than `_M` at slightly more quality cost.               |
| `Q3_K_M` | 3                 | Aggressive — usable but visibly worse.                         |
| `Q2_K`   | 2                 | Last-resort tiny; significant quality loss.                    |

If you don't have a strong reason to pick something else, **`Q4_K_M` is the default we recommend.** The community converges on it because it fits a 7B model in ~4.5 GB while keeping output quality very close to the un-quantized original.

For the format spec itself, see the [llama.cpp GGUF reference](https://github.com/ggml-org/llama.cpp/blob/master/docs/ggml.md).

### Where files live

Bodhi follows the standard HuggingFace cache layout. A 7B llama-3 download lands under:

```
$HF_HOME/
└── hub/
    └── models--bartowski--Meta-Llama-3-8B-Instruct-GGUF/
        ├── snapshots/
        │   └── <sha>/
        │       └── Meta-Llama-3-8B-Instruct-Q4_K_M.gguf
        └── ...
```

Bodhi never moves these files. If you have an existing `$HF_HOME` on a fast disk, point Bodhi at it and re-use it. See [Features → Model Files](/docs/features/models/model-files) for the import flow.

## When to override per-alias vs. globally

A useful default rule:

- **Set on `BODHI_LLAMACPP_ARGS`**: things that depend on the _machine_ — number of GPU layers (because you have a fixed VRAM budget), CPU thread count, KV cache type.
- **Set on the alias's `context_params`**: things that depend on the _use case_ — context size for a long-document RAG alias, smaller context for a fast chat alias, parallel slots tuned per workload.

Both layers compose, so it's fine to put a sensible host-wide default in `BODHI_LLAMACPP_ARGS` and override on a single alias when needed.

## Where to go next

- [Performance Tuning](/docs/advanced/performance-tuning) — variant × hardware decisions, quantization tradeoffs, concurrency.
- [Reference → Environment Variables](/docs/reference/env-vars) — the full env-var matrix.
- [Reference → Settings](/docs/reference/settings) — runtime-editable settings (precedence + UI).
- [Features → Model Aliases](/docs/features/models/model-alias) — `context_params` and the per-alias UI.
