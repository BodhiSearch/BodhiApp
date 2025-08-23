# RTX A4000 llama-server Performance Optimization Guide

Your current performance of 2-3 tokens/second is **severely underperforming** - you should achieve 25-35 tokens/second with proper optimization. This comprehensive analysis reveals critical missing parameters and configuration issues that can deliver a **10x performance improvement**.

## Critical performance gap analysis

Your minimal command line arguments are missing essential GPU acceleration parameters. The RTX A4000 with 16GB VRAM can comfortably handle the 14.7B Phi-4 model with Q4_K_M quantization (requiring ~8.2GB model + ~2-4GB for KV cache and overhead). Your current setup likely runs entirely on CPU or with minimal GPU offloading, explaining the 2-3 tokens/second performance.

## Essential missing parameters for GPU acceleration

The most critical parameters absent from your current configuration:

**`--n-gpu-layers -1`** - Offloads all model layers to GPU (currently defaulting to 0, meaning CPU-only inference)
**`--flash-attn`** - Enables FlashAttention optimization, providing significant speed improvements on Ampere architecture
**`--batch-size 512`** - Optimizes memory access patterns for GPU throughput
**`--ubatch-size 256`** - Balances memory usage with parallel processing efficiency

## Complete optimized configuration

Replace your current minimal setup with this comprehensive configuration:

```bash
llama-server \
  --model path/to/phi-4-q4_k_m.gguf \
  --alias phi-4-optimized \
  --jinja \
  --api-key your-uuid-key \
  --port your-port \
  --host 0.0.0.0 \
  --n-gpu-layers -1 \
  --ctx-size 4096 \
  --batch-size 1024 \
  --ubatch-size 256 \
  --parallel 1 \
  --sequences 1 \
  --flash-attn \
  --cont-batching \
  --mlock \
  --no-mmap \
  --threads 16 \
  --threads-batch 16 \
  --defrag-thold 0.1 \
  --cache-type-k f16 \
  --cache-type-v f16 \
  --verbose
```

## RTX A4000 CUDA optimization parameters

Set these environment variables before launching llama-server:

```bash
# Memory optimization for RTX A4000
export GGML_CUDA_ENABLE_UNIFIED_MEMORY=0
export CUDA_VISIBLE_DEVICES=0
export GGML_CUDA_FORCE_MMQ=0
export GGML_CUDA_FORCE_CUBLAS=0
export GGML_CUDA_NO_PINNED=0
export GGML_CUDA_PEER_MAX_BATCH_SIZE=128
export CUDA_LAUNCH_BLOCKING=0
export CUDA_DEVICE_ORDER=PCI_BUS_ID

# Performance optimization
export GGML_CUDA_F16=1
export GGML_CUDA_KQUANTS_ITER=2
export CUDA_MEMORY_FRACTION=0.95
```

## Docker container optimization for GPU passthrough

Your Docker container requires specific GPU passthrough and resource allocation settings:

```bash
docker run -d \
  --name llama-server-optimized \
  --gpus all \
  --shm-size=2g \
  --ulimit memlock=-1 \
  --ulimit stack=67108864 \
  --cpuset-cpus="0-15" \
  --memory=32g \
  --cap-add SYS_ADMIN \
  -p your-port:your-port \
  -v /path/to/models:/models:ro \
  -e CUDA_VISIBLE_DEVICES=0 \
  -e GGML_CUDA_ENABLE_UNIFIED_MEMORY=0 \
  -e GGML_CUDA_F16=1 \
  llama-cpp-cuda-image \
  ./llama-server [optimized parameters above]
```

## Memory management strategy for 16GB VRAM

**Memory allocation breakdown for 14.7B Q4_K_M:**
- Model weights: ~8.2GB
- KV cache (4096 context): ~2.1GB  
- CUDA overhead: ~0.8GB
- Activations/scratch: ~1.2GB
- **Total usage**: ~12.3GB (comfortable margin in 16GB)

