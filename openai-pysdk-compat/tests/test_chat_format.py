import json
from typing import List

import pytest
from deepdiff import DeepDiff
from pydantic import BaseModel, TypeAdapter

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


class Champion(BaseModel):
  yr: int
  first_name: str
  last_name: str


class Champions(BaseModel):
  champions: List[Champion]


schema = TypeAdapter(Champions).json_schema()

winner_expected = [
  {"yr": 2002, "first_name": "Lleyton", "last_name": "Hewitt"},
  {"yr": 2003, "first_name": "Roger", "last_name": "Federer"},
  {"yr": 2004, "first_name": "Roger", "last_name": "Federer"},
]
prompt_wimblendon = f"""provide list of wimblendon men's single winners from 2002 to 2004.
You respond in JSON format with each entry using the following schema:
```
{json.dumps(schema, indent=2)}
```
"""


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "response_format", "expected"],
  [
    pytest.param(
      "openai", GPT_MODEL, {"type": "json_object"}, {"champions": winner_expected}, id="openai", **mark_openai()
    ),
    pytest.param(
      "bodhi",
      LLAMA3_MODEL,
      {"type": "json_object", "schema": schema},
      {"champions": winner_expected, "title": "Wimbledon Men's Singles Winners (2002-2004)", "type": "array"},
      id="bodhi",
      **mark_bodhi(),
    ),
  ],
  indirect=["client"],
)
def test_chat_format_schema(client, model, response_format, expected):
  response = client.chat.completions.create(
    model=model,
    messages=[
      {
        "role": "system",
        "content": "You are a helpful assistant that generates the output in the given json schema format",
      },
      {
        "role": "user",
        "content": prompt_wimblendon,
      },
    ],
    response_format=response_format,
  )
  actual = json.loads(response.choices[0].message.content)
  assert expected["champions"] == actual["champions"]
  assert expected == actual
