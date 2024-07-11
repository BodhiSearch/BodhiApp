import json

import pytest

from .common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi, mark_openai


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
def test_chat_function_simple(client, model):
  response = client.chat.completions.create(
    model=model,
    messages=[{"role": "user", "content": "What's the weather like in Boston today?"}],
    functions=[
      {
        "name": "get_current_weather",
        "description": "Get the current weather in a given location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "The city and state, e.g. San Francisco, CA",
            },
            "unit": {"type": "string", "enum": ["celsius", "fahrenheit"]},
          },
          "required": ["location"],
        },
      }
    ],
  )
  choice = response.choices[0]
  assert choice.finish_reason == "function_call"
  assert choice.message.content is None
  function_call = choice.message.function_call
  assert json.loads(function_call.arguments) == {"location": "Boston, MA"}
  assert function_call.name == "get_current_weather"
