#!/usr/bin/env python3
"""
Generate test GGUF files for E2E Playwright tests with full HuggingFace cache structure.

This script creates 6 GGUF files in a synthetic HF_HOME directory structure:
- llama-plain.gguf - No capabilities (baseline)
- qwen-vision.gguf - Vision only (architecture-based)
- phi-tools.gguf - Tool calling + structured output (template-based)
- deepseek-thinking.gguf - Thinking capability (name-based + template)
- mistral-audio.gguf - Audio capability (metadata-based)
- llava-multimodal.gguf - Vision + tool calling + structured output (multi-capability)

Output directory: ../../lib_bodhiserver_napi/tests-js/data/test-gguf/
Directory structure mimics HuggingFace cache:
  hub/models--test--{model}/
    ├── refs/main (contains snapshot hash)
    ├── blobs/{sha256-hash} (actual GGUF binary)
    └── snapshots/{snapshot-hash}/{filename}.gguf (symlink to blob)
"""

from gguf import GGUFEndian, GGUFWriter
import os
import hashlib
import shutil
from pathlib import Path

# Fixed snapshot hash for all test models
SNAPSHOT_HASH = "test1234567890abcdef1234567890abcdef12345678"

# Chat templates
TOOLS_TEMPLATE = """{% if tools %}
<|start_header_id|>system<|end_header_id|>

You have access to the following tools:
{{ tools }}

Use the following format for tool calls:
<tool_call>
{"name": "function_name", "arguments": {"arg1": "value1"}}
</tool_call>
{% endif %}
{% for message in messages %}
<|start_header_id|>{{ message['role'] }}<|end_header_id|>
{{ message['content'] }}<|eot_id|>
{% endfor %}
{% if add_generation_prompt %}
<|start_header_id|>assistant<|end_header_id|>
{% endif %}"""

THINKING_TEMPLATE = """{% for message in messages %}
<|start_header_id|>{{ message['role'] }}<|end_header_id|>
{% if message['role'] == 'assistant' %}
<think>
Reasoning process here...
</think>
{% endif %}
{{ message['content'] }}<|eot_id|>
{% endfor %}
{% if add_generation_prompt %}
<|start_header_id|>assistant<|end_header_id|>
<think>
{% endif %}"""

BASIC_TEMPLATE = """{% for message in messages %}
<|start_header_id|>{{ message['role'] }}<|end_header_id|>
{{ message['content'] }}<|eot_id|>
{% endfor %}
{% if add_generation_prompt %}
<|start_header_id|>assistant<|end_header_id|>
{% endif %}"""


def compute_file_hash(file_path: str) -> str:
    """Compute SHA256 hash of a file"""
    sha256 = hashlib.sha256()
    with open(file_path, 'rb') as f:
        for chunk in iter(lambda: f.read(4096), b''):
            sha256.update(chunk)
    return sha256.hexdigest()


def create_hf_structure(base_dir: Path, model_name: str, gguf_file: Path):
    """
    Create full HuggingFace cache structure for a model:
    - refs/main containing snapshot hash
    - blobs/{content-hash} with actual file
    - snapshots/{snapshot-hash}/{filename} as symlink to blob
    """
    model_dir = base_dir / f"hub/models--test--{model_name}"
    model_dir.mkdir(parents=True, exist_ok=True)

    # 1. Create refs/main with snapshot hash
    refs_dir = model_dir / "refs"
    refs_dir.mkdir(exist_ok=True)
    with open(refs_dir / "main", 'w') as f:
        f.write(SNAPSHOT_HASH)

    # 2. Compute content hash and create blob
    content_hash = compute_file_hash(gguf_file)
    blobs_dir = model_dir / "blobs"
    blobs_dir.mkdir(exist_ok=True)
    blob_path = blobs_dir / content_hash
    shutil.copy2(gguf_file, blob_path)

    # 3. Create snapshot directory with symlink
    snapshot_dir = model_dir / "snapshots" / SNAPSHOT_HASH
    snapshot_dir.mkdir(parents=True, exist_ok=True)
    symlink_path = snapshot_dir / gguf_file.name

    # Remove existing symlink if it exists
    if symlink_path.exists() or symlink_path.is_symlink():
        symlink_path.unlink()

    # Create symlink: snapshots/{hash}/file.gguf -> ../../blobs/{content-hash}
    relative_blob = Path("../..") / "blobs" / content_hash
    symlink_path.symlink_to(relative_blob)

    print(f"  ✓ Created HF structure for {model_name}")
    print(f"    Blob: {content_hash[:16]}...")
    print(f"    Snapshot: {SNAPSHOT_HASH}")


def create_writer(path: str, arch: str, endianess: GGUFEndian = GGUFEndian.LITTLE) -> GGUFWriter:
    """Create a GGUFWriter with specified architecture"""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    return GGUFWriter(path, arch, endianess=endianess)


def add_basic_metadata(writer: GGUFWriter, model_name: str = "test-model") -> None:
    """Add basic required metadata"""
    writer.add_string("general.name", model_name)
    writer.add_array("tokenizer.ggml.tokens", ["<s>", "</s>", "hello", "world"])
    writer.add_uint32("tokenizer.ggml.bos_token_id", 0)
    writer.add_uint32("tokenizer.ggml.eos_token_id", 1)


def add_context_length(writer: GGUFWriter, arch: str, context_length: int = 4096) -> None:
    """Add context length for specified architecture"""
    writer.add_uint32(f"{arch}.context_length", context_length)