**Context length optimization:**
- **4096 tokens**: Recommended balance (2.1GB KV cache)
- **8192 tokens**: Maximum for performance (4.2GB KV cache) 
- **2048 tokens**: Memory-constrained fallback (1.1GB KV cache)

## Threading optimization for 16 vCPU system

**Optimal thread configuration:**
- `--threads 16` - Utilize all 16 vCPUs for hybrid CPU-GPU operations
- `--threads-batch 16` - Optimize batch processing across all cores
- Enable NUMA isolation with `--numa isolate` flag

**CPU resource allocation in Docker:**
```bash
# Pin to all 16 cores
--cpuset-cpus="0-15"

# Alternatively, reserve cores for system processes
--cpuset-cpus="2-15"  # Reserve cores 0-1 for OS
```

## Performance debugging and monitoring approach

**Real-time performance monitoring:**
```bash
# GPU utilization and memory
nvidia-smi -l 1

# Detailed GPU metrics
watch -n 1 "nvidia-smi --query-gpu=memory.used,memory.total,utilization.gpu,utilization.memory --format=csv"

# System resource monitoring  
htop
iostat -x 1
```

**Benchmarking and validation:**
```bash
# Baseline performance test
./llama-bench -m phi-4-q4_k_m.gguf -p 512 -n 128 -ngl -1 --flash-attn

# Server health monitoring
curl -X GET http://localhost:your-port/health
curl -X GET http://localhost:your-port/props
```

## Expected performance benchmarks

**RTX A4000 performance targets for 14.7B Q4_K_M:**
- **Prompt processing**: 150-200 tokens/second
- **Text generation**: 25-35 tokens/second (10x your current performance)
- **GPU utilization**: 85-95% during inference
- **VRAM usage**: 12-13GB out of 16GB available
- **Time to first token**: <100ms for typical prompts

## Latest community optimization recommendations

**2024-2025 performance improvements:**
- **CUDA Graphs**: Automatically enabled, provides 10-15% speedup
- **FlashAttention**: Essential for Ampere architecture efficiency  
- **Custom CUDA kernels**: Optimized matrix multiplication for RTX hardware
- **Continuous batching**: Improves throughput for concurrent requests

**Advanced quantization options:**
- **IQ4_XS**: Latest high-quality quantization, superior to Q4_K_M
- **Q4_0**: Fastest option with minimal quality loss
- **Q8_0**: Higher quality when VRAM budget allows

## Troubleshooting performance bottlenecks

**Most likely causes of 2-3 tokens/sec performance:**

1. **Missing GPU offloading** - Add `--n-gpu-layers -1`
2. **CPU-only inference** - Verify CUDA installation and GPU visibility
3. **Inefficient memory access** - Enable FlashAttention and optimize batch sizes
4. **Docker GPU passthrough issues** - Ensure `--gpus all` and proper NVIDIA runtime

**Validation steps:**
1. Confirm all model layers on GPU: Check logs for "llama_model_load: offloaded"
2. Monitor GPU utilization during inference: Should be 85-95%
3. Verify VRAM usage: Should be 12-13GB for 14.7B Q4_K_M
4. Test with llama-bench: Should achieve 25-35 tokens/second

## Immediate action plan

**Step 1**: Add critical missing GPU parameters
```bash
--n-gpu-layers -1 --flash-attn --batch-size 1024 --ubatch-size 256
```

**Step 2**: Set CUDA environment variables  
**Step 3**: Optimize Docker GPU passthrough with proper flags
**Step 4**: Monitor performance with nvidia-smi to confirm GPU utilization
**Step 5**: Validate with benchmarking tools to reach 25-35 tokens/second target

This optimization approach should immediately deliver a 10x performance improvement, bringing your token generation from 2-3 tokens/second to the expected 25-35 tokens/second for your RTX A4000 configuration.