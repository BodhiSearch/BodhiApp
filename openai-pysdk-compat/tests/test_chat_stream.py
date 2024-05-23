import re

import pytest
from deepdiff import DeepDiff
from openai import OpenAI

from .common import GPT_MODEL, LLAMA3_MODEL, mark_bodhi, mark_openai

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
  "stream": True,
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
  "args",
  [
    pytest.param(
      {
        "stream": True,
        "seed": 42,
        "messages": [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}],
      },
      id="stream_simple",
      marks=pytest.mark.unmarked(),
    ),
    pytest.param(
      params_overload,
      id="stream_overload",
      marks=pytest.mark.unmarked(),
    ),
  ],
)
def test_chat_stream_compare(openai_client, bodhi_client, args):
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  gpt_deltas = []
  for chunk in gpt_response:
    gpt_deltas.append(chunk)
  bodhi_response = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args)
  bodhi_deltas = []
  for chunk in bodhi_response:
    bodhi_deltas.append(chunk)
  exclude_dynamic = re.compile(r"root\[\d+\]\.(id|created|model|system_fingerprint)")
  exclude_content = re.compile(r"root\[\d+\]\.choices\[0\]\.delta\.content")
  exclude_finish_reason = re.compile(r"root\[\d+\]\.choices\[0\]\.finish_reason")
  exclude_regex_paths = [
    exclude_dynamic,
    exclude_content,
    exclude_finish_reason,
  ]
  exclude_paths = []
  diff = DeepDiff(
    gpt_deltas,
    bodhi_deltas,
    exclude_regex_paths=exclude_regex_paths,
    exclude_paths=exclude_paths,
  )
  assert gpt_deltas[-1].choices[0].finish_reason == bodhi_deltas[-1].choices[0].finish_reason
  assert {} == diff.pop("values_changed", {})
  # assert {} == diff.pop("type_changes", {}) # TODO: implement
  # assert {} == diff # TODO: implement


@pytest.mark.vcr
@pytest.mark.parametrize(
  "args",
  [
    pytest.param(
      params_overload,
      id="stream_usage",
      marks=pytest.mark.unmarked(),
    ),
  ],
)
def test_chat_stream_usage(openai_client, bodhi_client, args):
  args = args.copy()
  args["stream_options"] = {"include_usage": True}
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  gpt_deltas = []
  for chunk in gpt_response:
    gpt_deltas.append(chunk)
  assert gpt_deltas[-1].usage is not None
  bodhi_response = bodhi_client.chat.completions.create(model=LLAMA3_MODEL, **args)
  bodhi_deltas = []
  for chunk in bodhi_response:
    bodhi_deltas.append(chunk)
  # assert bodhi_deltas[-1].usage is not None # TODO: implement


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", **mark_openai()),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
def test_chat_stream_run(client, model):
  args = dict(**params_overload)
  response = client.chat.completions.create(model=model, **args)
  deltas = []
  for chunk in response:
    content = chunk.choices[0].delta.content
    if content is not None:
      deltas.append(content)
  assert "Tuesday" == "".join(deltas)


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
async def test_chat_async_stream_run(client, model):
  response = await client.chat.completions.create(model=model, **params_overload)
  deltas = []
  async for chunk in response:
    content = chunk.choices[0].delta.content
    if content is not None:
      deltas.append(content)
  assert "Tuesday" == "".join(deltas)
