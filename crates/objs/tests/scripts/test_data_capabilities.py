#!/usr/bin/env python3
"""
Generate test GGUF files with specific capability patterns for testing metadata extraction.

This script creates 6 GGUF files covering different capability combinations:
1. llama-plain.gguf - No capabilities (baseline)
2. qwen-vision.gguf - Vision only (architecture-based)
3. phi-tools.gguf - Tool calling + structured output (template-based)
4. deepseek-thinking.gguf - Thinking capability (name-based + template)
5. mistral-audio.gguf - Audio capability (metadata-based)
6. llava-multimodal.gguf - Vision + tool calling + structured output (multi-capability)
"""

from gguf import GGUFEndian, GGUFWriter
import os

# Chat template with tool calling and structured output patterns
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

# Chat template with thinking patterns
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

# Basic chat template without special capabilities
BASIC_TEMPLATE = """{% for message in messages %}
<|start_header_id|>{{ message['role'] }}<|end_header_id|>
{{ message['content'] }}<|eot_id|>
{% endfor %}
{% if add_generation_prompt %}
<|start_header_id|>assistant<|end_header_id|>
{% endif %}"""


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
    """Create all capability test GGUF files"""
    base_dir = "tests/data/gguf-capabilities"

    # 1. llama-plain.gguf - No capabilities (baseline)
    print("Creating llama-plain.gguf...")
    writer = create_writer(f"{base_dir}/llama-plain.gguf", "llama")
    add_basic_metadata(writer, "llama-plain")
    add_context_length(writer, "llama", 4096)
    add_parameter_count(writer, 7_000_000_000)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    write_and_close(writer)

    # 2. qwen-vision.gguf - Vision only (architecture-based detection)
    print("Creating qwen-vision.gguf...")
    writer = create_writer(f"{base_dir}/qwen-vision.gguf", "qwen2vl")
    add_basic_metadata(writer, "qwen-vision")
    add_context_length(writer, "qwen2vl", 32768)
    add_parameter_count(writer, 2_000_000_000)
    add_vision_metadata(writer)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    write_and_close(writer)

    # 3. phi-tools.gguf - Tool calling + structured output (template-based)
    print("Creating phi-tools.gguf...")
    writer = create_writer(f"{base_dir}/phi-tools.gguf", "phi3")
    add_basic_metadata(writer, "phi-tools")
    add_context_length(writer, "phi3", 4096)
    add_parameter_count(writer, 3_800_000_000)
    writer.add_string("tokenizer.chat_template", TOOLS_TEMPLATE)
    write_and_close(writer)

    # 4. deepseek-thinking.gguf - Thinking capability (name + template)
    print("Creating deepseek-thinking.gguf...")
    writer = create_writer(f"{base_dir}/deepseek-thinking.gguf", "deepseek")
    add_basic_metadata(writer, "deepseek-r1")  # Name-based detection
    add_context_length(writer, "deepseek", 8192)
    add_parameter_count(writer, 7_000_000_000)
    writer.add_string("tokenizer.chat_template", THINKING_TEMPLATE)
    write_and_close(writer)

    # 5. mistral-audio.gguf - Audio capability (metadata-based)
    print("Creating mistral-audio.gguf...")
    writer = create_writer(f"{base_dir}/mistral-audio.gguf", "mistral")
    add_basic_metadata(writer, "mistral-audio")
    add_context_length(writer, "mistral", 8192)
    add_parameter_count(writer, 7_000_000_000)
    add_audio_metadata(writer)
    writer.add_string("tokenizer.chat_template", BASIC_TEMPLATE)
    write_and_close(writer)

    # 6. llava-multimodal.gguf - Vision + tools + structured (multi-capability)
    print("Creating llava-multimodal.gguf...")
    writer = create_writer(f"{base_dir}/llava-multimodal.gguf", "llava")
    add_basic_metadata(writer, "llava-multimodal")
    add_context_length(writer, "llava", 4096)
    add_parameter_count(writer, 13_000_000_000)
    add_vision_metadata(writer)
    writer.add_string("tokenizer.chat_template", TOOLS_TEMPLATE)
    write_and_close(writer)

    print("All capability test files created successfully!")


if __name__ == "__main__":
    if os.path.basename(os.getcwd()) == "BodhiApp":
        os.chdir("crates/objs")
    if os.path.basename(os.getcwd()) != "objs":
        raise Exception(
            "Could not locate objs directory, either run from the project root (BodhiApp) or the module root (crates/objs)"
        )

    create_test_files()
