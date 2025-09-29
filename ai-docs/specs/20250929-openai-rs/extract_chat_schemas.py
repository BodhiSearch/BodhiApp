#!/usr/bin/env python3
"""
Extract all schema components related to chat completions from OpenAI spec.
This script identifies and extracts only the components needed for v1/chat/completions endpoint.
"""

import yaml
import re
from typing import Set, Dict, Any

def find_schema_references(schema: Dict[str, Any], visited: Set[str] = None) -> Set[str]:
    """Recursively find all schema references in a schema definition."""
    if visited is None:
        visited = set()

    refs = set()

    if isinstance(schema, dict):
        for key, value in schema.items():
            if key == '$ref' and isinstance(value, str):
                # Extract schema name from reference
                ref_name = value.split('/')[-1]
                if ref_name not in visited:
                    refs.add(ref_name)
                    visited.add(ref_name)
            elif isinstance(value, (dict, list)):
                refs.update(find_schema_references(value, visited))
    elif isinstance(schema, list):
        for item in schema:
            if isinstance(item, (dict, list)):
                refs.update(find_schema_references(item, visited))

    return refs

def extract_chat_completion_schemas(spec_path: str) -> Dict[str, Any]:
    """Extract all schemas related to chat completions."""
    with open(spec_path, 'r') as f:
        spec = yaml.safe_load(f)

    schemas = spec.get('components', {}).get('schemas', {})

    # Start with the main chat completion schemas
    chat_schemas = {
        'CreateChatCompletionRequest',
        'CreateChatCompletionResponse',
        'CreateChatCompletionStreamResponse',
        'ChatCompletionList'
    }

    # Track all found schemas to avoid infinite recursion
    all_found_schemas = set(chat_schemas)
    to_process = list(chat_schemas)

    # Recursively find all referenced schemas
    while to_process:
        schema_name = to_process.pop(0)
        if schema_name in schemas:
            schema_def = schemas[schema_name]
            refs = find_schema_references(schema_def)

            for ref in refs:
                if ref not in all_found_schemas and ref in schemas:
                    all_found_schemas.add(ref)
                    to_process.append(ref)

    # Extract the found schemas
    extracted_schemas = {}
    for schema_name in all_found_schemas:
        if schema_name in schemas:
            extracted_schemas[schema_name] = schemas[schema_name]

    return extracted_schemas

def main():
    spec_path = '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/ai-docs/specs/20250929-openai-rs/specs/openai-full.yml'

    print("Extracting chat completion schemas...")
    chat_schemas = extract_chat_completion_schemas(spec_path)

    print(f"Found {len(chat_schemas)} related schemas:")
    for schema_name in sorted(chat_schemas.keys()):
        print(f"  - {schema_name}")

    # Save the list to a file for reference
    output_path = '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/ai-docs/specs/20250929-openai-rs/chat_completion_schemas.txt'
    with open(output_path, 'w') as f:
        for schema_name in sorted(chat_schemas.keys()):
            f.write(f"{schema_name}\n")

    print(f"Schema list saved to: {output_path}")

if __name__ == "__main__":
    main()