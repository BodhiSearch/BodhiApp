from gguf import GGUFEndian, GGUFWriter
import os

CHAT_TEMPLATE = """{% set loop_messages = messages %}{% for message in loop_messages %}{% set content = '<|start_header_id|>' + message['role'] + '<|end_header_id|>\n\n'+ message['content'] | trim + '<|eot_id|>' %}{% if loop.index0 == 0 %}{% set content = bos_token + content %}{% endif %}{{ content }}{% endfor %}{% if add_generation_prompt %}{{ '<|start_header_id|>assistant<|end_header_id|>\n\n' }}{% endif %}"""


def create_writer(path: str, endianess: GGUFEndian = GGUFEndian.LITTLE) -> GGUFWriter:
  """Create a GGUFWriter with common settings"""
  os.makedirs(os.path.dirname(path), exist_ok=True)
  return GGUFWriter(path, "llama", endianess=endianess)


def add_chat_template_data(writer: GGUFWriter) -> None:
  """Add chat template and required token data"""
  writer.add_string("tokenizer.chat_template", CHAT_TEMPLATE)
  writer.add_array("tokenizer.ggml.tokens", ["<s>", "</s>", "hello", "world"])
  writer.add_uint32("tokenizer.ggml.bos_token_id", 0)
  writer.add_uint32("tokenizer.ggml.eos_token_id", 1)


def write_and_close(writer: GGUFWriter) -> None:
  """Write all sections and close the writer"""
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_test_files():
  """Create all test GGUF files"""
  base_dir = "tests/data/huggingface/hub"

  # TheBloke/TinyLlama-1.1B-Chat-v0.3-GGUF - with chat template
  writer = create_writer(
    f"{base_dir}/models--TheBloke--TinyLlama-1.1B-Chat-v0.3-GGUF/snapshots/b32046744d93031a26c8e925de2c8932c305f7b9/tinyllama-1.1b-chat-v0.3.Q2_K.gguf"
  )
  add_chat_template_data(writer)
  write_and_close(writer)

  # TheBloke/Llama-2-7B-Chat-GGUF - with chat template
  writer = create_writer(
    f"{base_dir}/models--TheBloke--Llama-2-7B-Chat-GGUF/snapshots/191239b3e26b2882fb562ffccdd1cf0f65402adb/llama-2-7b-chat.Q8_0.gguf"
  )
  add_chat_template_data(writer)
  write_and_close(writer)

  # MyFactory/testalias-gguf - with chat template
  writer = create_writer(
    f"{base_dir}/models--MyFactory--testalias-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213/testalias.Q8_0.gguf"
  )
  write_and_close(writer)

  # FakeFactory/fakemodel-gguf - without chat template
  writer = create_writer(
    f"{base_dir}/models--FakeFactory--fakemodel-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213/fakemodel.Q4_0.gguf"
  )
  add_chat_template_data(writer)
  write_and_close(writer)

  # Another snapshot of FakeFactory/fakemodel-gguf - without chat template
  writer = create_writer(
    f"{base_dir}/models--FakeFactory--fakemodel-gguf/snapshots/9ca625120374ddaae21f067cb006517d14dc91a6/fakemodel.Q4_0.gguf"
  )
  add_chat_template_data(writer)
  write_and_close(writer)

  # google/gemma-1.1-2b-it-GGUF - without chat template
  writer = create_writer(
    f"{base_dir}/models--google--gemma-1.1-2b-it-GGUF/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213/2b_it_v1p1.gguf"
  )
  write_and_close(writer)


def create_refs():
  """Create refs files"""
  base_dir = "tests/data/huggingface/hub"
  refs = {
    "models--TheBloke--TinyLlama-1.1B-Chat-v0.3-GGUF/refs/main": "b32046744d93031a26c8e925de2c8932c305f7b9",
    "models--TheBloke--Llama-2-7B-Chat-GGUF/refs/main": "191239b3e26b2882fb562ffccdd1cf0f65402adb",
    "models--MyFactory--testalias-gguf/refs/main": "5007652f7a641fe7170e0bad4f63839419bd9213",
    "models--FakeFactory--fakemodel-gguf/refs/main": "5007652f7a641fe7170e0bad4f63839419bd9213",
    "models--FakeFactory--fakemodel-gguf/refs/non-main": "9ca625120374ddaae21f067cb006517d14dc91a6",
    "models--google--gemma-1.1-2b-it-GGUF/refs/main": "5007652f7a641fe7170e0bad4f63839419bd9213",
  }

  for path, content in refs.items():
    full_path = f"{base_dir}/{path}"
    os.makedirs(os.path.dirname(full_path), exist_ok=True)
    with open(full_path, "w") as f:
      f.write(content)


if __name__ == "__main__":
  if os.path.basename(os.getcwd()) == "BodhiApp":
    os.chdir("crates/objs")
  if os.path.basename(os.getcwd()) != "objs":
    raise Exception(
      "Could not locate objs directory, either run from the project root (BodhiApp) or the module root (crates/objs)"
    )

  create_test_files()
  create_refs()
