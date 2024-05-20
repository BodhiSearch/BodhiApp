import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, OSS_MODEL


@pytest.mark.vcr
def test_chat_simple(openai_client, bodhi_client):
  messages = [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}]
  args = {"seed": 42, "messages": messages}
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=OSS_MODEL, **args)
  assert GPT_MODEL == gpt_response.model
  assert OSS_MODEL == bodhi_response.model
  assert "Tuesday." == gpt_response.choices[0].message.content
  assert "\nTuesday" == bodhi_response.choices[0].message.content
  exclude_paths = [
    "id",
    "created",
    "model",
    "system_fingerprint",
    "choices[0].message.content",
    "usage",  # TODO: implement
  ]
  diff = DeepDiff(gpt_response, bodhi_response, ignore_order=True, exclude_paths=exclude_paths)
  assert ["root.choices[0].model_fields_set['logprobs']"] == diff.pop("set_item_removed")
  expected_usage_diff = {
    "root.usage.completion_tokens": {
      "new_value": 5,
      "old_value": 2,
    },
    "root.usage.prompt_tokens": {
      "new_value": 24,
      "old_value": 18,
    },
    "root.usage.total_tokens": {
      "new_value": 29,
      "old_value": 20,
    },
  }
  # assert expected_usage_diff == diff.pop("values_changed") # TODO: implement
  assert {} == diff


@pytest.mark.vcr
def test_chat_system(openai_client, bodhi_client):
  messages = [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Answer in one word. What day comes after Monday?"},
  ]
  args = {"seed": 42, "messages": messages}
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=OSS_MODEL, **args)
  assert GPT_MODEL == gpt_response.model
  assert OSS_MODEL == bodhi_response.model
  assert "Tuesday" == gpt_response.choices[0].message.content
  assert "\nTuesday" == bodhi_response.choices[0].message.content
  exclude_paths = [
    "id",
    "created",
    "model",
    "system_fingerprint",
    "choices[0].message.content",
  ]
  diff = DeepDiff(gpt_response, bodhi_response, ignore_order=True, exclude_paths=exclude_paths)
  assert ["root.choices[0].model_fields_set['logprobs']"] == diff.pop("set_item_removed")
  expected_usage_diff = {
    "root.usage.completion_tokens": {
      "new_value": 5,
      "old_value": 1,
    },
    "root.usage.prompt_tokens": {
      "new_value": 29,
      "old_value": 28,
    },
    "root.usage.total_tokens": {
      "new_value": 34,
      "old_value": 29,
    },
  }
  assert expected_usage_diff == diff.pop("values_changed")
  assert {} == diff


@pytest.mark.vcr
def test_chat_params_temperature(openai_client, bodhi_client):
  messages = [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Answer in one word. What day comes after Monday?"},
  ]
  args = {"seed": 42, "messages": messages, "temperature": 0.7, "max_tokens": 10}
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=OSS_MODEL, **args)
  assert GPT_MODEL == gpt_response.model
  assert OSS_MODEL == bodhi_response.model
  assert "Tuesday" == gpt_response.choices[0].message.content
  assert "\nTuesday" == bodhi_response.choices[0].message.content
  exclude_paths = [
    "id",
    "created",
    "model",
    "system_fingerprint",
    "choices[0].message.content",
  ]
  diff = DeepDiff(gpt_response, bodhi_response, ignore_order=True, exclude_paths=exclude_paths)
  assert ["root.choices[0].model_fields_set['logprobs']"] == diff.pop("set_item_removed")
  expected_usage_diff = {
    "root.usage.completion_tokens": {
      "new_value": 5,
      "old_value": 1,
    },
    "root.usage.prompt_tokens": {
      "new_value": 0,
      "old_value": 28,
    },
    "root.usage.total_tokens": {
      "new_value": 5,
      "old_value": 29,
    },
  }
  assert expected_usage_diff == diff.pop("values_changed")
  assert {} == diff
