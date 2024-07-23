import pytest
from deepdiff import DeepDiff

from .common import LLAMA3_MODEL, mark_ollama_bodhi, mark_ollama

ollama_model_info = {
  "general.architecture": "llama",
  "general.file_type": 2,
  "general.parameter_count": 8030261248,
  "general.quantization_version": 2,
  "llama.attention.head_count": 32,
  "llama.attention.head_count_kv": 8,
  "llama.attention.layer_norm_rms_epsilon": 1e-05,
  "llama.block_count": 32,
  "llama.context_length": 8192,
  "llama.embedding_length": 4096,
  "llama.feed_forward_length": 14336,
  "llama.rope.dimension_count": 128,
  "llama.rope.freq_base": 500000,
  "llama.vocab_size": 128256,
  "tokenizer.ggml.bos_token_id": 128000,
  "tokenizer.ggml.eos_token_id": 128001,
  "tokenizer.ggml.merges": None,
  "tokenizer.ggml.model": "gpt2",
  "tokenizer.ggml.token_type": None,
  "tokenizer.ggml.tokens": None,
}
ollama_model_details = {
  "families": ["llama"],
  "family": "llama",
  "format": "gguf",
  "parameter_size": "8B",
  "parent_model": "",
  "quantization_level": "Q4_0",
}
bodhi_model_details = {
  "families": None,
  "family": "llama3",
  "format": "gguf",
  "parameter_size": "",
  "parent_model": None,
  "quantization_level": "",
}
bodhi_model_info = {}


@pytest.mark.parametrize(
  "client",
  [
    pytest.param("ollama", id="ollama", **mark_ollama()),
    pytest.param("ollama_bodhi", id="ollama_bodhi", **mark_ollama_bodhi()),
  ],
  indirect=["client"],
)
def test_ollama_models_list(client):
  models = list(client.list())
  assert len(models) > 0


@pytest.mark.parametrize(
  ["client", "expected"],
  [
    pytest.param(
      "ollama", {"details": ollama_model_details, "model_info": ollama_model_info}, id="ollama", **mark_ollama()
    ),
    pytest.param(
      "ollama_bodhi",
      {"details": bodhi_model_details, "model_info": bodhi_model_info},
      id="ollama_bodhi",
      **mark_ollama_bodhi(),
    ),
  ],
  indirect=["client"],
)
def test_ollama_models_retrieve(client, expected):
  model = client.show(LLAMA3_MODEL)
  assert {} == DeepDiff(model["details"], expected["details"])
  assert {} == DeepDiff(model["model_info"], expected["model_info"])
