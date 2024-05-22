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


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model"],
  [
    pytest.param("async_openai", GPT_EMBEDDINGS_MODEL, id="async_openai"),
    pytest.param(
      "async_bodhi", OSS_EMBEDDINGS_MODEL, id="async_bodhi", marks=pytest.mark.skip(reason="Not implemented yet")
    ),
  ],
  indirect=["client"],
)
async def test_embeddings_async_create(client, model):
  embeds = await client.embeddings.create(model=model, input="What day comes after Monday?", encoding_format="float")
  assert embeds is not None
  assert isinstance(embeds.data[0].embedding[0], float)
