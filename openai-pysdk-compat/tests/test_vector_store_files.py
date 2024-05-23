import pytest
from deepdiff import DeepDiff
from openai import OpenAI

from tests.common import mark_bodhi_skip, mark_openai


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client"],
  [
    pytest.param("openai", id="openai", **mark_openai()),
    pytest.param("bodhi", id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_vector_store_files_create(client: OpenAI):
  vectore_store = client.beta.vector_stores.create(name="test-vector-store-files")
  file = client.files.create(file=open("tests/data/assistants-guide.md", "rb"), purpose="assistants")
  vs_file = client.beta.vector_stores.files.create_and_poll(file_id=file.id, vector_store_id=vectore_store.id)
  assert vs_file is not None


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "vector_store_id"],
  [
    pytest.param("openai", "vs_z7r2OYHsaMp4MWT025PHctZ4", id="openai", **mark_openai()),
    pytest.param("bodhi", "xyz", id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_vector_store_files_list(client: OpenAI, vector_store_id):
  vector_store_files = list(client.beta.vector_stores.files.list(vector_store_id=vector_store_id))
  assert len(vector_store_files) >= 0


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "vector_store_id", "vector_store_file_id"],
  [
    pytest.param(
      "openai", "vs_z7r2OYHsaMp4MWT025PHctZ4", "file-G08t1WAY6fkAsVCYNnmSIcIU", id="openai", **mark_openai()
    ),
    pytest.param("bodhi", "vs-xyz", "file-xyz", id="bodhi", **mark_bodhi_skip()),
  ],
  indirect=["client"],
)
def test_vector_store_files_retrieve(client: OpenAI, vector_store_id, vector_store_file_id):
  file = client.beta.vector_stores.files.retrieve(vector_store_file_id, vector_store_id=vector_store_id)
  expected = {
    "id": vector_store_file_id,
    "last_error": None,
    "object": "vector_store.file",
    "status": "completed",
    "usage_bytes": 30049,
    "vector_store_id": vector_store_id,
  }
  diff = DeepDiff(expected, file.to_dict(), include_paths=expected.keys())
  assert {} == diff
