---
title: 'Performance Tuning'
description: 'Choosing variants, quantization, context window, and concurrency to match Bodhi App to your hardware'
order: 3
---

# Performance Tuning

This page is a decision guide for self-hosters: which llama.cpp variant to use on which hardware, how to balance quantization against quality, and how to size context and concurrency without running out of VRAM. The mechanics live in [Inference Stack](/docs/advanced/inference-stack); this page is the _what should I actually choose_ layer on top.

## Variant × hardware

Pick the variant that maps to the dominant compute target on your machine. If you have multiple options, the one further left in this list is usually faster for LLM inference.

| Hardware                                     | First choice                   | Notes                                                                                 |
| -------------------------------------------- | ------------------------------ | ------------------------------------------------------------------------------------- |
| Apple Silicon (M1–M4)                        | `metal`                        | Unified memory means there's no PCIe transfer cost. Beats CPU at every model size.    |
| NVIDIA consumer GPU (RTX 30xx / 40xx / 50xx) | `cuda`                         | The mature path. Use with `--n-gpu-layers 99` to offload all layers when VRAM allows. |
| NVIDIA datacenter GPU (A100 / H100)          | `cuda`                         | Same as above; tensor cores carry. `vulkan` works but isn't as well-optimised.        |
| AMD GPU (RDNA 2/3) on Linux                  | `rocm`                         | Native AMD path. `vulkan` is a portable fallback if ROCm install is painful.          |
| AMD GPU on Windows                           | `vulkan`                       | ROCm Windows support is uneven; Vulkan is the practical default.                      |
| Intel Arc / Data Center GPU Max              | `sycl`                         | Intel's native compute path.                                                          |
| Intel iGPU (UHD/Iris Xe)                     | `vulkan` or CPU                | Useful for tiny models; CPU often wins on iGPUs.                                      |
| Moore Threads GPU                            | `musa`                         | Vendor-specific path.                                                                 |
| Huawei Ascend NPU                            | `cann`                         | Vendor-specific path.                                                                 |
| x86_64 CPU only                              | `cpu_avx512` (then `cpu_avx2`) | Use the highest CPU-feature variant your chip supports.                               |

Set the choice via `BODHI_EXEC_VARIANT` (editable at runtime in the settings UI). The available list on a given install is reported in `BODHI_EXEC_VARIANTS`.

## Quantization tradeoff

Quantization is the lever that most directly buys speed and saves memory.

| Level    | 7B model size | Quality                | Speed (relative) | When to pick                                                 |
| -------- | ------------- | ---------------------- | ---------------- | ------------------------------------------------------------ |
| `F16`    | ~13 GB        | Best                   | 1.0×             | Embeddings; reference quality benchmarking.                  |
| `Q8_0`   | ~7 GB         | Near-best              | ~1.5×            | If you have RAM/VRAM headroom and want minimal quality loss. |
| `Q6_K`   | ~5.5 GB       | Very good              | ~1.7×            | Solid middle ground.                                         |
| `Q5_K_M` | ~4.8 GB       | Very good              | ~1.8×            | Quality-leaning sweet spot.                                  |
| `Q4_K_M` | ~4.4 GB       | Good                   | ~2.0×            | **Recommended default.** Best size/quality balance.          |
| `Q4_K_S` | ~4.0 GB       | Good                   | ~2.1×            | Tighter VRAM budget than `_M`.                               |
| `Q3_K_M` | ~3.4 GB       | Visibly degraded       | ~2.3×            | Only when smaller is non-negotiable.                         |
| `Q2_K`   | ~2.7 GB       | Substantially degraded | ~2.5×            | Last resort for laptop/edge devices.                         |

The "speed (relative)" column is rough — actual speed depends on bandwidth limits more than on arithmetic — but the ordering is reliable.

A useful rule: pick the largest model your VRAM can hold at `Q4_K_M`, not the smallest model at `Q8_0`. A `Q4_K_M` 13B will out-quality a `Q8_0` 7B almost every time.

## Context window sizing

The context window determines how many tokens the model can see at once. Bigger isn't free — VRAM cost grows roughly linearly with context size, dominated by the KV cache.

```bash
# Modest VRAM budget — fast, small context
BODHI_LLAMACPP_ARGS="-c 4096"

# Comfortable budget — fits most chat + tool-use sessions
BODHI_LLAMACPP_ARGS="-c 8192"

# Long-context RAG / document chat — significant VRAM cost
BODHI_LLAMACPP_ARGS="-c 32768"
```

A useful rough estimate: each KV cache slot for a 7B model at `f16` costs roughly half a megabyte per token. 32k context on a 7B model is ~16 GB of KV cache _on top of_ the weights — likely too much for an 8 GB GPU even at heavy quantization.

### KV cache type

If VRAM is tight, you can quantize the KV cache:

```bash
BODHI_LLAMACPP_ARGS="-c 16384 --cache-type-k q8_0 --cache-type-v q8_0"
```

