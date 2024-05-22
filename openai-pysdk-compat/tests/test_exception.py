import asyncio
import time

import pytest
from openai import APIConnectionError, AuthenticationError, BadRequestError, NotFoundError, OpenAI

from .common import GPT_MODEL, LLAMA3_MODEL

auth_error = {
  "code": "invalid_api_key",
  "message": "Incorrect API key provided: sk-foobar. You can find your API key at "
  "https://platform.openai.com/account/api-keys.",
  "param": None,
  "type": "invalid_request_error",
}
bad_request_error = {
  "code": "invalid_value",
  "message": "Invalid value: 'foobar'. " "Supported values are: 'system', 'assistant', 'user', 'function', and 'tool'.",
  "param": "messages[0].role",
  "type": "invalid_request_error",
}
not_found_error = {
  "code": "model_not_found",
  "message": "The model 'gpt-4o-foo' does not exist",
  "param": "model",
  "type": "invalid_request_error",
}
args = {"messages": [{"role": "user", "seed": 42, "content": "What day comes after Monday?"}]}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "error"],
  [
    pytest.param(
      "openai",
      GPT_MODEL,
      auth_error,
      id="openai",
    ),
    pytest.param("bodhi", LLAMA3_MODEL, {}, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_exception_auth_error_on_invalid_api_key(client: OpenAI, model, error):
  client.api_key = "sk-foobar"
  with pytest.raises(AuthenticationError) as e:
    client.chat.completions.create(model=model, **args)
  err = e.value
  assert 401 == err.status_code
  assert error == err.body


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "error"],
  [
    pytest.param(
      "async_openai",
      GPT_MODEL,
      auth_error,
      id="async_openai",
    ),
    pytest.param("async_bodhi", LLAMA3_MODEL, {}, id="async_bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
async def test_exception_async_auth_error_on_invalid_api_key(client: OpenAI, model, error):
  client.api_key = "sk-foobar"
  with pytest.raises(AuthenticationError) as e:
    await client.chat.completions.create(model=model, **args)
  err = e.value
  assert 401 == err.status_code
  assert error == err.body


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
@pytest.mark.parametrize(
  ["input", "exception", "error"],
  [
    (
      {"seed": 42, "messages": [{"role": "foobar", "content": "What day comes after Monday?"}]},
      BadRequestError,
      bad_request_error,
    )
  ],
)
def test_exception_input_error(client: OpenAI, model, input, exception, error):
  with pytest.raises(exception) as e:
    client.chat.completions.create(model=model, **input)
  err = e.value
  assert 400 == err.status_code
  assert error == err.body


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("async_openai", GPT_MODEL, id="async_openai"),
    pytest.param("async_bodhi", LLAMA3_MODEL, id="async_bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
@pytest.mark.parametrize(
  ["input", "exception", "error"],
  [
    (
      {"seed": 42, "messages": [{"role": "foobar", "content": "What day comes after Monday?"}]},
      BadRequestError,
      bad_request_error,
    )
  ],
)
async def test_exception_async_input_error(client: OpenAI, model, input, exception, error):
  with pytest.raises(exception) as e:
    await client.chat.completions.create(model=model, **input)
  err = e.value
  assert 400 == err.status_code
  assert error == err.body


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "exception", "error"],
  [
    pytest.param(
      "openai",
      "gpt-4o-foo",
      NotFoundError,
      not_found_error,
      id="openai",
    ),
    pytest.param("bodhi", "llama3:foo", NotFoundError, {}, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_exception_not_found(client, model, exception, error):
  with pytest.raises(exception) as e:
    client.models.retrieve(model)
  err = e.value
  assert 404 == err.status_code
  assert error == err.body


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "exception", "error"],
  [
    pytest.param(
      "async_openai",
      "gpt-4o-foo",
      NotFoundError,
      not_found_error,
      id="async_openai",
    ),
    pytest.param(
      "async_bodhi", "llama3:foo", NotFoundError, {}, id="async_bodhi", marks=pytest.mark.skip("Not implemented yet")
    ),
  ],
  indirect=["client"],
)
async def test_exception_async_not_found(client, model, exception, error):
  with pytest.raises(exception) as e:
    await client.models.retrieve(model)
  err = e.value
  assert 404 == err.status_code
  assert error == err.body


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai", marks=pytest.mark.skip("Not implemented yet")),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_exception_interrupt_generate(client, model):
  args = {
    "seed": 42,
    "stream": True,
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful content writer assistant. "
        "You write long form professional essays for a large media company. "
        "Follow user instruction and assist using best of your abilities.",
      },
      {
        "role": "user",
        "content": "Write a 500 words article on tourism in India. "
        "Cover the seasonal festivals, places to visit during those festivals, "
        "and other places with nature and interesting culture.",
      },
    ],
  }

  deltas = []

  response = client.chat.completions.create(model=model, **args)
  try:
    for chunk in response:
      content = chunk.choices[0].delta.content
      if content is not None:
        deltas.append(content)
      if len(deltas) > 5:
        break
  except Exception as e:
    print(f"{e=}")
  finally:
    response.close()
  assert 0 != len(deltas)
  assert "" != "".join(deltas)
