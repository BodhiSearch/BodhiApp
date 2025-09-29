# Comprehensive Implementation Plan: async-openai Types Import with utoipa Annotations (Revised)

## Project Structure
```
BodhiApp/
├── crates/
│   └── async-openai/                 # Our processed types crate
│       ├── Cargo.toml
│       ├── Makefile
│       ├── scripts/                  # Processing scripts
│       │   ├── extract_types.py      # Extract types from async-openai
│       │   ├── post_process.py       # Add utoipa annotations using Syn equivalent
│       │   └── trim_openapi.js       # Trim OpenAPI spec to required endpoints
│       └── src/
│           ├── lib.rs
│           ├── chat.rs               # Processed chat types with utoipa
│           ├── embeddings.rs         # Processed embedding types with utoipa
│           └── common.rs             # Common types with utoipa
├── repo-import/
│   └── async-openai/                 # Submodule: github.com/64bit/async-openai
└── ai-docs/
    └── specs/
        └── 20250929-repo-import/
            ├── repo-import-log.md    # Execution log
            ├── repo-import-ctx.md    # Context and insights
            └── repo-import-plan.md   # This plan document
```

## Phase 1: Environment Setup
**Agent: setup-agent**
**Context File**: `ai-docs/specs/20250929-repo-import/repo-import-ctx.md`
**Log File**: `ai-docs/specs/20250929-repo-import/repo-import-log.md`

### Tasks:
1. **Add async-openai as submodule**:
   ```bash
   git submodule add https://github.com/64bit/async-openai.git repo-import/async-openai
   git submodule update --init --recursive
   ```

2. **Create directory structure**:
   ```bash
   mkdir -p crates/async-openai/{src,scripts}
   mkdir -p ai-docs/specs/20250929-repo-import
   ```

3. **Update root Cargo.toml**:
   ```toml
   [workspace]
   members = [
       "crates/objs",
       "crates/services",
       # ... other existing crates ...
       "crates/async-openai",  # Add at the end
   ]
   exclude = ["repo-import/async-openai"]

   [workspace.dependencies]
   # ... existing dependencies ...
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   utoipa = { version = "5.0", features = ["preserve_order"] }
   ```

4. **Create crate Cargo.toml**:
   ```toml
   [package]
   name = "async-openai"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   serde = { workspace = true }
   serde_json = { workspace = true }
   utoipa = { workspace = true }
   ```

5. **Download OpenAI spec**:
   ```bash
   curl -o crates/async-openai/openapi.yaml \
     https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml
   ```

### Verification:
```bash
# Verify workspace configuration
cargo metadata --format-version 1 | jq '.workspace_members' | grep async-openai

# Verify submodule
cd repo-import/async-openai && git status

# Build empty crate to verify setup
cd crates/async-openai && cargo build
```

### Agent Actions:
- Initialize context file with project goals
- Log all setup steps
- Verify submodule checkout successfully
- Document directory structure created
- Run verification commands and log results

## Phase 2: OpenAPI Spec Trimming
**Agent: spec-trimmer-agent**
**Context File**: Reads Phase 1 context, updates with trimmed spec info
**Log File**: Continues from Phase 1 log

### Tasks:
1. **Create trim_openapi.js script**:
   ```javascript
   // crates/async-openai/scripts/trim_openapi.js
   const fs = require('fs');
   const yaml = require('js-yaml');
   const { trimOpenAPI } = require('openapi-endpoint-trimmer');

   // Read the full OpenAPI spec
   const spec = yaml.load(fs.readFileSync('../openapi.yaml', 'utf8'));

   // Trim to only /v1/chat/completions and /v1/embeddings
   const trimmed = trimOpenAPI(spec, {
     paths: ['/v1/chat/completions', '/v1/embeddings']
   });

   // Save trimmed spec
   fs.writeFileSync('../openapi-trim.json', JSON.stringify(trimmed, null, 2));
   console.log('Trimmed spec saved to openapi-trim.json');
   ```

2. **Install dependencies and run**:
   ```bash
   cd crates/async-openai
   npm init -y
   npm install openapi-endpoint-trimmer js-yaml
   node scripts/trim_openapi.js
   ```

### Verification:
```bash
# Verify trimmed spec exists and contains only desired endpoints
cd crates/async-openai
jq '.paths | keys' openapi-trim.json
# Should show only ["/v1/chat/completions", "/v1/embeddings"]

# Count components in trimmed spec
jq '.components.schemas | keys | length' openapi-trim.json
# Log the count for reference
```

### Agent Actions:
- Create trimming script
- Execute trimming operation
- Verify output contains only requested endpoints
- Log component count in trimmed spec
- Update context with trimming results

