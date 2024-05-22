import pytest
from openai import AuthenticationError, OpenAI

from .common import GPT_MODEL, LLAMA3_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_MODEL, id="openai"),
    pytest.param("bodhi", LLAMA3_MODEL, id="bodhi", marks=pytest.mark.skip("Not implemented yet")),
  ],
  indirect=["client"],
)
def test_auth_error_on_invalid_api_key(client: OpenAI, model):
  client.api_key = "sk-foobar"
  with pytest.raises(AuthenticationError):
    client.chat.completions.create(
      model=model, messages=[{"role": "user", "seed": 42, "content": "What day comes after Monday?"}]
    )
