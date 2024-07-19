import pytest

from .common import mark_bodhi, mark_ollama

@pytest.mark.vcr
@pytest.mark.parametrize(
  "client",
  [
    pytest.param("ollama", id="ollama", **mark_ollama()),
    pytest.param("ollama_bodhi", id="ollama_bodhi", **mark_bodhi()),
  ],
  indirect=["client"],
)
def test_models_list(client):
  models = list(client.list())
  assert len(models) > 0
