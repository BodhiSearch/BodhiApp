import pytest
from deepdiff import DeepDiff
from openai import OpenAI

from .common import GPT_MODEL, LLAMA3_MODEL

params_overload = {
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant. Do as directed by the user.",
      "name": "user",
    },
    {
      "role": "user",
      "content": "Answer in one word. What day comes after Monday?",
      "name": "user",
    },
  ],
  "frequency_penalty": 1,
  "n": 1,
  "presence_penalty": 1,
  "seed": 42,
  "temperature": 1,
  "top_p": 1,
  "user": "user-1234",
}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["args", "expected_gpt_response", "expected_diff"],
  [
    (
      {"seed": 42, "messages": [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}]},
      "Tuesday.",
      {"set_item_removed": ["root.choices[0].model_fields_set['logprobs']"]},  # TODO: implement
    ),
    (
      {
        "seed": 42,
        "messages": [
          {"role": "system", "content": "You are a helpful assistant."},
          {"role": "user", "content": "Answer in one word. What day comes after Monday?"},
        ],
      },
      "Tuesday",
      {},
    ),
    (params_overload, "Tuesday", {}),
  ],
  ids=["simple", "system", "overload"],
)
def test_chat_compare(openai_client, bodhi_client, args, expected_gpt_response, expected_diff):
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  bodhi_response = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args)
  assert GPT_MODEL == gpt_response.model
  assert LLAMA3_MODEL == bodhi_response.model
  assert expected_gpt_response == gpt_response.choices[0].message.content
  assert "Tuesday" == bodhi_response.choices[0].message.content
  exclude_paths = [
    "id",
    "created",
    "model",
    "system_fingerprint",
    "choices[0].message.content",
    "usage",  # TODO: implement
  ]
  diff = DeepDiff(gpt_response, bodhi_response, ignore_order=True, exclude_paths=exclude_paths)
  # expected_usage_diff = {
  #   "root.usage.completion_tokens": {
  #     "new_value": 5,
  #     "old_value": 2,
  #   },
  #   "root.usage.prompt_tokens": {
  #     "new_value": 24,
  #     "old_value": 18,
  #   },
  #   "root.usage.total_tokens": {
  #     "new_value": 29,
  #     "old_value": 20,
  #   },
  # }
  # assert expected_usage_diff == diff.pop("values_changed") # TODO: implement
  assert expected_diff == diff


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi"),
  ],
  indirect=["client"],
)
def test_chat_run(client, model):
  response = client.chat.completions.create(model=model, **params_overload)
  content = response.choices[0].message.content
  assert "Tuesday" == content


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
async def test_chat_async_run(client, model):
  response = await client.chat.completions.create(model=model, **params_overload)
  content = response.choices[0].message.content
  assert "Tuesday" == content
