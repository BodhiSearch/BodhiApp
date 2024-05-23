import sys

import pytest
from openai.cli._cli import _main

from tests.common import GPT_MODEL, LLAMA3_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["cli_arg", "model"],
  [
    pytest.param("openai_cli", GPT_MODEL, id="openai_cli"),
    pytest.param("bodhi_cli", LLAMA3_MODEL, id="bodhi_cli"),
  ],
  indirect=["cli_arg"],
)
def test_cli_chat(capsys, cli_arg, model):
  args = [
    *cli_arg,
    "api",
    "chat.completions.create",
    "-m",
    model,
    "-g",
    "system",
    "You are a helpful assistant. Answer to the best of your ability.",
    "-g",
    "user",
    "What day comes after Monday? Answer in one word.",
    "-t",
    "1",
    "-n",
    "1",
    "-M",
    "30",
  ]
  print(f"{args=}")
  sys.argv = args
  _main()
  captured = capsys.readouterr()
  assert "Tuesday" in captured.out
