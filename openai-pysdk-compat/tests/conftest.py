import os
import random
import string
from typing import Any, Dict, List, Union

import pytest
from openai import AsyncOpenAI, OpenAI

FILTER_RESPONSE_HEADERS = ["Set-Cookie", "openai-organization"]
OPENAI_API_KEY = "OPENAI_API_KEY"
OPENAI_BASE_URL = "OPENAI_BASE_URL"
BODHI_BASE_URL = "BODHI_BASE_URL"
BODHI_API_KEY = "BODHI_API_KEY"


def pytest_collection_modifyitems(config, items):
  for item in items:
    item.add_marker(pytest.mark.all)


@pytest.fixture(scope="function")
def openai_client() -> OpenAI:
  args = {"api_key": os.environ.get(OPENAI_API_KEY), "base_url": os.environ.get(OPENAI_BASE_URL)}
  args = {k: v for k, v in args.items() if v is not None}
  client = OpenAI(**args)
  return client


@pytest.fixture(scope="function")
def async_openai_client() -> AsyncOpenAI:
  args = {"api_key": os.environ.get(OPENAI_API_KEY), "base_url": os.environ.get(OPENAI_BASE_URL)}
  args = {k: v for k, v in args.items() if v is not None}
  return AsyncOpenAI(**args)


@pytest.fixture(scope="function")
def bodhi_client() -> OpenAI:
  api_key = os.environ.get(BODHI_API_KEY)
  base_url = os.environ.get(BODHI_BASE_URL)
  client = OpenAI(api_key=api_key, base_url=base_url)
  return client


@pytest.fixture(scope="function")
def async_bodhi_client() -> AsyncOpenAI:
  api_key = os.environ.get(BODHI_API_KEY)
  base_url = os.environ.get(BODHI_BASE_URL)
  client = AsyncOpenAI(api_key=api_key, base_url=base_url)
  return client


@pytest.fixture(scope="function")
def api_clients(
  openai_client, async_openai_client, bodhi_client, async_bodhi_client
) -> Dict[str, Union[OpenAI, AsyncOpenAI]]:
  return {
    "openai": openai_client,
    "async_openai": async_openai_client,
    "bodhi": bodhi_client,
    "async_bodhi": async_bodhi_client,
  }


@pytest.fixture(scope="function")
def client(api_clients, request) -> Union[OpenAI, AsyncOpenAI]:
  return api_clients[request.param]


@pytest.fixture(scope="function")
def openai_cli() -> List[str]:
  api_key = os.environ.get(OPENAI_API_KEY)
  base_url = os.environ.get(OPENAI_BASE_URL)
  return [
    "openai",
    "-b",
    base_url,
    "-k",
    api_key,
    "-t",
    "openai",
  ]


@pytest.fixture(scope="function")
def bodhi_cli() -> List[str]:
  api_key = os.environ.get(BODHI_API_KEY)
  base_url = os.environ.get(BODHI_BASE_URL)
  return [
    "openai",
    "-b",
    base_url,
    "-k",
    api_key,
    "-t",
    "openai",
  ]


@pytest.fixture(scope="function")
def cli_args(openai_cli, bodhi_cli) -> Dict[str, List[str]]:
  return {"openai_cli": openai_cli, "bodhi_cli": bodhi_cli}


@pytest.fixture
def cli_arg(cli_args, request) -> List[str]:
  return cli_args[request.param]


@pytest.fixture(scope="session")
def vcr_config() -> Dict[str, Any]:
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


@pytest.fixture(scope="function")
def random_suffix():
  return "".join(random.choices(string.ascii_letters + string.digits, k=4))
