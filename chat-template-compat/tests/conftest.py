import pytest
import yaml


def pytest_collection_modifyitems(config, items):
  for item in items:
    item.add_marker(pytest.mark.all)


@pytest.fixture(scope="function")
def input(inputs, request):
  result = inputs[request.param]
  for key, value in result.items():
    if isinstance(value, str):
      result[key] = value.rstrip().replace("\\n", "\n").replace("\\s", " ")
  return result


@pytest.fixture(scope="function")
def inputs():
  return inputs_yaml()


def inputs_yaml():
  with open("tests/data/inputs.yaml", "r") as file:
    inputs = yaml.safe_load(file)
  result = {input["id"]: input for input in inputs if "id" in input}
  return result
