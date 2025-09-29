#!/usr/bin/env python3
"""
Create a trimmed OpenAI specification containing only chat completions endpoint and related components.
"""

import yaml
import json
from typing import Set, Dict, Any

def load_chat_completion_schemas() -> Set[str]:
    """Load the list of chat completion related schemas."""
    with open('/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/ai-docs/specs/20250929-openai-rs/chat_completion_schemas.txt', 'r') as f:
        return set(line.strip() for line in f if line.strip())

def create_trimmed_spec(input_path: str, output_path: str):
    """Create a trimmed OpenAI specification with only chat completions components."""
    with open(input_path, 'r') as f:
        spec = yaml.safe_load(f)

    # Get the chat completion schemas
    chat_schemas = load_chat_completion_schemas()

    # Create new trimmed specification
    trimmed_spec = {
        'openapi': spec.get('openapi'),
        'info': {
            'title': 'OpenAI Chat Completions API (Trimmed)',
            'description': 'Trimmed OpenAI API specification containing only chat completions endpoint and related components',
            'version': spec.get('info', {}).get('version', '1.0.0')
        },
        'servers': spec.get('servers', []),
        'security': spec.get('security', []),
        'paths': {},
        'components': {
            'schemas': {},
            'securitySchemes': spec.get('components', {}).get('securitySchemes', {})
        }
    }

    # Extract only the chat completions path
    original_paths = spec.get('paths', {})
    if '/chat/completions' in original_paths:
        trimmed_spec['paths']['/chat/completions'] = original_paths['/chat/completions']

    # Extract only the related schemas
    original_schemas = spec.get('components', {}).get('schemas', {})
    for schema_name in chat_schemas:
        if schema_name in original_schemas:
            trimmed_spec['components']['schemas'][schema_name] = original_schemas[schema_name]

    # Save the trimmed specification
    with open(output_path, 'w') as f:
        yaml.dump(trimmed_spec, f, default_flow_style=False, sort_keys=False, indent=2)

    print(f"Trimmed specification created with:")
    print(f"  - {len(trimmed_spec['paths'])} paths")
    print(f"  - {len(trimmed_spec['components']['schemas'])} schemas")

def main():
    input_path = '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/ai-docs/specs/20250929-openai-rs/specs/openai-full.yml'
    output_path = '/Users/amir36/Documents/workspace/src/github.com/BodhiSearch/BodhiApp/ai-docs/specs/20250929-openai-rs/specs/openai-chat-completions.yml'

    print("Creating trimmed OpenAI specification for chat completions...")
    create_trimmed_spec(input_path, output_path)
    print(f"Trimmed specification saved to: {output_path}")

if __name__ == "__main__":
    main()