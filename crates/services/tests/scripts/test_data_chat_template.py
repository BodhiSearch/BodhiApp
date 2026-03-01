from gguf import GGUFEndian, GGUFWriter
from typing import Optional, List

TARGET_DIR = (
  "tests/data/gguf-chat-template/models--FakeFactory--fakemodel-gguf/snapshots/5007652f7a641fe7170e0bad4f63839419bd9213"
)
CHAT_TEMPLATE = "{% for message in messages %}{{ message.role }}: {{ message.content }}{% endfor %}"


def create_chat_template_writer(filename: str, endianess: GGUFEndian = GGUFEndian.LITTLE) -> GGUFWriter:
  """Create a GGUFWriter for chat template tests"""
  return GGUFWriter(f"{TARGET_DIR}/{filename}", "llama", endianess=endianess)


def add_chat_template_data(
  writer: GGUFWriter,
  chat_template: Optional[str] = None,
  tokens: Optional[List[str]] = None,
  bos_token_id: Optional[int] = None,
  eos_token_id: Optional[int] = None,
) -> None:
  """Add chat template and token related data"""
  if chat_template is not None:
    writer.add_string("tokenizer.chat_template", chat_template)

  if tokens is not None:
    writer.add_array("tokenizer.ggml.tokens", tokens)

  if bos_token_id is not None:
    writer.add_uint32("tokenizer.ggml.bos_token_id", bos_token_id)

  if eos_token_id is not None:
    writer.add_uint32("tokenizer.ggml.eos_token_id", eos_token_id)


def create_valid_complete() -> None:
  """Create GGUF with all required fields for chat template"""
  writer = create_chat_template_writer("valid_complete.gguf")
  add_chat_template_data(
    writer,
    chat_template=CHAT_TEMPLATE,
    tokens=["<s>", "</s>", "hello", "world"],
    bos_token_id=0,
    eos_token_id=1,
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_missing_chat_template() -> None:
  """Create GGUF without chat template"""
  writer = create_chat_template_writer("missing_chat_template.gguf")
  add_chat_template_data(writer, tokens=["<s>", "</s>", "hello", "world"], bos_token_id=0, eos_token_id=1)
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_missing_tokens() -> None:
  """Create GGUF without tokens array"""
  writer = create_chat_template_writer("missing_tokens.gguf")
  add_chat_template_data(
    writer,
    chat_template=CHAT_TEMPLATE,
    bos_token_id=0,
    eos_token_id=1,
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_missing_token_ids() -> None:
  """Create GGUF without bos/eos token IDs"""
  writer = create_chat_template_writer("missing_token_ids.gguf")
  add_chat_template_data(
    writer,
    chat_template=CHAT_TEMPLATE,
    tokens=["<s>", "</s>", "hello", "world"],
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_invalid_token_ids() -> None:
  """Create GGUF with out-of-bounds token IDs"""
  writer = create_chat_template_writer("invalid_token_ids.gguf")
  add_chat_template_data(
    writer,
    chat_template=CHAT_TEMPLATE,
    tokens=["<s>", "</s>", "hello", "world"],
    bos_token_id=999,  # Out of bounds
    eos_token_id=1000,  # Out of bounds
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_empty_template() -> None:
  """Create GGUF with empty chat template string"""
  writer = create_chat_template_writer("empty_template.gguf")
  add_chat_template_data(
    writer, chat_template="", tokens=["<s>", "</s>", "hello", "world"], bos_token_id=0, eos_token_id=1
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def create_unicode_tokens() -> None:
  """Create GGUF with Unicode tokens"""
  writer = create_chat_template_writer("unicode_tokens.gguf")
  add_chat_template_data(
    writer,
    chat_template=CHAT_TEMPLATE,
    tokens=["<s>", "</s>", "こんにちは", "世界", "🌍", "👋"],
    bos_token_id=0,
    eos_token_id=1,
  )
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


if __name__ == "__main__":
  import os

  if os.path.basename(os.getcwd()) == "BodhiApp":
    os.chdir("crates/services")
  if os.path.basename(os.getcwd()) not in ("objs", "services"):
    raise Exception(
      "Could not locate services directory, either run from the project root (BodhiApp) or the module root (crates/services)"
    )

  os.makedirs(
    TARGET_DIR,
    exist_ok=True,
  )

  # Create all test files
  create_valid_complete()
  create_missing_chat_template()
  create_missing_tokens()
  create_missing_token_ids()
  create_invalid_token_ids()
  create_empty_template()
  create_unicode_tokens()
