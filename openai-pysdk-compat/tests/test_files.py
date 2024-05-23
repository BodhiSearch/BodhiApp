import pytest
from deepdiff import DeepDiff
from openai import OpenAI

from tests.common import not_implemented

file_to_upload = {
  "bytes": 3395,
  "filename": "file-to-upload.txt",
  "object": "file",
  "purpose": "assistants",
  "status": "processed",
  "status_details": None,
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
def test_files_upload(client: OpenAI):
  # TODO: pyvcr not working with uploading binary image file
  # failing when converting it to utf-8 encodings
  # solve for pyvcr and add test to upload binary image
  result = client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="assistants")
  diff = DeepDiff(file_to_upload, result.to_dict(), include_paths=file_to_upload.keys())
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
async def test_files_async_upload(client: OpenAI):
  # TODO: pyvcr not working with uploading binary image file
  # failing when converting it to utf-8 encodings
  # solve for pyvcr and add test to upload binary image
  result = await client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="assistants")
  diff = DeepDiff(file_to_upload, result.to_dict(), include_paths=file_to_upload.keys())
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
def test_files_list(client: OpenAI):
  result = list(client.files.list())
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
async def test_files_async_list(client: OpenAI):
  result = list(await client.files.list())
  assert 1 <= len(result)


@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "file_id"],
  [
    pytest.param("openai", "file-utwlH66iyvPBs61N1uIP2ANE", id="openai"),
    pytest.param("bodhi", "xyz", id="bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
def test_files_retrieve(client: OpenAI, file_id: str):
  result = client.files.retrieve(file_id)
  expected = {**file_to_upload, "id": file_id}
  diff = DeepDiff(expected, result.to_dict(), include_paths=expected.keys())
  assert {} == diff


@pytest.mark.asyncio
@pytest.mark.vcr
@pytest.mark.parametrize(
  ["client", "file_id"],
  [
    pytest.param("async_openai", "file-utwlH66iyvPBs61N1uIP2ANE", id="async_openai"),
    pytest.param("async_bodhi", "xyz", id="async_bodhi", **not_implemented()),
  ],
  indirect=["client"],
)
async def test_files_retrieve_async(client: OpenAI, file_id):
  result = await client.files.retrieve(file_id)
  expected = {**file_to_upload, "id": file_id}
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
def test_files_delete(client: OpenAI):
  file = client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="assistants")
  result = client.files.delete(file.id)
  expected = {"id": file.id, "deleted": True, "object": "file"}
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
async def test_files_delete_async(client: OpenAI):
  file = await client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="assistants")
  result = await client.files.delete(file.id)
  expected = {"id": file.id, "deleted": True, "object": "file"}
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
def test_files_content(client: OpenAI):
  content = open("tests/data/file-to-upload.txt", "r").read()
  file = client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="fine-tune")
  result = client.files.content(file.id)
  assert content == result.text


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
async def test_files_content_async(client: OpenAI):
  content = open("tests/data/file-to-upload.txt", "r").read()
  file = await client.files.create(file=open("tests/data/file-to-upload.txt", "rb"), purpose="fine-tune")
  result = await client.files.content(file.id)
  assert content == result.text
