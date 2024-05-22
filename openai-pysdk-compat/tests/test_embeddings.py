import pytest

from .common import GPT_EMBEDDINGS_MODEL, OSS_EMBEDDINGS_MODEL


@pytest.mark.vcr
def test_embeddings_create(openai_client, bodhi_client):
  embeds = openai_client.embeddings.create(
    model=GPT_EMBEDDINGS_MODEL, input="What day comes after Monday?", encoding_format="float"
  )
  assert embeds is not None
  assert isinstance(embeds.data[0].embedding[0], float)

  # TODO: implement
  # embeds = bodhi_client.embeddings.create(
  #   model=OSS_EMBEDDINGS_MODEL, input="What day comes after Monday?", encoding_format="float"
  # )
  # assert embeds is not None
  # assert isinstance(embeds.data[0].embedding[0], float)
