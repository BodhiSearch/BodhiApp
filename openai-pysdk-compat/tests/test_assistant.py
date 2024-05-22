import pytest

from .common import GPT_MODEL, LLAMA3_MODEL

assistant_args = {
  "instructions": "You are a personal math tutor. When asked a question, write and run Python code to answer the question.",
  "name": "Math Tutor",
  "tools": [{"type": "code_interpreter"}],
}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip(reason="Not implemented yet")),
  ],
  indirect=["client"],
)
def test_assistant_create(client, model):
  assistant = client.beta.assistants.create(model=model, **assistant_args)
  assert assistant is not None


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("async_openai", GPT_MODEL, id="async_openai"),
    pytest.param("async_bodhi", LLAMA3_MODEL, id="async_bodhi", marks=pytest.mark.skip(reason="Not implemented yet")),
  ],
  indirect=["client"],
)
async def test_assistant_async_create(client, model):
  assistant = await client.beta.assistants.create(model=model, **assistant_args)
  assert assistant is not None
