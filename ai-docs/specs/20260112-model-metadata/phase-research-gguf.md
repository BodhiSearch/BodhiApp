# GGUF Metadata Specification Research

## Overview

Research on GGUF format metadata keys, architecture patterns, and capability detection for multimodal models. Based on llama.cpp source files and real-world GGUF model examples.

**Reference Sources:**
- [llama.cpp constants.py](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/constants.py)
- [llama.cpp convert_hf_to_gguf.py](https://github.com/ggml-org/llama.cpp/blob/master/convert_hf_to_gguf.py)
- [Multimodal GGUFs Collection](https://huggingface.co/collections/ggml-org/multimodal-ggufs-68244e01ff1f39e5bebeeedc)

## MODEL_ARCH Enum - Complete Architecture List

From `gguf-py/gguf/constants.py`, the complete list of supported architectures:

| Enum | String Identifier | Description |
|------|------------------|-------------|
| MMPROJ | `clip` | Dummy architecture for clip.cpp (vision/audio encoders) |
| LLAMA | `llama` | LLaMA family |
| LLAMA4 | `llama4` | LLaMA 4 family |
| LLAMA_EMBED | `llama-embed` | LLaMA embedding models |
| DECI | `deci` | DeciLM |
| FALCON | `falcon` | Falcon |
| FALCON_H1 | `falcon-h1` | Falcon H1 |
| BAICHUAN | `baichuan` | Baichuan |
| GROK | `grok` | Grok (xAI) |
| GPT2 | `gpt2` | GPT-2 |
| GPTJ | `gptj` | GPT-J |
| GPTNEOX | `gptneox` | GPT-NeoX |
| MPT | `mpt` | MosaicML MPT |
| STARCODER | `starcoder` | StarCoder |
| STARCODER2 | `starcoder2` | StarCoder2 |
| REFACT | `refact` | Refact |
| BERT | `bert` | BERT |
| MODERN_BERT | `modern-bert` | Modern BERT |
| NOMIC_BERT | `nomic-bert` | Nomic BERT |
| NOMIC_BERT_MOE | `nomic-bert-moe` | Nomic BERT MoE |
| NEO_BERT | `neo-bert` | Neo BERT |
| JINA_BERT_V2 | `jina-bert-v2` | Jina BERT v2 |
| JINA_BERT_V3 | `jina-bert-v3` | Jina BERT v3 |
| BLOOM | `bloom` | BLOOM |
| STABLELM | `stablelm` | StableLM |
| QWEN | `qwen` | Qwen |
| QWEN2 | `qwen2` | Qwen2 |
| QWEN2MOE | `qwen2moe` | Qwen2 MoE |
| QWEN2VL | `qwen2vl` | Qwen2-VL (vision-language) |
| QWEN3 | `qwen3` | Qwen3 |
| QWEN3MOE | `qwen3moe` | Qwen3 MoE |
| QWEN3NEXT | `qwen3next` | Qwen3 Next |
| QWEN3VL | `qwen3vl` | Qwen3-VL (vision-language) |
| QWEN3VLMOE | `qwen3vlmoe` | Qwen3-VL MoE |
| PHI2 | `phi2` | Phi-2 |
| PHI3 | `phi3` | Phi-3 |
| PHIMOE | `phimoe` | Phi MoE |
| PLAMO | `plamo` | PLaMo |
| PLAMO2 | `plamo2` | PLaMo-2 |
| PLAMO3 | `plamo3` | PLaMo-3 |
| CODESHELL | `codeshell` | CodeShell |
| ORION | `orion` | Orion |
| INTERNLM2 | `internlm2` | InternLM2 |
| MINICPM | `minicpm` | MiniCPM |
| MINICPM3 | `minicpm3` | MiniCPM3 |
| GEMMA | `gemma` | Gemma |
| GEMMA2 | `gemma2` | Gemma 2 |
| GEMMA3 | `gemma3` | Gemma 3 |
| GEMMA3N | `gemma3n` | Gemma 3N |
| GEMMA_EMBEDDING | `gemma-embedding` | Gemma Embedding |
| RWKV6 | `rwkv6` | RWKV v6 |
| RWKV6QWEN2 | `rwkv6qwen2` | RWKV v6 Qwen2 |
| RWKV7 | `rwkv7` | RWKV v7 |
| ARWKV7 | `arwkv7` | AR-RWKV v7 |
| MAMBA | `mamba` | Mamba |
| MAMBA2 | `mamba2` | Mamba2 |
| JAMBA | `jamba` | Jamba |
| XVERSE | `xverse` | XVERSE |
| COMMAND_R | `command-r` | Command-R |
| COHERE2 | `cohere2` | Cohere 2 |
| DBRX | `dbrx` | DBRX |
| OLMO | `olmo` | OLMo |
| OLMO2 | `olmo2` | OLMo 2 |
| OLMOE | `olmoe` | OLMoE |
| OPENELM | `openelm` | OpenELM |
| ARCTIC | `arctic` | Arctic |
| DEEPSEEK | `deepseek` | DeepSeek |
| DEEPSEEK2 | `deepseek2` | DeepSeek-V2 |
| CHATGLM | `chatglm` | ChatGLM |
| GLM4 | `glm4` | GLM-4 |
| GLM4_MOE | `glm4moe` | GLM-4 MoE |
| BITNET | `bitnet` | BitNet |
| T5 | `t5` | T5 |
| T5ENCODER | `t5encoder` | T5 Encoder |
| JAIS | `jais` | JAIS |
| NEMOTRON | `nemotron` | Nemotron |
| NEMOTRON_H | `nemotron_h` | Nemotron H |
| NEMOTRON_H_MOE | `nemotron_h_moe` | Nemotron H MoE |
| EXAONE | `exaone` | EXAONE |
| EXAONE4 | `exaone4` | EXAONE 4 |
| GRANITE | `granite` | Granite |
| GRANITE_MOE | `granitemoe` | Granite MoE |
| GRANITE_HYBRID | `granitehybrid` | Granite Hybrid |
| CHAMELEON | `chameleon` | Chameleon (multimodal) |
| WAVTOKENIZER_DEC | `wavtokenizer-dec` | WavTokenizer Decoder |
| PLM | `plm` | PLM |
| BAILINGMOE | `bailingmoe` | BaiLingMoE |
| BAILINGMOE2 | `bailingmoe2` | BaiLingMoE 2 |
| DOTS1 | `dots1` | DoTS-1 |
| ARCEE | `arcee` | Arcee |
| AFMOE | `afmoe` | AFMoE |
| ERNIE4_5 | `ernie4_5` | ERNIE 4.5 |
| ERNIE4_5_MOE | `ernie4_5-moe` | ERNIE 4.5 MoE |
| HUNYUAN_MOE | `hunyuan-moe` | Hunyuan MoE |
| HUNYUAN_DENSE | `hunyuan-dense` | Hunyuan Dense |
| SMOLLM3 | `smollm3` | SmolLM3 |
| GPT_OSS | `gpt-oss` | GPT OSS |
| LFM2 | `lfm2` | LFM2 |
| LFM2MOE | `lfm2moe` | LFM2 MoE |
| DREAM | `dream` | Dream |
| SMALLTHINKER | `smallthinker` | SmallThinker |
| LLADA | `llada` | LLaDA |
| LLADA_MOE | `llada-moe` | LLaDA MoE |
| SEED_OSS | `seed_oss` | SEED OSS |
| GROVEMOE | `grovemoe` | GroveMoE |
| APERTUS | `apertus` | Apertus |
| MINIMAXM2 | `minimax-m2` | MiniMax M2 |
| COGVLM | `cogvlm` | CogVLM (vision-language) |
| RND1 | `rnd1` | RND-1 |
| PANGU_EMBED | `pangu-embedded` | Pangu Embedded |
| MISTRAL3 | `mistral3` | Mistral 3 |
| MIMO2 | `mimo2` | MiMo-2 |
| MAINCODER | `maincoder` | MainCoder |

**Total Architectures:** 119+ architectures supported as of January 2026

## CLIP/Multimodal Metadata Keys

### General CLIP Architecture Keys

From `Keys.Clip` class in constants.py:

| Key | Type | Description |
|-----|------|-------------|
| `clip.projector_type` | string | Type of projector (mlp, ldp, ldpv2, resampler, glm_edge, merger, gemma3n, gemma3, qwen3vl, cogvlm, pixtral) |
| `clip.has_vision_encoder` | bool | Model includes vision encoder |
| `clip.has_audio_encoder` | bool | Model includes audio encoder |
| `clip.has_llava_projector` | bool | Model uses LLaVA-style projector |

### Vision Encoder Metadata Keys

From `Keys.ClipVision` class:

| Key | Type | Description |
|-----|------|-------------|
| `clip.vision.projector_type` | string | Projector type for mixed modality models |
| `clip.vision.image_size` | u32 | Input image size (e.g., 448) |
| `clip.vision.preproc_image_size` | u32 | Preprocessing image size |
| `clip.vision.patch_size` | u32 | Vision patch size (e.g., 14) |
| `clip.vision.embedding_length` | u32 | Vision embedding dimension |
| `clip.vision.feed_forward_length` | u32 | Feed-forward layer dimension |
| `clip.vision.projection_dim` | u32 | Projection output dimension |
| `clip.vision.block_count` | u32 | Number of vision transformer blocks |
| `clip.vision.image_mean` | arr[f32,3] | Image normalization mean values |
| `clip.vision.image_std` | arr[f32,3] | Image normalization std values |
| `clip.vision.spatial_merge_size` | u32 | Spatial pooling/merge size |
| `clip.use_gelu` | bool | Use GELU activation |
| `clip.use_silu` | bool | Use SiLU activation |
| `clip.vision.n_wa_pattern` | u32 | Window attention pattern (Qwen2.5-VL) |
| `clip.vision.wa_layer_indexes` | arr | Window attention layer indexes (YouTuVL) |
| `clip.vision.is_deepstack_layers` | bool | Uses deepstack layers |
| `clip.vision.window_size` | u32 | Attention window size |

#### Vision Attention Sub-keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.vision.attention.head_count` | u32 | Number of attention heads |
| `clip.vision.attention.layer_norm_epsilon` | f32 | LayerNorm epsilon |

#### Vision Projector Sub-keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.vision.projector.scale_factor` | u32 | Projector scaling factor |

### Audio Encoder Metadata Keys

From `Keys.ClipAudio` class:

| Key | Type | Description |
|-----|------|-------------|
| `clip.audio.projector_type` | string | Projector type for mixed modality |
| `clip.audio.num_mel_bins` | u32 | Number of mel spectrogram bins |
| `clip.audio.embedding_length` | u32 | Audio embedding dimension |
| `clip.audio.feed_forward_length` | u32 | Feed-forward layer dimension |
| `clip.audio.projection_dim` | u32 | Projection output dimension |
| `clip.audio.block_count` | u32 | Number of audio transformer blocks |

#### Audio Attention Sub-keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.audio.attention.head_count` | u32 | Number of attention heads |
| `clip.audio.attention.layer_norm_epsilon` | f32 | LayerNorm epsilon |

#### Audio Projector Sub-keys

| Key | Type | Description |
|-----|------|-------------|
| `clip.audio.projector.stack_factor` | u32 | Audio frame stacking factor |

### Vision Projector Types

From `VISION_PROJECTOR_TYPE` enum:

| Enum | Description |
|------|-------------|
| MLP | Multi-layer perceptron projector (LLaVA-style) |
| LDP | Linear-Depth Pooling |
| LDPV2 | Linear-Depth Pooling v2 |
| RESAMPLER | Perceiver Resampler (MiniCPM-V) |
| GLM_EDGE | GLM Edge projector |
| MERGER | Merging projector |
| GEMMA3N | Gemma 3N projector |
| GEMMA3 | Gemma 3 projector |
| QWEN3VL | Qwen3-VL projector |
| COGVLM | CogVLM projector |

## Architecture-Specific Context Length Keys

Pattern: `{arch}.context_length` where `{arch}` is the architecture identifier.

### Complete Architecture → Context Key Mapping

| Architecture | Context Key | Example Value |
|--------------|------------|---------------|
| llama | `llama.context_length` | 4096 |
| llama4 | `llama4.context_length` | 8192 |
| llama-embed | `llama-embed.context_length` | 4096 |
| deci | `deci.context_length` | 4096 |
| falcon | `falcon.context_length` | 2048 |
| falcon-h1 | `falcon-h1.context_length` | 4096 |
| baichuan | `baichuan.context_length` | 4096 |
| grok | `grok.context_length` | 8192 |
| gpt2 | `gpt2.context_length` | 1024 |
| gptj | `gptj.context_length` | 2048 |
| gptneox | `gptneox.context_length` | 2048 |
| mpt | `mpt.context_length` | 2048 |
| starcoder | `starcoder.context_length` | 8192 |
| starcoder2 | `starcoder2.context_length` | 16384 |
| refact | `refact.context_length` | 16384 |
| bert | `bert.context_length` | 512 |
| modern-bert | `modern-bert.context_length` | 512 |
| nomic-bert | `nomic-bert.context_length` | 8192 |
| nomic-bert-moe | `nomic-bert-moe.context_length` | 8192 |
| neo-bert | `neo-bert.context_length` | 512 |
| jina-bert-v2 | `jina-bert-v2.context_length` | 8192 |
| jina-bert-v3 | `jina-bert-v3.context_length` | 8192 |
| bloom | `bloom.context_length` | 2048 |
| stablelm | `stablelm.context_length` | 4096 |
| qwen | `qwen.context_length` | 8192 |
| qwen2 | `qwen2.context_length` | 32768 |
| qwen2moe | `qwen2moe.context_length` | 32768 |
| qwen2vl | `qwen2vl.context_length` | 32768 |
| qwen3 | `qwen3.context_length` | 32768 |
| qwen3moe | `qwen3moe.context_length` | 32768 |
| qwen3next | `qwen3next.context_length` | 32768 |
| qwen3vl | `qwen3vl.context_length` | 32768 |
| qwen3vlmoe | `qwen3vlmoe.context_length` | 32768 |
| phi2 | `phi2.context_length` | 2048 |
| phi3 | `phi3.context_length` | 4096 |
| phimoe | `phimoe.context_length` | 4096 |
| plamo | `plamo.context_length` | 4096 |
| plamo2 | `plamo2.context_length` | 4096 |
| plamo3 | `plamo3.context_length` | 4096 |
| codeshell | `codeshell.context_length` | 8192 |
| orion | `orion.context_length` | 4096 |
| internlm2 | `internlm2.context_length` | 32768 |
| minicpm | `minicpm.context_length` | 4096 |
| minicpm3 | `minicpm3.context_length` | 4096 |
| gemma | `gemma.context_length` | 8192 |
| gemma2 | `gemma2.context_length` | 8192 |
| gemma3 | `gemma3.context_length` | 8192 |
| gemma3n | `gemma3n.context_length` | 8192 |
| gemma-embedding | `gemma-embedding.context_length` | 8192 |
| rwkv6 | `rwkv6.context_length` | - (stateful) |
| rwkv6qwen2 | `rwkv6qwen2.context_length` | - (stateful) |
| rwkv7 | `rwkv7.context_length` | - (stateful) |
| arwkv7 | `arwkv7.context_length` | - (stateful) |
| mamba | `mamba.context_length` | - (stateful) |
| mamba2 | `mamba2.context_length` | - (stateful) |
| jamba | `jamba.context_length` | 256000 |
| xverse | `xverse.context_length` | 8192 |
| command-r | `command-r.context_length` | 131072 |
| cohere2 | `cohere2.context_length` | 131072 |
| dbrx | `dbrx.context_length` | 32768 |
| olmo | `olmo.context_length` | 2048 |
| olmo2 | `olmo2.context_length` | 4096 |
| olmoe | `olmoe.context_length` | 131072 |
| openelm | `openelm.context_length` | 2048 |
| arctic | `arctic.context_length` | 4096 |
| deepseek | `deepseek.context_length` | 4096 |
| deepseek2 | `deepseek2.context_length` | 163840 |
| chatglm | `chatglm.context_length` | 32768 |
| glm4 | `glm4.context_length` | 131072 |
| glm4moe | `glm4moe.context_length` | 131072 |
| bitnet | `bitnet.context_length` | 2048 |
| t5 | `t5.context_length` | 512 |
| t5encoder | `t5encoder.context_length` | 512 |
| jais | `jais.context_length` | 2048 |
| nemotron | `nemotron.context_length` | 4096 |
| nemotron_h | `nemotron_h.context_length` | 4096 |
| nemotron_h_moe | `nemotron_h_moe.context_length` | 4096 |
| exaone | `exaone.context_length` | 2048 |
| exaone4 | `exaone4.context_length` | 4096 |
| granite | `granite.context_length` | 4096 |
| granitemoe | `granitemoe.context_length` | 4096 |
| granitehybrid | `granitehybrid.context_length` | 4096 |
| chameleon | `chameleon.context_length` | 4096 |
| wavtokenizer-dec | `wavtokenizer-dec.context_length` | - |
| plm | `plm.context_length` | 4096 |
| bailingmoe | `bailingmoe.context_length` | 4096 |
| bailingmoe2 | `bailingmoe2.context_length` | 4096 |
| dots1 | `dots1.context_length` | 4096 |
| arcee | `arcee.context_length` | 4096 |
| afmoe | `afmoe.context_length` | 4096 |
| ernie4_5 | `ernie4_5.context_length` | 131072 |
| ernie4_5-moe | `ernie4_5-moe.context_length` | 131072 |
| hunyuan-moe | `hunyuan-moe.context_length` | 256000 |
| hunyuan-dense | `hunyuan-dense.context_length` | 256000 |
| smollm3 | `smollm3.context_length` | 8192 |
| gpt-oss | `gpt-oss.context_length` | varies |
| lfm2 | `lfm2.context_length` | 16384 |
| lfm2moe | `lfm2moe.context_length` | 16384 |
| dream | `dream.context_length` | 4096 |
| smallthinker | `smallthinker.context_length` | 4096 |
| llada | `llada.context_length` | 4096 |
| llada-moe | `llada-moe.context_length` | 4096 |
| seed_oss | `seed_oss.context_length` | varies |
| grovemoe | `grovemoe.context_length` | 4096 |
| apertus | `apertus.context_length` | 4096 |
| minimax-m2 | `minimax-m2.context_length` | 4096 |
| cogvlm | `cogvlm.context_length` | 8192 |
| rnd1 | `rnd1.context_length` | varies |
| pangu-embedded | `pangu-embedded.context_length` | 2048 |
| mistral3 | `mistral3.context_length` | 131072 |
| mimo2 | `mimo2.context_length` | 4096 |
| maincoder | `maincoder.context_length` | 4096 |

**Note:** Values marked with "varies" depend on specific model configuration. Stateful models (RWKV, Mamba) don't use traditional context windows.

## Real-World Multimodal GGUF Examples

### Example 1: MiniCPM-V-4 GGUF (Vision Model)

**Source:** [MiniCPM-V GGUF Issue #957](https://github.com/OpenBMB/MiniCPM-V/issues/957)

Language model metadata:
```
general.architecture = minicpm
minicpm.context_length = 32768
```

Vision encoder metadata (mmproj file):
```
n_tensors: 455
n_kv: 19

clip_model_loader:
  general.architecture = clip
  clip.has_text_encoder = false
  clip.has_vision_encoder = true
  clip.has_llava_projector = false
  clip.has_minicpmv_projector = true
  clip.minicpmv_version = 5
  clip.projector_type = resampler
  clip.vision.image_size = 448
  clip.vision.patch_size = 14
  general.file_type = u32 1
  general.description = "image encoder for MiniCPM-V"
```

### Example 2: LLaVA-Compatible Vision Model

**Source:** [llama.cpp Issue #11249](https://github.com/ggml-org/llama.cpp/issues/11249)

Vision encoder metadata:
```
general.architecture = clip
clip.has_text_encoder = false
clip.has_vision_encoder = true
clip.has_llava_projector = true
clip.projector_type = mlp
clip.vision.image_size = u32 448
clip.vision.patch_size = u32 14
clip.vision.embedding_length = u32 1024
clip.vision.projection_dim = u32 4096
clip.vision.image_mean = arr[f32,3]
clip.vision.image_std = arr[f32,3]
general.description = "image encoder for LLaVA"
```

### Example 3: Qwen2.5-Omni-7B GGUF (Any-to-Any Multimodal)

**Source:** [Qwen2.5-Omni-7B-GGUF on HuggingFace](https://huggingface.co/ggml-org/Qwen2.5-Omni-7B-GGUF)

Capabilities:
- Text input ✅
- Audio input ✅
- Image input ✅
- Audio generation ❌

Metadata:
```
general.architecture = qwen2vl
qwen2vl.context_length = 131072
```

Compatible with llama-server and llama-mtmd-cli for multimodal inference.

### Example 4: Pixtral-12B GGUF (Vision Model)

**Source:** [pixtral-12b-GGUF on HuggingFace](https://huggingface.co/ggml-org/pixtral-12b-GGUF)

Metadata:
```
general.architecture = llama
llama.context_length = 4096
clip.projector_type = pixtral
clip.vision.attention.layer_norm_epsilon = (from hparams)
```

Available in multiple quantizations: Q2_K (4.79 GB), Q4_K_M (7.48 GB), Q8_0 (13 GB), F16 (24.5 GB)

## Multimodal Model Collections

### ggml-org Multimodal GGUFs Collection

**Source:** [Multimodal GGUFs Collection](https://huggingface.co/collections/ggml-org/multimodal-ggufs-68244e01ff1f39e5bebeeedc)

#### Vision Models (Image-Text-to-Text)
1. **Mistral-Small-3.1-24B-Instruct-2503-GGUF** (24B, 426 downloads)
2. **moondream2-20250414-GGUF** (1B, 1.87k downloads)
3. **pixtral-12b-GGUF** (12B, 1.27k downloads)

#### Audio Models (Audio-Text-to-Text)
4. **ultravox-v0_5-llama-3_2-1b-GGUF** (1B, 823 downloads)
5. **ultravox-v0_5-llama-3_1-8b-GGUF** (8B, 252 downloads)

#### Vision + Audio Models (Any-to-Any)
6. **Qwen2.5-Omni-3B-GGUF** (3B, 1.31k downloads)
7. **Qwen2.5-Omni-7B-GGUF** (8B, 831 downloads)
8. **Voxtral-Mini-3B-2507-GGUF** (4B, 386 downloads)
9. **LFM2-Audio-1.5B-GGUF** (1B, 375 downloads)

All models compatible with **llama-server** and **llama-mtmd-cli**.

### NexaAI Multimodal GGUF Collection

**Source:** [Multimodal - GGUF Collection](https://huggingface.co/collections/NexaAI/multimodal-gguf)

Alternative collection with additional multimodal GGUF models.

### LM Studio Vision Models Collection

**Source:** [Vision Models (GGUF)](https://huggingface.co/collections/lmstudio-ai/vision-models-gguf-6577e1ce821f439498ced0c1)

Curated collection of vision-capable GGUF models for LM Studio.

## Conversion Patterns from HuggingFace to GGUF

From `convert_hf_to_gguf.py`:

### Vision Encoder Handling

```python
has_vision_encoder: bool = True  # by default
has_audio_encoder: bool = False

# Extract config
def get_vision_config(self) -> dict[str, Any] | None:
    config_name = "vision_config" if not self.is_mistral_format else "vision_encoder"
    return self.global_config.get(config_name)

# Set metadata
if self.has_vision_encoder:
    self.gguf_writer.add_clip_has_vision_encoder(True)
    self.gguf_writer.add_vision_projection_dim(self.n_embd_text)
    # ... vision config metadata
    self.gguf_writer.add_vision_image_mean(image_mean)
    self.gguf_writer.add_vision_image_std(image_std)
```

### Audio Encoder Handling

```python
def get_audio_config(self) -> dict[str, Any] | None:
    # Extract from config

if self.has_audio_encoder:
    self.gguf_writer.add_clip_has_audio_encoder(True)
    self.gguf_writer.add_audio_projection_dim(self.n_embd_text)
    # ... audio config metadata
    self.gguf_writer.add_audio_block_count(self.find_aparam(self.n_block_keys))
    self.gguf_writer.add_audio_head_count(self.find_aparam(["num_attention_heads"]))
```

### Projector Type Detection

```python
# Example: Pixtral projector
if hparams.get("model_type") == "pixtral":
    self.gguf_writer.add_clip_projector_type(gguf.VisionProjectorType.PIXTRAL)
    self.gguf_writer.add_vision_attention_layernorm_eps(hparams["layer_norm_eps"])
```

### Multimodal Tensor Detection

Vision/audio tensors identified by prefixes:
```python
vision_prefixes = [
    "vision_encoder.",
    "vision_language_adapter.",
    "patch_merger.",
    "pre_mm_projector_norm",
    "audio_encoder.",
]

is_multimodal_tensor = "vision_tower" in name \
    or "vision_model" in name \
    or "audio_tower" in name \
    or "model.connector" in name \
    or "multi_modal_projector" in name \
    or any(name.startswith(prefix) for prefix in vision_prefixes)
```

## Capability Detection Strategy

### For Vision Capabilities

Check in order:
1. **Architecture-specific vision models**: `qwen2vl`, `qwen3vl`, `qwen3vlmoe`, `chameleon`, `cogvlm`
2. **MMPROJ architecture**: `general.architecture == "clip"` with `clip.has_vision_encoder == true`
3. **Projector type presence**: Non-null `clip.projector_type` or `clip.vision.projector_type`
4. **Vision-specific metadata**: Presence of `clip.vision.image_size` or `clip.vision.patch_size`

### For Audio Capabilities

Check in order:
1. **MMPROJ architecture**: `general.architecture == "clip"` with `clip.has_audio_encoder == true`
2. **Audio-specific metadata**: Presence of `clip.audio.num_mel_bins` or `clip.audio.embedding_length`
3. **Architecture hints**: `lfm2`, `lfm2moe` architectures

### For Mixed Modality (Vision + Audio)

Check:
1. **Both flags present**: `clip.has_vision_encoder == true` AND `clip.has_audio_encoder == true`
2. **Qwen Omni models**: `qwen2vl` architecture with audio metadata
3. **Separate projector types**: Both `clip.vision.projector_type` and `clip.audio.projector_type` present

## Key Implementation Insights

1. **Architecture determines context key**: Always use `{arch}.context_length` pattern
2. **MMPROJ is dummy arch**: `clip` architecture is special - used for vision/audio encoders only
3. **Separate files common**: Many models split language model and vision encoder into separate GGUF files
4. **Projector types matter**: Different projector architectures (MLP, Resampler, etc.) affect inference
5. **Boolean flags are critical**: `has_vision_encoder`, `has_audio_encoder`, `has_llava_projector` determine capabilities
6. **MiniCPM uses custom projector**: `has_minicpmv_projector` instead of `has_llava_projector`
7. **Image preprocessing varies**: `image_mean` and `image_std` are model-specific and required for correct inference

## Sources

- [llama.cpp constants.py](https://github.com/ggml-org/llama.cpp/blob/master/gguf-py/gguf/constants.py) - Complete metadata key definitions
- [llama.cpp convert_hf_to_gguf.py](https://github.com/ggml-org/llama.cpp/blob/master/convert_hf_to_gguf.py) - Conversion patterns
- [Multimodal GGUFs Collection](https://huggingface.co/collections/ggml-org/multimodal-ggufs-68244e01ff1f39e5bebeeedc) - Official multimodal models
- [NexaAI Multimodal Collection](https://huggingface.co/collections/NexaAI/multimodal-gguf) - Additional examples
- [Vision Models (GGUF) - LM Studio](https://huggingface.co/collections/lmstudio-ai/vision-models-gguf-6577e1ce821f439498ced0c1) - Curated vision models
- [GGUF Metadata Guide](https://opus4i.com/gguf) - GGUF vs HF metadata comparison
- [Understanding GGUF Format](https://medium.com/@vimalkansal/understanding-the-gguf-format-a-comprehensive-guide-67de48848256) - Comprehensive guide
- [GGUF File Format Docs](https://deepwiki.com/ggml-org/llama.cpp/6.1-gguf-file-format) - Technical documentation
- [Vision Language Models 2025](https://huggingface.co/blog/vlms-2025) - Recent VLM developments
- [Qwen2.5-Omni GGUF](https://huggingface.co/ggml-org/Qwen2.5-Omni-7B-GGUF) - Any-to-any multimodal example
- [pixtral-12b-GGUF](https://huggingface.co/ggml-org/pixtral-12b-GGUF) - Vision model example
- [MiniCPM-V GGUF Discussion](https://github.com/OpenBMB/MiniCPM-V/issues/957) - Real metadata examples
- [llama.cpp CLIP Issues](https://github.com/ggml-org/llama.cpp/issues/11249) - Multimodal troubleshooting
