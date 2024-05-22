import pytest

from .common import GPT_EMBEDDINGS_MODEL, OSS_EMBEDDINGS_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client_key", "model"],
  [
    pytest.param("openai", GPT_EMBEDDINGS_MODEL, id="openai"),
    pytest.param("bodhi", OSS_EMBEDDINGS_MODEL, id="bodhi", marks=pytest.mark.skip(reason="Not implemented yet")),
  ],
)
def test_embeddings_create(api_clients, client_key, model):
  client = api_clients[client_key]
  embeds = client.embeddings.create(
    model=GPT_EMBEDDINGS_MODEL, input="What day comes after Monday?", encoding_format="float"
  )
  assert embeds is not None
  assert isinstance(embeds.data[0].embedding[0], float)