## Phase 3: Dynamic Type Extraction - Iteration 1
**Agent: type-extractor-agent**
**Context File**: Reads Phase 2 context with trimmed spec
**Log File**: Continues from Phase 2 log

### Tasks:
1. **Create schema analyzer script**:
   ```python
   # crates/async-openai/scripts/analyze_schemas.py
   import json

   def analyze_trimmed_spec():
       """Read openapi-trim.json and list all component schemas"""
       with open('../openapi-trim.json', 'r') as f:
           spec = json.load(f)

       schemas = spec.get('components', {}).get('schemas', {})

       # Create initial extraction list
       extraction_list = []
       for schema_name in schemas.keys():
           # Map OpenAPI schema names to async-openai file locations
           rust_type = map_to_rust_type(schema_name)
           extraction_list.append(rust_type)

       # Save extraction list
       with open('extraction-list.json', 'w') as f:
           json.dump(extraction_list, f, indent=2)

       return extraction_list

   def map_to_rust_type(schema_name):
       """Map OpenAPI schema name to async-openai type location"""
       # Mapping logic based on schema name patterns
       if 'ChatCompletion' in schema_name:
           return {'schema': schema_name, 'file': 'chat.rs', 'type': schema_name}
       elif 'Embedding' in schema_name:
           return {'schema': schema_name, 'file': 'embedding.rs', 'type': schema_name}
       else:
           return {'schema': schema_name, 'file': 'common.rs', 'type': schema_name}
   ```

2. **Extract types based on dynamic list**:
   ```python
   # crates/async-openai/scripts/extract_types.py
   import json
   import os
   import re
   from pathlib import Path

   def extract_types():
       """Extract types from async-openai based on extraction-list.json"""
       with open('extraction-list.json', 'r') as f:
           extraction_list = json.load(f)

       source_dir = Path('../../repo-import/async-openai/async-openai/src/types')
       output_dir = Path('../src')
       output_dir.mkdir(exist_ok=True)

       extracted_types = {}
       missing_types = []

       for item in extraction_list:
           source_file = source_dir / item['file']
           if not source_file.exists():
               missing_types.append(item)
               continue

           # Extract the specific type from the file
           type_content = extract_type_from_file(source_file, item['type'])
           if type_content:
               extracted_types[item['type']] = type_content
           else:
               missing_types.append(item)

       # Save results
       with open('extraction-results.json', 'w') as f:
           json.dump({
               'extracted': list(extracted_types.keys()),
               'missing': missing_types
           }, f, indent=2)

       return extracted_types, missing_types
   ```

3. **Run initial extraction**:
   ```bash
   cd crates/async-openai/scripts
   python analyze_schemas.py
   python extract_types.py
   ```

### Verification:
```bash
# Check extraction results
cd crates/async-openai/scripts
cat extraction-results.json

# If missing types exist, proceed to iteration 2
```

### Agent Actions:
- Analyze trimmed spec for required schemas
- Map schemas to async-openai type locations
- Extract types from source
- Document extracted and missing types
- Update context with extraction results

## Phase 4: Dynamic Type Extraction - Iteration 2+ (Supporting Components)
**Agent: dependency-resolver-agent**
**Context File**: Reads Phase 3 context with missing types
**Log File**: Continues from Phase 3 log

### Tasks:
1. **Identify missing dependencies**:
   ```python
   # crates/async-openai/scripts/resolve_dependencies.py
   import json
   import re

   def find_type_dependencies(type_content):
       """Find other types referenced in the extracted type"""
       # Look for patterns like: field: SomeType, Vec<SomeType>, Option<SomeType>
       pattern = r':\s*(?:Vec<|Option<)?([A-Z][a-zA-Z0-9_]+)'
       matches = re.findall(pattern, type_content)
       return list(set(matches))

   def resolve_missing_types():
       """Iteratively find and extract missing dependency types"""
       with open('extraction-results.json', 'r') as f:
           results = json.load(f)

       iteration = 1
       while results['missing'] and iteration <= 5:  # Max 5 iterations
           print(f"Iteration {iteration}: Resolving {len(results['missing'])} missing types")

           # Try to find missing types in other files
           newly_extracted = search_for_missing_types(results['missing'])

           # Update results
           results['extracted'].extend(newly_extracted)
           results['missing'] = [t for t in results['missing']
                                if t['type'] not in newly_extracted]

           iteration += 1

       # Save final results
       with open('extraction-final.json', 'w') as f:
           json.dump(results, f, indent=2)

       return results
   ```

2. **Run dependency resolution**:
   ```bash
   cd crates/async-openai/scripts
   python resolve_dependencies.py
   ```

