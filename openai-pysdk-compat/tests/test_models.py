import openai
import pytest
from .common import LLAMA3_MODEL, GPT_MODEL, not_implemented


@pytest.mark.vcr
@pytest.mark.parametrize(
  "client",
  [
    pytest.param("openai", id="openai"),
    pytest.param("bodhi", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_models_list(client):
  models = list(client.models.list())
  assert len(models) > 0


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  "client",
  [
    pytest.param("async_openai", id="async_openai"),
    pytest.param("async_bodhi", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_models_async_list(client):
  models = list(await client.models.list())
  assert len(models) > 0


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "expected"],
  [
    pytest.param(
      "openai",
      GPT_MODEL,
      {"id": "gpt-4o-2024-05-13", "object": "model", "created": 1715368132, "owned_by": "system"},
      id="openai",
    ),
    pytest.param("bodhi", LLAMA3_MODEL, {}, id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_models_retrieve(client, model, expected):
  model = client.models.retrieve(model)
  expected = openai.types.model.Model(**expected)
  assert expected == model


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "model", "expected"],
  [
    pytest.param(
      "async_openai",
      GPT_MODEL,
      {"id": "gpt-4o-2024-05-13", "object": "model", "created": 1715368132, "owned_by": "system"},
      id="async_openai",
    ),
    pytest.param("async_bodhi", LLAMA3_MODEL, {}, id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_models_async_retrieve(client, model, expected):
  model = await client.models.retrieve(model)
  expected = openai.types.model.Model(**expected)
  assert expected == model
