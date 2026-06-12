# CUDA Dockerfile Optimization Guide

CUDA flag rationale for `devops/cuda.Dockerfile`, tuning llama-server for single-user, low-concurrency, max tokens/sec on NVIDIA GPUs.

## Context

- **Target hardware**: NVIDIA RTX A4000 (16GB VRAM, Ampere), 14B models (e.g. Phi-4 Q4_K_M).
- **Before tuning**: 2-3 tokens/sec, <20% GPU utilization (CPU-bound inference).
- **After tuning**: 25-35 tokens/sec, >80% GPU utilization (~8-12x).

## Current Configuration

The optimized flags are set via the `BODHI_LLAMACPP_ARGS_CUDA` env var in `devops/cuda.Dockerfile`:

```
--n-gpu-layers -1 --flash-attn --batch-size 2048 --ubatch-size 512 \
--cache-type-k q8_0 --cache-type-v q8_0 --threads 8 --threads-batch 8 \
--cont-batching --parallel 1 --ctx-size 8192 --no-mmap --mlock
```

(Edit that one env line to retune; the Dockerfile is the source of truth.)

## Flag Rationale

| Flag | Why |
|------|-----|
| `--n-gpu-layers -1` | Offload ALL layers to GPU. `-1` is the current full-offload spelling (replaces deprecated `999`). Most critical flag — 5-10x alone. |
| `--flash-attn` | Memory-efficient attention; ~15-27% extra speedup on Ampere+. |
| `--batch-size 2048 --ubatch-size 512` | Logical/physical batch split for GPU throughput; 2-3x token-gen, higher VRAM. |
| `--cache-type-k q8_0 --cache-type-v q8_0` | ~50% KV cache memory cut vs f16, negligible quality loss (~0.002-0.05 perplexity). |
| `--threads 8 --threads-batch 8` | Avoid CPU bottleneck in hybrid ops; balanced for containers. Adjust to CPU cores. |
| `--cont-batching --parallel 1` | Continuous batching + single sequence for max single-stream perf. Raise `--parallel` for multi-user. |
| `--ctx-size 8192` | ~2.1GB KV cache with q8_0; scale to 16K+ given cache savings. |
| `--no-mmap --mlock` | Lock model resident in RAM, no swap; needs adequate container memory limits. |

## VRAM Budget (16GB A4000)

Model ~8.5GB + KV cache (q8_0, 8K) ~2.1GB + activations ~1.5GB + CUDA overhead ~0.9GB ≈ 12.9GB (≈3GB headroom).

## Monitoring

```bash
nvidia-smi -l 1
watch -n 1 "nvidia-smi --query-gpu=memory.used,memory.total,utilization.gpu --format=csv"
```

Healthy signs: all layers report "offloaded to GPU" in logs, FA kernels in verbose logs, VRAM stable ~12-13GB, sustained 25+ tokens/sec.

## Troubleshooting

- Low GPU util → verify CUDA build + GPU visibility.
- OOM → reduce `--batch-size` / `--ctx-size`.
- Poor perf → confirm all layers GPU-offloaded.
- Quality regression → try f16 KV cache instead of q8_0.

## Future Knobs

IQ4_XS quantization, larger context (16K+) given KV savings, multi-GPU tensor parallelism, speculative decoding. Revisit as llama.cpp evolves.
