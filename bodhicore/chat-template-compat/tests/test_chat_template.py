import pytest
from transformers import AutoTokenizer

from .conftest import inputs_yaml

cases = list(inputs_yaml().keys())
# cases = ["error-user-at-even-with-system"]


# covered: llama2, llama2-legacy, llama3, phi3, gemma, zephyr, deepseek, command-r, openchat, empty
# missing: monarch, orion, vicuna, vicuna-orca
@pytest.mark.parametrize(
  ["format", "model"],
  [
    ("llama3", "meta-llama/Meta-Llama-3-8B-Instruct"),
    ("llama2", "meta-llama/Llama-2-13b-chat-hf"),
    ("phi3", "microsoft/Phi-3-mini-4k-instruct"),
    ("llama2-legacy", "mistralai/Mistral-7B-Instruct-v0.3"),
    ("llama2-legacy", "mistralai/Mixtral-8x7B-Instruct-v0.1"),
    ("gemma", "google/gemma-7b-it"),
    # ("zephyr", "HuggingFaceH4/zephyr-7b-beta"),
    ("deepseek", "deepseek-ai/deepseek-llm-67b-chat"),
    ("command-r", "CohereForAI/c4ai-command-r-plus"),
    ("openchat", "openchat/openchat-3.6-8b-20240522"),
  ],
)
@pytest.mark.parametrize(
  "input",
  cases,
  indirect=["input"],
)
def test_chat_template(format, model, input):
  messages = input["messages"]
  expected = input[format]
  tokenizer = AutoTokenizer.from_pretrained(model)
  if isinstance(expected, str):
    output = tokenizer.apply_chat_template(messages, add_generation_prompt=True, tokenize=False)
    assert expected == output, f"Expected:\n{output}\nend"
  elif expected.get("exception", False):
    with pytest.raises(Exception) as e:
      tokenizer.apply_chat_template(messages, add_generation_prompt=True, tokenize=False)
    assert expected["message"] == str(e.value)
  else:
    raise AssertionError(f"expectation object not supported: {expected}")