### Verification:
```bash
# Create temporary Rust file to test compilation
cd crates/async-openai
cat > src/lib.rs << 'EOF'
// Temporary test file with extracted types
#![allow(dead_code)]
use serde::{Deserialize, Serialize};

// Include extracted types here
EOF

# Try to compile
cargo build 2>&1 | tee build-test.log

# Check for unresolved type errors
grep "cannot find type" build-test.log
```

### Agent Actions:
- Analyze compilation errors for missing types
- Search for missing types in async-openai source
- Iterate until all dependencies resolved or max iterations reached
- Document resolution process
- Update context with final extraction list

## Phase 5: Syn-Based Post-Processing
**Agent: post-processor-agent**
**Context File**: Reads Phase 4 context with complete type list
**Log File**: Continues from Phase 4 log

### Tasks:
1. **Create post-processing script**:
   ```python
   # crates/async-openai/scripts/post_process.py
   import re
   import json
   from pathlib import Path

   def add_utoipa_derive(content):
       """Add utoipa::ToSchema to existing derive macros"""
       # Pattern to match derive attributes
       derive_pattern = r'#\[derive\(([^)]+)\)\]'

       def add_to_derive(match):
           derives = match.group(1)
           # Check if utoipa::ToSchema already present
           if 'ToSchema' in derives or 'utoipa::ToSchema' in derives:
               return match.group(0)
           # Add utoipa::ToSchema
           return f'#[derive({derives}, utoipa::ToSchema)]'

       # Apply transformation
       modified = re.sub(derive_pattern, add_to_derive, content)

       # Add use statement if not present
       if 'use utoipa' not in modified:
           # Add after other use statements
           modified = re.sub(
               r'(use serde::[^;]+;)',
               r'\1\nuse utoipa::ToSchema;',
               modified,
               count=1
           )

       return modified

   def process_all_types():
       """Process all extracted types to add utoipa annotations"""
       src_dir = Path('../src')

       for rust_file in src_dir.glob('*.rs'):
           if rust_file.name == 'lib.rs':
               continue

           content = rust_file.read_text()
           modified = add_utoipa_derive(content)

           # Special handling for untagged enums
           if '#[serde(untagged)]' in modified:
               # Add schema example after untagged attribute
               # This needs careful handling based on specific enum
               pass

           rust_file.write_text(modified)
           print(f"Processed: {rust_file.name}")

       return True
   ```

2. **Run post-processing**:
   ```bash
   cd crates/async-openai/scripts
   python post_process.py
   ```

### Verification:
```bash
# Verify utoipa derives were added
cd crates/async-openai
grep -r "utoipa::ToSchema" src/

# Compile with utoipa
cargo build

# Run clippy to check for issues
cargo clippy
```

### Agent Actions:
- Apply utoipa annotations to all types
- Preserve existing derives and attributes
- Handle special cases (untagged enums)
- Verify successful processing
- Update context with processing results

## Phase 6: Integration and Final Assembly
**Agent: integration-agent**
**Context File**: Reads Phase 5 context
**Log File**: Continues from Phase 5 log

### Tasks:
1. **Organize module structure**:
   ```rust
   // crates/async-openai/src/lib.rs
   //! async-openai types with utoipa annotations

   pub mod chat;
   pub mod embeddings;
   pub mod common;

   // Re-export main types for convenience
   pub use chat::{
       CreateChatCompletionRequest,
       CreateChatCompletionResponse,
       // ... other chat types
   };

   pub use embeddings::{
       CreateEmbeddingRequest,
       CreateEmbeddingResponse,
       // ... other embedding types
   };

   pub use common::{
       Role,
       CompletionUsage,
       // ... other common types
   };
   ```

2. **Create Makefile**:
   ```makefile
   # crates/async-openai/Makefile

   .PHONY: all clean extract process build test verify

   all: extract process build verify

   trim:
   	node scripts/trim_openapi.js

   analyze:
   	python scripts/analyze_schemas.py

   extract: trim analyze
   	python scripts/extract_types.py
   	python scripts/resolve_dependencies.py

   process:
   	python scripts/post_process.py

   build:
   	cargo build

   test:
   	cargo test

   verify: build test
   	@echo "Verification complete"

   clean:
   	rm -rf src/*.rs scripts/*.json
   	rm -f openapi-trim.json
   	cargo clean
   ```

