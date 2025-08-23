
## llama-server Parameters

This document outlines the various parameters available for the `llama-server` executable, along with their descriptions and potential impact on performance, especially for GPU acceleration.

### Model Settings (`ModelSettings`)

These settings are crucial for how the model is loaded and utilized.

*   `model`: The path to the model to use for generating completions.
*   `model_alias`: The alias of the model to use for generating completions.
*   `n_gpu_layers` (default=0, ge=-1): The number of layers to put on the GPU. The rest will be on the CPU. Set -1 to move all to GPU.
    *   **Optimization Note:** For maximum GPU utilization, this should be set to `-1` if your GPU has enough VRAM to hold the entire model. For an A4000 with 16GB VRAM and a 14.7B parameter model (Phi-4-Q4_K_M.gguf), it's highly probable that the entire model can fit. This is a critical parameter for performance.
*   `split_mode`: The split mode to use. (e.g., `LLAMA_SPLIT_MODE_LAYER`)
*   `main_gpu` (default=0, ge=0): Main GPU to use.
*   `tensor_split`: Split layers across multiple GPUs in proportion.
*   `vocab_only` (default=False): Whether to only return the vocabulary.
*   `use_mmap` (default=(llama_cpp.llama_supports_mmap())): Use mmap.
*   `use_mlock` (default=(llama_cpp.llama_supports_mlock())): Use mlock.
*   `kv_overrides`: List of model kv overrides in the format `key=type:value` where type is one of (bool, int, float). Valid true values are (true, TRUE, 1), otherwise false.

### Server Settings (`ServerSettings`)

These settings configure the FastAPI and Uvicorn server.

*   `host` (default='localhost'): Listen address.
*   `port` (default=8000): Listen port.
*   `ssl_keyfile` (default=None): SSL key file for HTTPS.
*   `ssl_certfile` (default=None): SSL certificate file for HTTPS.
*   `api_key` (default=None): API key for authentication. If set all requests need to be authenticated.
*   `interrupt_requests` (default=True): Whether to interrupt requests when a new request is received.
*   `disable_ping_events` (default=False): Disable EventSource pings.
*   `root_path` (default=''): The root path for the server. Useful when running behind a reverse proxy.

### Other Important Parameters (from general llama.cpp usage and common optimizations)

While not explicitly listed in the `llama-cpp-python` server settings, these are often critical for `llama.cpp` performance and may be configurable via environment variables or direct `llama-server` arguments (if `llama-cpp-python` passes them through).

*   `--n-ctx` (context size): This determines the maximum context window the model can handle. A larger context window allows the model to 



## Model and Hardware Specifications

### Microsoft Phi-4 Model

*   **Parameters:** 14 billion parameters
*   **Context Window:** 16K tokens (as per some sources, though this can vary with GGUF quantization and `llama.cpp` settings)
*   **GGUF Quantization:** The user is using `phi-4-Q4_K_M.gguf`. Q4_K_M is a common quantization type that balances size and performance. It typically means 4-bit quantization with some layers in 5-bit or other formats for better quality.

### NVIDIA RTX A4000 GPU

*   **VRAM:** 16 GB GDDR6
*   **Architecture:** Ampere
*   **CUDA Cores:** 6144
*   **Tensor Cores:** 192 (3rd generation)
*   **Memory Bandwidth:** (Need to find this for a complete picture, but 16GB GDDR6 is a good indicator)

### Initial Analysis of Constraints:

*   **VRAM vs. Model Size:** A 14B parameter model, even when quantized to Q4_K_M, can be quite large. The 16GB VRAM of the A4000 is a critical constraint. We need to estimate if the entire model can fit into VRAM. A rough estimate for Q4_K_M is around 4-5 bits per parameter. For 14 billion parameters, this would be: 
    14,000,000,000 parameters * 4.5 bits/parameter = 63,000,000,000 bits = 7.875 GB. 
    This suggests the model *should* fit into 16GB VRAM, leaving room for the KV cache and other overhead. However, the KV cache size depends on the context window (`n_ctx`) and batch size (`n_batch`).
*   **Token Generation Speed:** The reported 2-3 tokens/second is very low for an A4000 with a 14B model, especially if the model fits entirely in VRAM. This indicates that either the model is not fully offloaded to the GPU, or other parameters are suboptimal.

**Next Steps:**

1.  **Confirm VRAM usage:** Research how to accurately estimate VRAM usage for GGUF models in `llama.cpp`, considering `n_ctx` and `n_batch`.
2.  **Identify key `llama-server` parameters for GPU optimization:** Focus on parameters related to GPU offloading, batching, and context management.
3.  **Investigate `llama.cpp` build options:** Ensure `llama.cpp` is built with CUDA support and other relevant optimizations for NVIDIA GPUs.




## GPU Optimization Techniques for llama.cpp

