import os
import random
import string
from typing import Any, Dict, List, Union

import pytest
import yaml
from ollama import Client
from openai import AsyncOpenAI, OpenAI

FILTER_RESPONSE_HEADERS = ["Set-Cookie", "openai-organization"]
OPENAI_API_KEY = "OPENAI_API_KEY"
OPENAI_BASE_URL = "OPENAI_BASE_URL"
BODHI_BASE_URL = "BODHI_BASE_URL"
BODHI_HOST = "BODHI_HOST"
BODHI_API_KEY = "BODHI_API_KEY"
OLLAMA_BASE_URL = "OLLAMA_BASE_URL"
OLLAMA_API_KEY = "OLLAMA_API_KEY"


class MultilineDumper(yaml.Dumper):
  def ignore_aliases(self, data):
    return True


def str_presenter(dumper, data):
  if len(data.splitlines()) > 1:
    return dumper.represent_scalar("tag:yaml.org,2002:str", data, style="|")
  return dumper.represent_scalar("tag:yaml.org,2002:str", data)


MultilineDumper.add_representer(str, str_presenter)


class MultlilineSerializer:
  def serialize(self, cassette_dict):
    return yaml.dump(cassette_dict, Dumper=MultilineDumper)

  def deserialize(self, cassette_string):
    return yaml.safe_load(cassette_string)


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
def ollama_client() -> Client:
  client = Client(host=os.environ.get(OLLAMA_BASE_URL))
  return client


@pytest.fixture(scope="function")
def ollama_bodhi_client() -> Client:
  client = Client(host=os.environ.get(BODHI_HOST))
  return client


@pytest.fixture(scope="function")
def api_clients(
  openai_client, async_openai_client, bodhi_client, async_bodhi_client, ollama_client, ollama_bodhi_client
) -> Dict[str, Union[OpenAI, AsyncOpenAI]]:
  return {
    "openai": openai_client,
    "async_openai": async_openai_client,
    "bodhi": bodhi_client,
    "async_bodhi": async_bodhi_client,
    "ollama": ollama_client,
    "ollama_bodhi": ollama_bodhi_client,
  }


@pytest.fixture(scope="function")
def client(api_clients, request) -> Union[OpenAI, AsyncOpenAI, Client]:
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
    "serializer": "yaml",
  }


def pytest_recording_configure(config, vcr):
  vcr.register_serializer("yaml", MultlilineSerializer())


def before_record_response(response):
  for header in FILTER_RESPONSE_HEADERS:
    if header in response["headers"]:
      del response["headers"][header]
  return response


@pytest.fixture(scope="function")
def random_suffix():
  return "".join(random.choices(string.ascii_letters + string.digits, k=4))
