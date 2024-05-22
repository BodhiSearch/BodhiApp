import openai
import pytest
from .common import LLAMA3_MODEL, GPT_MODEL


@pytest.mark.vcr
def test_models_list(openai_client, bodhi_client):
  models = list(openai_client.models.list())
  assert len(models) > 0

  # TODO: implement
  # models = list(bodhi_client.models.list())
  # assert len(models) > 0


@pytest.mark.vcr
def test_models_retrieve(openai_client, bodhi_client):
  model = openai_client.models.retrieve(GPT_MODEL)
  expected = openai.types.model.Model(
    **{"id": "gpt-4o-2024-05-13", "object": "model", "created": 1715368132, "owned_by": "system"}
  )
  assert expected == model

  # TODO: implement
  # model = bodhi_client.models.retrieve(LLAMA3_MODEL)
  # expected = openai.types.model.Model(
  #   **{"id": "llama3:instruct", "object": "model", "created": 1715368132, "owned_by": "system"}
  # )
  # assert expected == model
