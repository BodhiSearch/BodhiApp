import pytest
from openai import AuthenticationError, BadRequestError, NotFoundError, OpenAI

from .common import GPT_MODEL, LLAMA3_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "error"],
  [
    pytest.param(
      "openai",
      GPT_MODEL,
      {
        "code": "invalid_api_key",
        "message": "Incorrect API key provided: sk-foobar. You can find your API key at "
        "https://platform.openai.com/account/api-keys.",
        "param": None,
        "type": "invalid_request_error",
      },
      id="openai",
    ),
    pytest.param("bodhi", LLAMA3_MODEL, {}, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_exception_auth_error_on_invalid_api_key(client: OpenAI, model, error):
  client.api_key = "sk-foobar"
  with pytest.raises(AuthenticationError) as e:
    client.chat.completions.create(
      model=model, messages=[{"role": "user", "seed": 42, "content": "What day comes after Monday?"}]
    )
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
      {
        "code": "invalid_value",
        "message": "Invalid value: 'foobar'. "
        "Supported values are: 'system', 'assistant', 'user', 'function', and 'tool'.",
        "param": "messages[0].role",
        "type": "invalid_request_error",
      },
    )
  ],
)
def test_exception_input_error(client: OpenAI, model, input, exception, error):
  with pytest.raises(exception) as e:
    client.chat.completions.create(model=model, **input)
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
      {
        "code": "model_not_found",
        "message": "The model 'gpt-4o-foo' does not exist",
        "param": "model",
        "type": "invalid_request_error",
      },
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
