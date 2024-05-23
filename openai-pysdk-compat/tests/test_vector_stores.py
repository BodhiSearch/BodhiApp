import pytest
from deepdiff import DeepDiff
from openai import OpenAI

from tests.common import not_implemented

vector_store_meta = {
  "object": "vector_store",
  "file_counts": {"in_progress": 0, "completed": 0, "failed": 0, "cancelled": 0, "total": 0},
}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("openai", id="openai"),
    pytest.param("bodhi", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_vector_store_create(client: OpenAI, random_suffix):
  name = f"test_vector_store_create_{random_suffix}"
  result = client.beta.vector_stores.create(name=name)
  diff = DeepDiff(vector_store_meta, result.to_dict(), include_paths=vector_store_meta.keys())
  assert diff == {}


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("async_openai", id="async_openai"),
    pytest.param("async_bodhi", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_vector_store_create_async(client: OpenAI, random_suffix):
  name = f"test_vector_store_create_{random_suffix}"
  result = await client.beta.vector_stores.create(name=name)
  diff = DeepDiff(vector_store_meta, result.to_dict(), include_paths=vector_store_meta.keys())
  assert diff == {}


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("openai", id="openai"),
    pytest.param("bodhi", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_vector_stores_list(client: OpenAI):
  result = list(client.beta.vector_stores.list())
  assert 1 <= len(result)


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("async_openai", id="async_openai"),
    pytest.param("async_bodhi", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_vector_stores_async_list(client: OpenAI):
  result = list(await client.beta.vector_stores.list())
  assert 1 <= len(result)


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "vector_store_id"],
  [
    pytest.param("openai", "vs_63OsmuEsbAIqopz4APtR0JSp", id="openai"),
    pytest.param("bodhi", "xyz", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_vector_stores_retrieve(client: OpenAI, vector_store_id: str):
  result = client.beta.vector_stores.retrieve(vector_store_id)
  expected = {"id": vector_store_id, "object": "vector_store"}
  diff = DeepDiff(expected, result.to_dict(), include_paths=expected.keys())
  assert {} == diff


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "vector_store_id"],
  [
    pytest.param("async_openai", "vs_cUdw3KuFnFkWTsLcwTvoAaEi", id="async_openai"),
    pytest.param("async_bodhi", "xyz", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_vector_stores_retrieve_async(client: OpenAI, vector_store_id):
  result = await client.beta.vector_stores.retrieve(vector_store_id)
  expected = {"id": vector_store_id, "object": "vector_store"}
  diff = DeepDiff(expected, result.to_dict(), include_paths=expected.keys())
  assert {} == diff


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("openai", id="openai"),
    pytest.param("bodhi", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_vector_stores_delete(client: OpenAI, random_suffix):
  name = f"test_vs_delete_{random_suffix}"
  vs = client.beta.vector_stores.create(name=name)
  result = client.beta.vector_stores.delete(vs.id)
  expected = {"id": vs.id, "deleted": True, "object": "vector_store.deleted"}
  diff = DeepDiff(expected, result.to_dict(), include_paths=expected.keys())
  assert {} == diff


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("async_openai", id="async_openai"),
    pytest.param("async_bodhi", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_vector_stores_delete_async(client: OpenAI, random_suffix):
  name = f"test_vs_delete_{random_suffix}"
  vs = await client.beta.vector_stores.create(name=name)
  result = await client.beta.vector_stores.delete(vs.id)
  expected = {"id": vs.id, "deleted": True, "object": "vector_store.deleted"}
  diff = DeepDiff(expected, result.to_dict(), include_paths=expected.keys())
  assert {} == diff
