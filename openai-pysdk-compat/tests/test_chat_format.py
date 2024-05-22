import json

import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, LLAMA3_MODEL

input_json_format = {
  "seed": 42,
  "messages": [
    {
      "role": "user",
      "content": "Generate a JSON object representing a person with "
      "first name as John, last name as string Doe, age as 30",
    }
  ],
  "response_format": {"type": "json_object"},
}

expected = {"firstName": "John", "lastName": "Doe", "age": 30}


@pytest.mark.vcr
@pytest.mark.parametrize(
  "args",
  [input_json_format],
  ids=["format_json"],
)
def test_format_compare(openai_client, bodhi_client, args):
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args)
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
  # assert expected_usage_diff == diff.pop("values_changed") # TODO: implement
  assert ["root.choices[0].model_fields_set['logprobs']"] == diff.pop("set_item_removed")  # TODO: implement
  # assert {} == diff # TODO: implement


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi"),
  ],
  indirect=["client"],
)
def test_chat_format_simple(client, model):
  response = client.chat.completions.create(model=model, **input_json_format)
  json_obj = json.loads(response.choices[0].message.content)
  assert expected == json_obj


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("async_openai", GPT_MODEL, id="async_openai"),
    pytest.param("async_bodhi", LLAMA3_MODEL, id="async_bodhi"),
  ],
  indirect=["client"],
)
async def test_chat_async_format_simple(client, model):
  response = await client.chat.completions.create(model=model, **input_json_format)
  json_obj = json.loads(response.choices[0].message.content)
  assert expected == json_obj
