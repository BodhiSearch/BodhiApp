import pytest

from .common import LLAMA3_MODEL, mark_ollama, mark_ollama_bodhi

chat_params = {
  "messages": [
    {
      "role": "user",
      "content": "Answer in one word. What day comes after Monday?",
    },
  ]
}


@pytest.mark.parametrize(
  "client",
  [
    pytest.param("ollama", id="ollama", **mark_ollama()),
    pytest.param("ollama_bodhi", id="ollama_bodhi", **mark_ollama_bodhi()),
  ],
  indirect=["client"],
)
def test_ollama_chat_run(client):
  response = client.chat(model=LLAMA3_MODEL, **chat_params)
  content = response["message"]["content"]
  assert "Tuesday" in content


@pytest.mark.parametrize(
  "client",
  [
    pytest.param("ollama", id="ollama", **mark_ollama()),
    pytest.param("ollama_bodhi", id="ollama_bodhi", **mark_ollama_bodhi()),
  ],
  indirect=["client"],
)
def test_ollama_chat_stream(client):
  response = client.chat(model=LLAMA3_MODEL, stream=True, **chat_params)
  deltas = []
  for chunk in response:
    content = chunk["message"]["content"]
    if content is not None:
      deltas.append(content)
  assert "Tuesday" in "".join(deltas)