Optimizing `llama.cpp` for NVIDIA GPUs involves several key aspects, from build configurations to runtime parameters.

### Build-time Optimizations

To ensure `llama.cpp` can fully leverage your NVIDIA GPU, it's crucial to build it with CUDA support. This typically involves:

*   **CUDA Support:** Compiling `llama.cpp` with `LLAMA_CUBLAS=1` (or `LLAMA_CUDA=1` in newer versions) enables the use of NVIDIA's cuBLAS library for GPU-accelerated matrix multiplications, which are a core component of LLM inference. This is a fundamental requirement for good GPU performance.
*   **CUDA Graphs:** Recent versions of `llama.cpp` have integrated CUDA Graphs, which can significantly improve performance by reducing CPU overhead in launching GPU kernels. This is often enabled by default when building with CUDA support, but it's worth verifying the build process to ensure it's included.

### Runtime Parameters for GPU Optimization

Beyond `n_gpu_layers`, several other parameters can influence GPU performance:

*   **`--n-gpu-layers` (`n_gpu_layers` in `llama-cpp-python`):** As discussed, this is the most critical parameter for GPU offloading. Setting it to `-1` attempts to offload all model layers to the GPU. Given the A4000's 16GB VRAM and the Phi-4 Q4_K_M model (estimated ~7.875 GB), it should be possible to offload the entire model. If VRAM is insufficient, `llama.cpp` will automatically fall back to CPU for some layers, but this will significantly reduce performance.
*   **`--n-batch` (`n_batch` in `llama-cpp-python`):** This parameter controls the batch size for prompt processing. A larger batch size can improve throughput, especially for multiple concurrent requests or longer prompts. However, it also increases VRAM usage. Experimentation is needed to find the optimal value that maximizes throughput without exceeding VRAM capacity. Common values range from 512 to 2048, but for a 16GB card with a 14B model, a smaller `n_batch` might be necessary if `n_ctx` is large.
*   **`--n-ctx` (`n_ctx` in `llama-cpp-python`):** The context window size. While not directly a GPU optimization, a larger context window requires more VRAM for the KV cache. If `n_ctx` is too large, it can lead to VRAM overflow, forcing layers to CPU or causing out-of-memory errors. The Phi-4 model card mentions a 16K token context window, which is substantial. It's important to balance this with available VRAM.
*   **`--main-gpu` (`main_gpu` in `llama-cpp-python`):** Specifies the main GPU to use in a multi-GPU setup. For a single A4000, this would typically be `0` (default).
*   **`--tensor-split` (`tensor_split` in `llama-cpp-python`):** Used for splitting layers across multiple GPUs. Not directly relevant for a single GPU setup, but good to be aware of for future scaling.
*   **`--no-mmap` / `--mmap` (`use_mmap` in `llama-cpp-python`):** Memory mapping. `mmap` can be faster for loading models, but `no-mmap` might be necessary in some cases, especially with limited RAM or specific file system configurations. Generally, `mmap` is preferred.
*   **`--mlock` (`use_mlock` in `llama-cpp-python`):** Locks the model in RAM, preventing it from being swapped to disk. This can improve performance by ensuring the model is always in physical memory, but requires sufficient RAM. Given 62GB RAM, this could be beneficial.
*   **`--n-threads`:** While the primary bottleneck will be the GPU, ensuring sufficient CPU threads are allocated for pre- and post-processing can prevent CPU from becoming a bottleneck. This is typically set to the number of physical CPU cores or slightly higher. The user has 16 vCPUs, so a value around 8-16 could be a good starting point.

### Other Considerations

*   **Quantization:** The user is already using Q4_K_M, which is a good balance between model size and quality. Further quantization (e.g., Q2_K) could reduce VRAM usage but might impact model quality.
*   **Driver Updates:** Ensure the latest NVIDIA GPU drivers are installed for optimal performance and compatibility with CUDA.
*   **System Configuration:** Minimize background processes and ensure sufficient system RAM is available, even when offloading to GPU, as some operations still utilize CPU RAM.
*   **Monitoring:** Use tools like `nvidia-smi` to monitor GPU utilization, VRAM usage, and temperature during inference to identify bottlenecks and fine-tune parameters. This will be crucial for validating the optimized configuration.



## Optimized Configuration Recommendations for llama-server

Based on the analysis of the Microsoft Phi-4 model, NVIDIA RTX A4000 GPU, and `llama.cpp` optimization techniques, here are the recommended configurations for `llama-server` to maximize token generation speed.

### Key Parameters and Recommended Values:

1.  **`--model <path-to-gguf>`**: Use the provided `phi-4-Q4_K_M.gguf` model file.
    *   **Reasoning:** This is the specified model and quantization. Q4_K_M offers a good balance of size and quality.

