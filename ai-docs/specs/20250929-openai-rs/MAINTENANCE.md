# OpenAI Types Maintenance Guide

This document provides comprehensive guidance for maintaining the OpenAI types generation system in BodhiApp.

## Overview

The OpenAI types generation system automatically creates Rust types from OpenAI's official OpenAPI specification with utoipa support. This system consists of:

- **Custom templates** with utoipa::ToSchema derives
- **Trimmed specification** containing only chat completions components
- **Automated generation** via xtask command
- **Integration** with BodhiApp's OpenAPI documentation system

## Key Components

### Files and Directories
- `crates/openai_types/` - Generated types crate
- `xtask/src/openai_types.rs` - Generation command implementation
- `ai-docs/specs/20250929-openai-rs/templates/rust/` - Custom Mustache templates
- `ai-docs/specs/20250929-openai-rs/specs/` - Downloaded and trimmed specifications
- `ai-docs/specs/20250929-openai-rs/scripts/` - Python scripts for spec processing

### Dependencies
- **OpenAPI Generator CLI** (v7.14.0+) - Type generation tool
- **Node.js** (v22+) and npm - For OpenAPI Generator CLI
- **Python 3** - For specification processing scripts

## Routine Maintenance

### When to Regenerate Types

Regenerate OpenAI types when:

1. **OpenAI API Updates**: OpenAI releases new features or changes to chat completions
2. **Bug Fixes**: Issues discovered in generated types or OpenAPI documentation
3. **Template Updates**: Modifications to custom Mustache templates
4. **BodhiApp Integration**: Changes to utoipa or workspace dependencies

### Regeneration Workflow

#### 1. Standard Regeneration
```bash
# Regenerate OpenAI types from latest specification
cargo xtask generate-openai-types

# Verify compilation
cargo build -p openai_types

# Update OpenAPI documentation
cargo run --package xtask openapi

# Update TypeScript client
cd ts-client && npm run generate

# Test integration
cargo test -p openai_types
cargo test -p routes_app
```

#### 2. Manual Regeneration (Advanced)
If the automated process fails, use manual steps:

```bash
# Navigate to project root
cd /path/to/BodhiApp

# Download latest OpenAI specification
curl -o ai-docs/specs/20250929-openai-rs/specs/openai-full.yml \
  https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml

# Create trimmed specification
cd ai-docs/specs/20250929-openai-rs/scripts
python create_trimmed_spec.py

# Generate types with custom templates
cd /path/to/BodhiApp
openapi-generator-cli generate \
  -i ai-docs/specs/20250929-openai-rs/specs/openai-chat-completions.yml \
  -g rust \
  -o crates/openai_types \
  -t ai-docs/specs/20250929-openai-rs/templates/rust \
  --package-name openai_types \
  --additional-properties packageVersion=0.1.0

# Apply manual fixes (see Troubleshooting section)
```

## Specification Updates

### Monitoring OpenAI Changes

OpenAI periodically updates their API specification. Monitor these sources:

1. **Primary Source**: https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml
2. **Alternative Source**: https://raw.githubusercontent.com/openai/openai-openapi/manual_spec/openapi.yaml
3. **OpenAI Documentation**: https://platform.openai.com/docs/api-reference/chat
4. **GitHub Repository**: https://github.com/openai/openai-openapi

### Version Management

The generation system is designed to work with OpenAI API v2.3.0 and OpenAPI 3.1.0 format. When major versions change:

1. Test compatibility with existing templates
2. Update version numbers in generated Cargo.toml
3. Validate breaking changes don't affect BodhiApp integration
4. Update documentation if API patterns change

## Troubleshooting

### Common Issues and Solutions

#### 1. Compilation Errors After Generation

**Problem**: Generated types fail to compile with Rust errors.

**Common causes**:
- Recursive type definitions
- Missing Default implementations
- Invalid serde attributes

**Solutions**:
```bash
# Fix recursive types (typically GrammarFormat)
# Edit crates/openai_types/src/models/grammar_format.rs
# Change: grammar: models::GrammarFormat
# To: grammar: Box<models::GrammarFormat>

# Remove empty Default implementations for union enums
# Search for and remove lines like: impl Default for SomeUnionEnum { fn default() -> Self { todo!() } }

# Format generated code
cargo fmt --package openai_types
```

#### 2. OpenAPI Generator CLI Issues

**Problem**: Command fails with "openapi-generator-cli not found".

**Solution**:
```bash
# Reinstall OpenAPI Generator CLI
npm install -g @openapitools/openapi-generator-cli

# Verify installation
openapi-generator-cli version
```

**Problem**: Generation fails with template errors.

**Solution**:
```bash
# Verify template files exist
ls -la ai-docs/specs/20250929-openai-rs/templates/rust/

# Restore from backup if corrupted
cp ai-docs/specs/20250929-openai-rs/templates/rust/model.mustache.backup \
   ai-docs/specs/20250929-openai-rs/templates/rust/model.mustache
```

#### 3. Specification Download Issues

**Problem**: Unable to download OpenAI specification.

