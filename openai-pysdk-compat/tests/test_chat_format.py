import json

import pytest

from .common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi, mark_openai

input_json_format = {
  "seed": 42,
  "stream": False,
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
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
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
    pytest.param("async_openai", GPT_MODEL, id="async_openai", **mark_openai()),
    pytest.param("async_bodhi", LLAMA3_MODEL, id="async_bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
async def test_chat_async_format_simple(client, model):
  response = await client.chat.completions.create(model=model, **input_json_format)
  content = response.choices[0].message.content
  try:
    json_obj = json.loads(content)
  except json.JSONDecodeError:
    pytest.fail(f"Invalid JSON:\n{content}")
  assert expected == json_obj
