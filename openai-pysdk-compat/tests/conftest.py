import os

import pytest
from openai import AsyncOpenAI, OpenAI

FILTER_RESPONSE_HEADERS = ["Set-Cookie", "openai-organization"]


def pytest_collection_modifyitems(config, items):
  for item in items:
    item.add_marker(pytest.mark.all)


@pytest.fixture(scope="function")
def openai_client():
  args = {"api_key": os.environ.get("OPENAI_API_KEY"), "base_url": os.environ.get("OPENAI_URL")}
  args = {k: v for k, v in args.items() if v is not None}
  client = OpenAI(**args)
  return client


@pytest.fixture(scope="function")
def async_openai_client():
  args = {"api_key": os.environ.get("OPENAI_API_KEY"), "base_url": os.environ.get("OPENAI_URL")}
  args = {k: v for k, v in args.items() if v is not None}
  return AsyncOpenAI(**args)


@pytest.fixture(scope="function")
def bodhi_client():
  api_key = os.environ.get("BODHI_API_KEY")
  base_url = os.environ.get("BODHI_URL")
  client = OpenAI(api_key=api_key, base_url=base_url)
  return client


@pytest.fixture(scope="function")
def async_bodhi_client():
  api_key = os.environ.get("BODHI_API_KEY")
  base_url = os.environ.get("BODHI_URL")
  client = AsyncOpenAI(api_key=api_key, base_url=base_url)
  return client


@pytest.fixture(scope="function")
def api_clients(openai_client, async_openai_client, bodhi_client, async_bodhi_client):
  return {
    "openai": openai_client,
    "async_openai": async_openai_client,
    "bodhi": bodhi_client,
    "async_bodhi": async_bodhi_client,
  }


@pytest.fixture(scope="function")
def client(api_clients, request):
  return api_clients[request.param]


@pytest.fixture(scope="session")
def vcr_config():
  return {
    "filter_headers": ["authorization"],
    "decode_compressed_response": True,
    "ignore_localhost": True,
    "before_record_response": before_record_response,
  }


def before_record_response(response):
  for header in FILTER_RESPONSE_HEADERS:
    if header in response["headers"]:
      del response["headers"][header]
  return response
