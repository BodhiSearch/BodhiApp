import yaml
from pathlib import Path
import re
from jinja2 import Environment, meta
import logging
from transformers import AutoTokenizer

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)


def load_yaml(file_path):
  with open(file_path, "r") as f:
    return yaml.safe_load(f)


def save_yaml(data, file_path):
  def str_presenter(dumper, data):
    if '\n' in data:
      return dumper.represent_scalar('tag:yaml.org,2002:str', data, style='|')
    return dumper.represent_scalar('tag:yaml.org,2002:str', data)

  yaml.add_representer(str, str_presenter)

  with open(file_path, "w") as f:
    yaml.dump(data, f, sort_keys=False, allow_unicode=True, default_flow_style=False)


def extract_jinja_variables(template_str):
  try:
    env = Environment()
    ast = env.parse(template_str)
    variables = meta.find_undeclared_variables(ast)

    # for consistent output
    return sorted(list(variables))

  except Exception as e:
    logger.error(f"Error parsing template: {str(e)}")
    logger.error(f"Template content: {template_str}")
    return []


def get_token_info(base_model: str, variables: list) -> dict:
  token_info = {}

  if not any(token in variables for token in ['bos_token', 'eos_token']):
    return token_info

  try:
    cache_dir = Path(__file__).parent / "tests" / "data" / "tokenizers"

    logger.info(f"Loading tokenizer for {base_model}")
    tokenizer = AutoTokenizer.from_pretrained(
      base_model,
      cache_dir=cache_dir,
      trust_remote_code=True
    )

    if 'bos_token' in variables:
      token_info['bos_token'] = tokenizer.bos_token
      
    if 'eos_token' in variables:
      token_info['eos_token'] = tokenizer.eos_token
      
    logger.info(f"Found token info: {token_info}")
    
  except Exception as e:
    logger.error(f"Error getting token info for {base_model}: {str(e)}")
  
  return token_info


def main():
  current_dir = Path(__file__).parent
  input_file = current_dir / "tests" / "data" / "embedded-repos-with-base.yaml"
  output_file = current_dir / "tests" / "data" / "embedded-repos-with-vars.yaml"

  logger.info(f"Loading data from {input_file}")
  data = load_yaml(input_file)

  output_data = []
  for entry in data:
    logger.info(f"Processing template for {entry['id']}")
    template = entry["template"]

    variables = extract_jinja_variables(template)
    logger.info(f"Found variables: {variables}")

    new_entry = {
      "id": entry["id"],
      "base": entry["base"],
      "variables": variables
    }

    token_info = get_token_info(entry["base"], variables)
    if token_info:
      new_entry.update(token_info)

    output_data.append(new_entry)

  logger.info(f"Saving results to {output_file}")
  save_yaml(output_data, output_file)
  logger.info("Done!")


if __name__ == "__main__":
  main()
