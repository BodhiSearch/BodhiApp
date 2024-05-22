import re
import pytest
from deepdiff import DeepDiff

from .common import GPT_MODEL, OSS_MODEL

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
    (
      {
        "stream": True,
        "seed": 42,
        "messages": [{"role": "user", "content": "Answer in one word. What day comes after Monday?"}],
      }
    ),
    (params_overload),
  ],
  ids=["stream_simple", "stream_overload"],
)
def test_chat_stream(openai_client, bodhi_client, args):
  gpt_response = openai_client.chat.completions.create(model=GPT_MODEL, **args)
  gpt_deltas = []
  for chunk in gpt_response:
    gpt_deltas.append(chunk)
  bodhi_response = bodhi_client.chat.completions.create(model=OSS_MODEL, **args)
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