**Solutions**:
```bash
# Try alternative URL
curl -o specs/openai-full.yml \
  https://raw.githubusercontent.com/openai/openai-openapi/manual_spec/openapi.yaml

# Verify file format
head -n 20 specs/openai-full.yml

# Check file size (should be ~2MB)
ls -lh specs/openai-full.yml
```

#### 4. Utoipa Integration Problems

**Problem**: Generated types don't appear in OpenAPI documentation.

**Solutions**:
```bash
# Verify ToSchema derives are present
grep -r "ToSchema" crates/openai_types/src/models/

# Check routes_app integration
grep -A 10 -B 5 "openai_types" crates/routes_app/src/openapi.rs

# Regenerate OpenAPI spec
cargo run --package xtask openapi
```

#### 5. TypeScript Generation Issues

**Problem**: TypeScript client missing OpenAI types.

**Solutions**:
```bash
# Verify OpenAPI spec contains types
grep -c "CreateChatCompletionRequest" crates/routes_app/src/openapi.rs

# Regenerate TypeScript client
cd ts-client
npm run generate
npm run build

# Check generated files
ls -la src/types/types.gen.ts
```

### Validation Checklist

After regeneration, verify:

- [ ] `cargo build -p openai_types` succeeds
- [ ] `cargo test -p openai_types` passes
- [ ] `cargo run --package xtask openapi` generates spec with OpenAI types
- [ ] `grep -c "CreateChatCompletionRequest" crates/routes_app/src/openapi.rs` returns > 0
- [ ] TypeScript client generation includes OpenAI types
- [ ] `cargo test -p routes_app` passes without regressions

## Advanced Maintenance

### Template Customization

When updating Mustache templates in `ai-docs/specs/20250929-openai-rs/templates/rust/`:

1. **Always backup existing templates**:
   ```bash
   cp model.mustache model.mustache.backup
   ```

2. **Test changes with small modifications first**

3. **Validate utoipa imports are preserved**:
   - Line 4: `use utoipa::ToSchema;`
   - All derive macros include `ToSchema`

4. **Document template changes** in this file

### Specification Processing

The system uses Python scripts to trim the full OpenAI specification:

- `extract_chat_schemas.py` - Identifies chat completion dependencies
- `create_trimmed_spec.py` - Creates minimal specification

When modifying these scripts:

1. Test with latest OpenAI specification
2. Verify output contains all required schemas
3. Maintain size efficiency (target <150KB)

### CI Integration Recommendations

For automated regeneration in CI pipelines:

```yaml
- name: Check OpenAI Types
  run: |
    # Download latest spec
    curl -o /tmp/openai-current.yml \
      https://app.stainless.com/api/spec/documented/openai/openapi.documented.yml

    # Compare with stored version
    if ! cmp -s ai-docs/specs/20250929-openai-rs/specs/openai-full.yml /tmp/openai-current.yml; then
      echo "OpenAI specification has changed"
      # Trigger regeneration or create issue
    fi

    # Verify generated types are current
    cargo xtask generate-openai-types --verify-only
```

## Version Compatibility

### Supported Versions
- **OpenAI API**: v2.3.0+
- **OpenAPI Format**: 3.1.0
- **OpenAPI Generator**: 7.14.0+
- **Utoipa**: 5.3.1+
- **Rust**: 1.87.0+

### Breaking Change Management

When OpenAI introduces breaking changes:

1. **Assess Impact**: Review changes to chat completions endpoint
2. **Test Compatibility**: Verify existing BodhiApp integration works
3. **Update Templates**: Modify Mustache templates if needed
4. **Document Changes**: Update this maintenance guide
5. **Coordinate Release**: Plan updates with BodhiApp releases

## Contacts and Resources

### Documentation
- **OpenAI API Reference**: https://platform.openai.com/docs/api-reference
- **OpenAPI Generator**: https://openapi-generator.tech/docs/generators/rust
- **Utoipa Documentation**: https://docs.rs/utoipa/latest/utoipa/

### Internal Resources
- **Implementation Log**: `ai-docs/specs/20250929-openai-rs/impl-log.md`
- **Context File**: `ai-docs/specs/20250929-openai-rs/impl-ctx.md`
- **Project CLAUDE.md**: `/CLAUDE.md` (search for "openai-types")

### Emergency Rollback

If regeneration causes critical issues:

```bash
# Restore from git
git checkout HEAD -- crates/openai_types/

# Or restore from backup if available
cp -r /backup/openai_types/ crates/

# Verify restoration
cargo build -p openai_types
cargo test -p routes_app
```

## Future Enhancements

Potential improvements to consider:

1. **Automated Monitoring**: CI job to detect OpenAI specification changes
2. **Selective Updates**: Only regenerate types when chat completions change
3. **Multiple API Versions**: Support for different OpenAI API versions
4. **Enhanced Validation**: Automated testing of generated types against real API
5. **Template Versioning**: Version control for custom templates

---

**Last Updated**: 2025-09-29
**Maintainer**: BodhiApp Development Team