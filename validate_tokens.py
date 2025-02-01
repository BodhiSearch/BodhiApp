import yaml
from typing import Dict, List

def validate_tokens(data: List[Dict]) -> List[str]:
    errors = []
    
    for item in data:
        base = item.get('base', 'NO_BASE_SPECIFIED')
        variables = item.get('variables', [])
        
        # Check bos_token
        has_bos = 'bos_token' in item
        has_bos_var = 'bos_token' in variables
        if has_bos != has_bos_var:
            if has_bos:
                errors.append(f"Model {base}: has bos_token value but missing from variables")
            else:
                errors.append(f"Model {base}: has bos_token in variables but no value defined")
                
        # Check eos_token
        has_eos = 'eos_token' in item
        has_eos_var = 'eos_token' in variables
        if has_eos != has_eos_var:
            if has_eos:
                errors.append(f"Model {base}: has eos_token value but missing from variables")
            else:
                errors.append(f"Model {base}: has eos_token in variables but no value defined")
    
    return errors

def main():
    # Read the YAML file
    with open('chat-template-compat/tests/data/embedded-repos-with-vars.yaml', 'r') as file:
        data = yaml.safe_load(file)
    
    # Validate and get errors
    errors = validate_tokens(data)
    
    # Print results
    if errors:
        print("\nFound inconsistencies:")
        for error in errors:
            print(f"- {error}")
        print(f"\nTotal issues found: {len(errors)}")
    else:
        print("No inconsistencies found!")

if __name__ == "__main__":
    main() 