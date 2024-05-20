import json

import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, OSS_MODEL


@pytest.mark.vcr
def test_create_with_response_format(openai_client, bodhi_client):
  messages = [
    {
      "role": "user",
      "content": "Generate a JSON object representing a person with "
      "first name as John, last name as string Doe, age as 30",
    }
  ]
  args = {"seed": 42, "messages": messages, "response_format": {"type": "json_object"}}
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=OSS_MODEL, **args)
  exclude_paths = [
    "id",
    "created",
    "model",
    "system_fingerprint",
    "choices[0].message.content",
  ]
  diff = DeepDiff(gpt_response, bodhi_response, ignore_order=True, exclude_paths=exclude_paths)
  expected_usage_diff = {
    "root.usage.completion_tokens": {
      "new_value": 28,
      "old_value": 26,
    },
    "root.usage.prompt_tokens": {
      "new_value": 0,
      "old_value": 26,
    },
    "root.usage.total_tokens": {
      "new_value": 28,
      "old_value": 52,
    },
  }
  json_obj = json.loads(bodhi_response.choices[0].message.content)
  assert {"firstName": "John", "lastName": "Doe", "age": 30} == json_obj
  assert expected_usage_diff == diff.pop("values_changed")
  assert ["root.choices[0].model_fields_set['logprobs']"] == diff.pop("set_item_removed")  # TODO: implement
  assert {} == diff
