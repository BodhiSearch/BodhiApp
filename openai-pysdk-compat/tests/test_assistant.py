import pytest

from .common import GPT_MODEL, LLAMA3_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client_key", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip(reason="Not implemented yet")),
  ],
)
def test_assistant_create(api_clients, client_key, model):
  client = api_clients[client_key]
  assistant = client.beta.assistants.create(
    instructions="You are a personal math tutor. When asked a question, write and run Python code to answer the question.",
    name="Math Tutor",
    tools=[{"type": "code_interpreter"}],
    model=model,
  )
  assert assistant is not None