2.  **`--n-gpu-layers -1`**: Offload all layers to the GPU.
    *   **Reasoning:** With 16GB of VRAM on the RTX A4000 and an estimated model size of ~7.875 GB for the Q4_K_M quantization of the 14B Phi-4 model, the entire model should fit comfortably in VRAM. Offloading all layers to the GPU (`-1`) is crucial for maximizing GPU utilization and minimizing CPU fallback, which is a major performance bottleneck.

3.  **`--n-ctx <context_size>`**: Set an appropriate context window size.
    *   **Recommendation:** Start with `--n-ctx 4096` or `--n-ctx 8192`. While the model card mentions 16K, a larger context window consumes more VRAM for the KV cache. Given the 16GB VRAM, a 4K or 8K context should be manageable alongside the model. If you experience out-of-memory errors or significant performance degradation, reduce this value. If performance is good, you can gradually increase it to find the maximum stable context size.
    *   **Reasoning:** The context window directly impacts the KV cache size. Balancing this with available VRAM is essential. A smaller context window will use less VRAM, potentially allowing for larger batch sizes or leaving more room for other processes.

4.  **`--n-batch <batch_size>`**: Optimize the batch size for prompt processing.
    *   **Recommendation:** Start with `--n-batch 512`. This parameter affects throughput. Larger batch sizes can improve performance for concurrent requests or longer prompts, but increase VRAM usage. Experiment with values like 256, 512, 1024 to find the sweet spot for your specific workload and `n_ctx` setting. If you are primarily doing single-user, short-prompt inference, a smaller batch size might be sufficient.
    *   **Reasoning:** Batching allows the GPU to process multiple tokens or prompts in parallel. Finding the optimal batch size is an empirical process that depends on the model, GPU, and other parameters.

5.  **`--mlock`**: Lock the model in RAM.
    *   **Recommendation:** Include `--mlock` in your arguments.
    *   **Reasoning:** This prevents the model from being swapped out of RAM to disk, ensuring it's always in fast physical memory. With 62GB of RAM, this should not be an issue and can contribute to more consistent performance.

6.  **`--n-threads <number_of_threads>`**: Set the number of CPU threads.
    *   **Recommendation:** Set `--n-threads` to a value between 8 and 16, corresponding to your 16 vCPUs. You can start with `--n-threads 12` or `--n-threads 16`.
    *   **Reasoning:** Even with GPU offloading, some pre- and post-processing tasks occur on the CPU. Sufficient CPU threads prevent the CPU from becoming a bottleneck. Experimentation might be needed to find the optimal value, as too many threads can sometimes lead to overhead.

7.  **`--api-key <random-uuid>`**: Keep this as you are already using it for security.

8.  **`--port <random-port>`**: Keep this as you are already using it.

9.  **`--jinja`**: Keep this if you are using Jinja templating.

### Example Optimized `llama-server` Command:

```bash
./llama-server \
  --alias phi-4-server \
  --model <path-to-your-phi-4-Q4_K_M.gguf> \
  --n-gpu-layers -1 \
  --n-ctx 8192 \
  --n-batch 512 \
  --mlock \
  --n-threads 12 \
  --jinja \
  --api-key <your-random-uuid> \
  --port <your-random-port>
```

### Additional Optimization Tips:

*   **CUDA Build:** Ensure your `llama.cpp` executable is built with CUDA support. If you compiled it yourself, verify that `LLAMA_CUBLAS=1` (or `LLAMA_CUDA=1`) was used during compilation. If you are using pre-built binaries, ensure they are CUDA-enabled.
*   **NVIDIA Drivers:** Always keep your NVIDIA GPU drivers updated to the latest stable version. This is crucial for performance and compatibility.
*   **Monitor Performance:** Use `nvidia-smi` to monitor your GPU during inference. Pay attention to:
    *   **`Memory Usage`**: Confirm that the VRAM usage is high, indicating the model is fully loaded onto the GPU. If it's low, it suggests layers are not being offloaded.
    *   **`GPU-Util`**: High GPU utilization (e.g., >90%) during token generation indicates the GPU is actively working. If it's low, there might be a bottleneck elsewhere (e.g., CPU, I/O).
    *   **`Power Draw / Temp`**: Monitor these to ensure the GPU is operating within safe limits, especially during extended inference sessions.
*   **Quantization Experimentation:** While Q4_K_M is a good starting point, if you still face VRAM constraints or want to push for higher token generation, you could experiment with even lower quantizations (e.g., Q3_K_M or Q2_K) if available for your model, but be aware of potential quality degradation.
*   **System Resources:** Ensure no other demanding applications are running in the background that might compete for GPU or CPU resources.

By carefully applying these recommendations and monitoring your system, you should be able to significantly improve the token generation speed of your `llama-server` with the Phi-4 model on your RTX A4000 GPU.