def add_parameter_count(writer: GGUFWriter, count: int = 1_000_000_000) -> None:
    """Add parameter count"""
    writer.add_uint64("general.parameter_count", count)


def add_vision_metadata(writer: GGUFWriter) -> None:
    """Add CLIP vision encoder metadata"""
    writer.add_bool("clip.has_vision_encoder", True)
    writer.add_string("clip.projector_type", "mlp")
    writer.add_uint32("clip.vision.image_size", 224)
    writer.add_uint32("clip.vision.patch_size", 14)
    writer.add_uint32("clip.vision.embedding_length", 1024)


def add_audio_metadata(writer: GGUFWriter) -> None:
    """Add CLIP audio encoder metadata"""
    writer.add_bool("clip.has_audio_encoder", True)
    writer.add_uint32("clip.audio.num_mel_bins", 128)
    writer.add_uint32("clip.audio.embedding_length", 768)
    writer.add_uint32("clip.audio.block_count", 12)


def write_and_close(writer: GGUFWriter) -> None:
    """Write all sections and close the writer"""
    writer.write_header_to_file()
    writer.write_kv_data_to_file()
    writer.write_tensors_to_file()
    writer.close()


def create_test_files():
    """Create all capability test GGUF files with full HF structure"""
    # Determine paths relative to project root
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent.parent  # BodhiApp root
    temp_dir = script_dir.parent / "data" / "temp-e2e-gguf"
    output_dir = project_root / "crates" / "lib_bodhiserver_napi" / "tests-js" / "data" / "test-gguf"

    # Clean and create temp directory
    if temp_dir.exists():
        shutil.rmtree(temp_dir)
    temp_dir.mkdir(parents=True, exist_ok=True)

    # Clean output directory
    if output_dir.exists():
        shutil.rmtree(output_dir)

    print(f"Generating test GGUF files...")
    print(f"Output directory: {output_dir}")
    print()

    models = []

    # 1. llama-plain.gguf - No capabilities (baseline)
    print("Creating llama-plain.gguf...")
    gguf_path = temp_dir / "llama-plain.gguf"
    writer = create_writer(str(gguf_path), "llama")
    add_basic_metadata(writer, "llama-plain")
    add_context_length(writer, "llama", 4096)
    add_parameter_count(writer, 7_000_000_000)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("llama-plain", gguf_path))

    # 2. qwen-vision.gguf - Vision only (architecture-based detection)
    print("Creating qwen-vision.gguf...")
    gguf_path = temp_dir / "qwen-vision.gguf"
    writer = create_writer(str(gguf_path), "qwen2vl")
    add_basic_metadata(writer, "qwen-vision")
    add_context_length(writer, "qwen2vl", 32768)
    add_parameter_count(writer, 2_000_000_000)
    add_vision_metadata(writer)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("qwen-vision", gguf_path))

    # 3. phi-tools.gguf - Tool calling + structured output (template-based)
    print("Creating phi-tools.gguf...")
    gguf_path = temp_dir / "phi-tools.gguf"
    writer = create_writer(str(gguf_path), "phi3")
    add_basic_metadata(writer, "phi-tools")
    add_context_length(writer, "phi3", 4096)
    add_parameter_count(writer, 3_800_000_000)
    writer.add_string("tokenizer.chat_template", TOOLS_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("phi-tools", gguf_path))

    # 4. deepseek-thinking.gguf - Thinking capability (name + template)
    print("Creating deepseek-thinking.gguf...")
    gguf_path = temp_dir / "deepseek-thinking.gguf"
    writer = create_writer(str(gguf_path), "deepseek")
    add_basic_metadata(writer, "deepseek-r1")  # Name-based detection
    add_context_length(writer, "deepseek", 8192)
    add_parameter_count(writer, 7_000_000_000)
    writer.add_string("tokenizer.chat_template", THINKING_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("deepseek-thinking", gguf_path))

    # 5. mistral-audio.gguf - Audio capability (metadata-based)
    print("Creating mistral-audio.gguf...")
    gguf_path = temp_dir / "mistral-audio.gguf"
    writer = create_writer(str(gguf_path), "mistral")
    add_basic_metadata(writer, "mistral-audio")
    add_context_length(writer, "mistral", 8192)
    add_parameter_count(writer, 7_000_000_000)
    add_audio_metadata(writer)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("mistral-audio", gguf_path))

    # 6. llava-multimodal.gguf - Vision + tools + structured (multi-capability)
    print("Creating llava-multimodal.gguf...")
    gguf_path = temp_dir / "llava-multimodal.gguf"
    writer = create_writer(str(gguf_path), "llava")
    add_basic_metadata(writer, "llava-multimodal")
    add_context_length(writer, "llava", 4096)
    add_parameter_count(writer, 13_000_000_000)
    add_vision_metadata(writer)
    writer.add_string("tokenizer.chat_template", TOOLS_TEMPLATE)
    writer.add_string("general.file_type", "Q4_K_M")
    write_and_close(writer)
    models.append(("llava-multimodal", gguf_path))

    print()
    print("Creating HuggingFace cache structure...")
    for model_name, gguf_path in models:
        create_hf_structure(output_dir, model_name, gguf_path)

    # Clean up temp directory
    shutil.rmtree(temp_dir)

    print()
    print("✅ All test files created successfully!")
    print(f"📁 Output directory: {output_dir}")
    print()
    print("Created models:")
    for model_name, _ in models:
        print(f"  - models--test--{model_name}")


if __name__ == "__main__":
    try:
        create_test_files()
    except Exception as e:
        print(f"❌ Error: {e}")
        import traceback
        traceback.print_exc()
        exit(1)