`q8_0` cuts KV cache size roughly in half versus `f16`, with a small quality cost. This is one of the highest-leverage tuning knobs on a constrained GPU.

## Concurrency

By default, llama.cpp serves requests sequentially per process. To handle multiple concurrent requests, use the `--parallel` flag in `BODHI_LLAMACPP_ARGS`:

```bash
# Up to 4 concurrent slots — good for shared dev or small team
BODHI_LLAMACPP_ARGS="-c 16384 --parallel 4 --cont-batching"
```

Parallel slots share the model weights but each gets its own KV cache, so the context-size cost multiplies by the parallel count. Sizing: if you set `-c 16384 --parallel 4`, plan for ~4× the KV cache budget compared to a single-slot setup.

For most desktop installs, `--parallel 1` (the default) is the right answer — chat and tool-use are bursty rather than concurrent. For a Docker single-tenant deployment serving a small team, `--parallel 2` to `--parallel 4` adds throughput without disastrous memory cost.

If you have a use case that genuinely needs high concurrency (a CI bot calling the model in parallel, a batch indexer), spawn multiple aliases pointing at the same GGUF — Bodhi will run them as separate llama-server processes, isolating their memory footprints.

## Cold-start vs. steady-state

Cold start = the time between the first request to a quiet alias and the first token. It's dominated by GGUF load — usually tens of seconds for a 7B `Q4_K_M`, longer for larger or less-quantized models.

Steady-state = subsequent requests hit a warm process and return immediately.

`BODHI_KEEP_ALIVE_SECS` (default `300`) controls how long a warm process is kept alive after the last request:

- **Bursty individual use:** keep the default.
- **Sporadic team use throughout the day:** bump to `1800` (30 min) or `3600` (1 hr) so the popular alias stays warm.
- **Tight memory pressure (multi-alias on one GPU):** drop to `60` or `120` so an idle alias releases VRAM faster.
- **Single-alias dedicated server:** set high (e.g. `7200`); cold starts only happen once per restart anyway.

The setting is editable from the UI at runtime — no restart needed. See [Reference → Settings](/docs/reference/settings).

## macOS specifics — Apple Silicon

Apple Silicon's unified memory means GPU and CPU access the same physical pool. There is no PCIe transfer cost, and VRAM and RAM are the same thing.

- **Use the `metal` variant unconditionally** for anything bigger than a tiny model. CPU-only inference on Apple Silicon is genuinely slow despite the strong cores.
- **Tiny models (< 1B) sometimes prefer CPU** because Metal kernel-launch overhead dominates the actual compute. Benchmark, don't assume.
- **There is no "GPU layers" decision** — Metal handles all layers. Don't pass `--n-gpu-layers` on Metal builds (it's harmless but ignored).
- **Memory pressure shows up as system slowdown**, not as Bodhi crashing — macOS will compress and swap aggressively before refusing the allocation. Watch the Memory tab in Activity Monitor; if "Memory Pressure" turns yellow during inference, drop the context size or pick a smaller model.

## Filesystem notes

GGUF files are large and read-heavy. A few real-world traps:

- **HuggingFace cache uses symlinks.** The `snapshots/` directory contains symlinks into a content-addressed `blobs/` directory. Filesystems that don't preserve symlinks (notably some Windows network shares and FAT32 USB sticks) will break the cache. On Windows, prefer NTFS for `HF_HOME`.
- **Linux on case-insensitive filesystems.** Repos with mixed-case filenames (rare, but not unheard of) misbehave on a case-insensitive ext4 (some distros configure this) — stick to the default case-sensitive setup.
- **Network filesystems (NFS, SMB) for HF_HOME.** Workable for small teams sharing a model pool, but cold-start latency goes up by the network round-trip cost. Not recommended for production.
- **macOS bundle quarantine.** Files downloaded by other apps may carry a `com.apple.quarantine` extended attribute that confuses some tools. The HuggingFace downloader Bodhi uses is fine; this affects only models you copied in by hand.

## Diagnostics

If performance feels wrong, check in this order:

1. **Variant.** `BODHI_EXEC_VARIANT` actually matches your hardware? Check the variants surface in the settings page.
2. **GPU offload.** For CUDA/ROCm/Vulkan, are you setting `--n-gpu-layers 99`? Without it, layers run on CPU even with the GPU variant.
3. **Quantization.** Are you accidentally on `F16` weights when a `Q4_K_M` would fit?
4. **Context size vs. VRAM.** Set `nvidia-smi` (or equivalent) running while you load the model. If you see allocation right up to the VRAM ceiling, drop `-c` or use `--cache-type-k q8_0`.
5. **Concurrency.** A `--parallel 4` config on a tight VRAM budget can OOM the moment four users arrive simultaneously even if singular use was fine.

For a wider observability picture (logs, settings introspection), see [Observability](/docs/advanced/observability).
