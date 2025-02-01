import pytest
from pathlib import Path
import yaml
from transformers import AutoTokenizer
import os
import logging
from typing import Union
from jinja2.exceptions import TemplateError

from .conftest import inputs_yaml


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


# Load test cases and YAML data
def load_test_data():
  current_dir = Path(__file__).parent
  yaml_path = current_dir / "data" / "embedded-repos-with-base.yaml"
  with open(yaml_path, "r") as f:
    return yaml.safe_load(f)


def get_test_cases():
  # Get test cases from inputs_yaml
  return list(inputs_yaml().keys())


# Setup paths
CACHE_DIR = Path(__file__).parent / "data" / "tokenizers"
OUTPUT_DIR = Path(__file__).parent / "data" / "popular"


def sanitize_model_name(model_name: Union[str, list]) -> str:
  """Convert model name to a safe filename"""
  # If model_name is a list, take the first item
  if isinstance(model_name, list):
    model_name = model_name[0]
  return model_name.replace("/", "--")


@pytest.fixture(scope="session", autouse=True)
def setup_directories():
  """Ensure cache and output directories exist"""
  CACHE_DIR.mkdir(parents=True, exist_ok=True)
  OUTPUT_DIR.mkdir(parents=True, exist_ok=True)


@pytest.fixture(scope="session")
def model_data():
  """Load the YAML data once per session"""
  return load_test_data()


@pytest.fixture(params=get_test_cases())
def test_case(request):
  """Fixture to provide test cases"""
  case_id = request.param
  test_data = inputs_yaml()[case_id]
  return case_id, test_data


@pytest.mark.parametrize("model_info", load_test_data())
def test_chat_templates(model_info, test_case, setup_directories):
  """Test chat templates for each model and test case"""
  case_id, case_data = test_case
  messages = case_data["messages"]

  # Extract model info
  repo_id = model_info["id"]
  base_model = model_info["base"]
  template = model_info["template"]

  # Check if output file already exists
  sanitized_base = sanitize_model_name(base_model)
  output_file = OUTPUT_DIR / f"{sanitized_base}--{case_id}.txt"
  
  if output_file.exists():
    logger.info(f"Output file already exists for {base_model} with case {case_id}, skipping...")
    pytest.skip("Output file already exists")

  try:
    # If base_model is a list, use the first item
    tokenizer_model = base_model[0] if isinstance(base_model, list) else base_model
    
    # Load the base model tokenizer
    tokenizer = AutoTokenizer.from_pretrained(tokenizer_model, cache_dir=CACHE_DIR, trust_remote_code=True)

    try:
      output = tokenizer.apply_chat_template(messages, add_generation_prompt=True, tokenize=False)

      # Save output
      with open(output_file, "w", encoding="utf-8") as f:
        f.write(output)

      # Basic validation that output is non-empty
      assert output.strip(), f"Empty output for {base_model} with case {case_id}"

    except Exception as e:
      error_msg = str(e)
      
      # Check if it's a Jinja template error
      if isinstance(e, TemplateError) or "jinja2" in error_msg.lower():
        # Save the error message to file
        with open(output_file, "w", encoding="utf-8") as f:
          f.write(f"JINJA_ERROR: {error_msg}")
        logger.info(f"Saved Jinja error for {base_model} with case {case_id}")
      else:
        # For non-Jinja errors, fail the test
        pytest.fail(f"Error applying chat template for {base_model} with case {case_id}: {error_msg}")

  except Exception as e:
    # Log any exceptions during tokenizer loading
    pytest.fail(f"Failed to load tokenizer for {base_model}: {str(e)}")


if __name__ == "__main__":
  pytest.main([__file__])
