import pytest

from .common import GPT_EMBEDDINGS_MODEL, OSS_EMBEDDINGS_MODEL


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("openai", GPT_EMBEDDINGS_MODEL, id="openai"),
    pytest.param("bodhi", OSS_EMBEDDINGS_MODEL, id="bodhi", marks=pytest.mark.skip(reason="Not implemented yet")),
  ],
  indirect=["client"],
)
def test_embeddings_create(client, model):
  embeds = client.embeddings.create(model=model, input="What day comes after Monday?", encoding_format="float")
  assert embeds is not None
  assert isinstance(embeds.data[0].embedding[0], float)
