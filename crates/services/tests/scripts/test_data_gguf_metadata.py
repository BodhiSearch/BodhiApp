from gguf import GGUFEndian, GGUFWriter


def create_writer(filename: str, endianess: GGUFEndian = GGUFEndian.LITTLE) -> GGUFWriter:
  """Create a GGUFWriter with common settings"""
  return GGUFWriter(f"tests/data/gguf/{filename}", "llama", endianess=endianess)


def write_and_close(writer: GGUFWriter) -> None:
  """Write all sections and close the writer"""
  writer.write_header_to_file()
  writer.write_kv_data_to_file()
  writer.write_tensors_to_file()
  writer.close()


def add_sample1_kv_data(writer: GGUFWriter) -> None:
  """Add all KV data types used in sample1"""
  # Add one of each KV data type
  writer.add_uint8("test_uint8", 255)
  writer.add_int8("test_int8", -128)
  writer.add_uint16("test_uint16", 65535)
  writer.add_int16("test_int16", -32768)
  writer.add_uint32("test_uint32", 4294967295)
  writer.add_int32("test_int32", -2147483648)
  writer.add_uint64("test_uint64", 18446744073709551615)
  writer.add_int64("test_int64", -9223372036854775808)
  writer.add_float32("test_float32", 3.14159)
  writer.add_float64("test_float64", 2.718281828459045)
  writer.add_bool("test_bool", True)
  writer.add_string("test_string", "Hello GGUF!")
  writer.add_array("test_array_int", [1, 2, 3, 4, 5])
  writer.add_array("test_array_str", ["a", "b", "c"])

  # Original KV data
  writer.add_uint32("context_length", 2048)
  writer.add_float32("rope_freq_base", 10000.0)


def add_token_data(writer: GGUFWriter) -> None:
  """Add token-related data used in sample_tokens"""
  # Basic token info
  writer.add_uint32("vocab_size", 100)

  # Special tokens
  writer.add_bos_token_id(1)
  writer.add_eos_token_id(2)
  writer.add_pad_token_id(3)
  writer.add_sep_token_id(4)

  # Token list - mix of different types of tokens
  tokens = [
    "<s>",  # BOS token
    "</s>",  # EOS token
    "<pad>",  # Padding token
  ]
  writer.add_token_list(tokens)

  # Token settings
  writer.add_add_bos_token(True)  # Add BOS token automatically
  writer.add_add_eos_token(True)  # Add EOS token automatically
  writer.add_add_space_prefix(True)  # Add space prefix to words

  # Tokenizer settings
  writer.add_tokenizer_model("llama")
  writer.add_remove_extra_whitespaces(True)


def create_gguf_sample0(endianess: GGUFEndian) -> None:
  """Create GGUF with empty kv and tensors"""
  suffix = "le" if endianess == GGUFEndian.LITTLE else "be"
  writer = create_writer(f"sample0_{suffix}.gguf", endianess)
  write_and_close(writer)


def create_gguf_sample1(endianess: GGUFEndian) -> None:
  """Create GGUF with all KV data types"""
  suffix = "le" if endianess == GGUFEndian.LITTLE else "be"
  writer = create_writer(f"sample1_{suffix}.gguf", endianess)
  add_sample1_kv_data(writer)
  write_and_close(writer)


def create_gguf_sample_tokens(endianess: GGUFEndian = GGUFEndian.LITTLE) -> None:
  """Create GGUF with token list and token metadata"""
  suffix = "_be" if endianess == GGUFEndian.BIG else ""
  writer = create_writer(f"sample_tokens{suffix}.gguf", endianess)
  add_token_data(writer)
  write_and_close(writer)


if __name__ == "__main__":
  import os

  if os.path.basename(os.getcwd()) == "BodhiApp":
    os.chdir("crates/services")
  if os.path.basename(os.getcwd()) not in ("objs", "services"):
    raise Exception(
      "Could not locate services directory, either run from the project root (BodhiApp) or the module root (crates/services)"
    )

  os.makedirs("tests/data/gguf", exist_ok=True)

  # Create all sample files
  create_gguf_sample0(GGUFEndian.LITTLE)
  create_gguf_sample0(GGUFEndian.BIG)
  create_gguf_sample1(GGUFEndian.LITTLE)
  create_gguf_sample1(GGUFEndian.BIG)
  create_gguf_sample_tokens()
  create_gguf_sample_tokens(GGUFEndian.BIG)