3. **Create integration test**:
   ```rust
   // crates/async-openai/tests/schema_generation.rs
   #[cfg(test)]
   mod tests {
       use async_openai::*;
       use utoipa::OpenApi;

       #[derive(OpenApi)]
       #[openapi(
           components(schemas(
               CreateChatCompletionRequest,
               CreateChatCompletionResponse,
               CreateEmbeddingRequest,
               CreateEmbeddingResponse,
           ))
       )]
       struct ApiDoc;

       #[test]
       fn test_schema_generation() {
           let doc = ApiDoc::openapi();
           let json = serde_json::to_string_pretty(&doc).unwrap();

           // Verify schemas are present
           assert!(json.contains("CreateChatCompletionRequest"));
           assert!(json.contains("CreateEmbeddingRequest"));
       }
   }
   ```

### Verification:
```bash
# Run full build and test cycle
cd crates/async-openai
make all

# Verify OpenAPI generation in main project
cd ../..
cargo run --package xtask openapi
jq '.components.schemas | keys' openapi.json | grep -E "(ChatCompletion|Embedding)"
```

### Agent Actions:
- Create proper module organization
- Set up build automation with Makefile
- Create integration tests
- Verify full pipeline works
- Update context with final status

## Phase 7: Final Validation
**Agent: validation-agent**
**Context File**: Reads all previous context
**Log File**: Final phase log

### Tasks:
1. **Comprehensive testing**:
   ```bash
   # Full workspace build
   cargo build --workspace

   # Run all tests
   cargo test --workspace

   # Check that routes_app can use the types
   cd crates/routes_app
   # Add test import in a file
   echo "use async_openai::CreateChatCompletionRequest;" >> src/test_import.rs
   cargo check
   ```

2. **OpenAPI spec validation**:
   ```bash
   # Generate full OpenAPI spec
   cargo run --package xtask openapi

   # Validate spec structure
   npx @apidevtools/swagger-cli validate openapi.json
   ```

3. **Documentation generation**:
   ```bash
   # Generate docs to ensure all types are properly documented
   cargo doc --no-deps -p async-openai
   ```

### Verification Checklist:
- [ ] All required types extracted from async-openai
- [ ] utoipa::ToSchema added to all types
- [ ] Code compiles without errors
- [ ] Tests pass
- [ ] OpenAPI schema includes new types
- [ ] Documentation generates correctly

### Agent Actions:
- Run comprehensive validation suite
- Document any issues found
- Create final summary in context
- Mark project as complete or list remaining issues

## Agent Coordination

### Context File Structure (`repo-import-ctx.md`)
```markdown
# async-openai Import Context

## Current Phase: [Phase Number]
## Status: [In Progress/Complete/Failed]

## Phase Results
### Phase 1: Environment Setup
- Submodule added: [Yes/No]
- Directories created: [List]
- Cargo.toml updated: [Yes/No]

### Phase 2: Spec Trimming
- Endpoints in trimmed spec: [Count]
- Components in trimmed spec: [Count]

### Phase 3-4: Type Extraction
- Types to extract: [Count]
- Successfully extracted: [List]
- Missing after iterations: [List]
- Iterations performed: [Number]

### Phase 5: Post-Processing
- Types processed: [Count]
- utoipa annotations added: [Yes/No]
- Special cases handled: [List]

### Phase 6: Integration
- Compilation status: [Success/Failed]
- Test results: [Pass/Fail]

## Insights and Issues
- [Any important discoveries]
- [Problems encountered and solutions]
```

### Log File Structure (`repo-import-log.md`)
```markdown
# async-openai Import Execution Log

## [Timestamp] Phase 1: Environment Setup
- Created directory: crates/async-openai
- Added submodule: repo-import/async-openai
- Updated Cargo.toml workspace
- Verification: cargo build succeeded

## [Timestamp] Phase 2: Spec Trimming
- Downloaded OpenAPI spec: 3.5MB
- Trimmed to 2 endpoints
- Resulting components: 45 schemas
- Saved to: openapi-trim.json

[Continue for each phase with specific actions and results]
```

## Error Recovery Strategy

Each agent should:
1. Read context file to understand current state
2. Check for partial completion markers
3. Verify previous phase outputs exist
4. Re-run only failed steps
5. Update context with recovery actions

## Success Criteria

1. ✅ Only components from trimmed spec are processed
2. ✅ All referenced types successfully extracted
3. ✅ utoipa::ToSchema added while preserving builders
4. ✅ Each phase has verification step
5. ✅ Final crate compiles and tests pass
6. ✅ OpenAPI generation includes new schemas

## Timeline Estimate

- Phase 1: 30 minutes (setup)
- Phase 2: 20 minutes (trimming)
- Phase 3-4: 1.5 hours (extraction with iterations)
- Phase 5: 45 minutes (post-processing)
- Phase 6: 30 minutes (integration)
- Phase 7: 20 minutes (validation)
- **Total**: ~4 hours