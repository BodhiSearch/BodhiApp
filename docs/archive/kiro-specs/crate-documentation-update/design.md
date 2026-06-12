# Design Document

## Overview

This design outlines a pure LLM-based analysis approach for examining and updating CLAUDE.md and PACKAGE.md files across the BodhiApp multi-crate workspace. The design emphasizes thorough analysis of existing documentation against current implementation, followed by incremental updates that preserve existing structure and Git history.

## Architecture

### LLM-Based Analysis Methodology
The design follows a comprehensive analysis approach using LLM capabilities to understand and validate documentation:

1. **Documentation Reading**: Read all existing CLAUDE.md and PACKAGE.md files to understand current structure and content
2. **Implementation Analysis**: Thoroughly analyze the actual crate implementation including source code, tests, and dependencies
3. **Synchronization Validation**: Compare documented information against actual implementation to identify discrepancies
4. **Test Utils Analysis**: Examine test_utils modules to understand fixtures, utilities, and testing patterns
5. **Incremental Documentation Updates**: Apply targeted updates that preserve existing quality and structure

### Crate Dependency Hierarchy
The analysis follows the workspace dependency sequence to ensure upstream crates are analyzed before downstream crates that depend on them:

1. **objs** - Foundation domain objects and error types (no dependencies)
2. **services** - Business logic services (depends on: objs)
3. **commands** - CLI command orchestration (depends on: objs, services)
4. **server_core** - HTTP server infrastructure (depends on: objs, services, commands)
5. **auth_middleware** - Authentication middleware (depends on: objs, services, server_core)
6. **routes_oai** - OpenAI API routes (depends on: objs, services, server_core, auth_middleware)
7. **routes_app** - Application API routes (depends on: objs, services, server_core, auth_middleware)
8. **routes_all** - Route composition (depends on: objs, services, server_core, auth_middleware, routes_oai, routes_app)
9. **server_app** - Main server executable (depends on: all routing and infrastructure crates)
10. **lib_bodhiserver** - Embeddable server library (depends on: server_app and all dependencies)
11. **lib_bodhiserver_napi** - Node.js bindings (depends on: lib_bodhiserver and all dependencies)
12. **bodhi/src-tauri** - Desktop application (depends on: lib_bodhiserver and all dependencies)

### Independent Crates
These crates have minimal dependencies and can be analyzed independently:
- **llama_server_proc** - Process management for llama.cpp
- **integration-tests** - End-to-end testing infrastructure
- **xtask** - Build automation and code generation
- **ci_optims** - CI/CD optimization dummy crate
- **errmeta_derive** - Procedural macros for error metadata

### Workspace Analysis Scope
The analysis covers all 17 workspace member crates:
- Core infrastructure: objs, services, server_core
- HTTP routing: routes_oai, routes_app, routes_all  
- Authentication: auth_middleware
- Command orchestration: commands
- Server implementations: server_app, lib_bodhiserver, lib_bodhiserver_napi
- Process management: llama_server_proc
- Testing infrastructure: integration-tests
- Build tooling: xtask, ci_optims
- Procedural macros: errmeta_derive
- Desktop application: bodhi/src-tauri

## Components and Interfaces

### Documentation Analysis Process
The LLM-based analysis follows this systematic approach for each crate:

#### Phase 1: Documentation Reading
- Read existing CLAUDE.md file completely to understand current documentation structure
- Read existing PACKAGE.md file (if present) to understand detailed technical documentation
- Identify the documentation patterns, style, and organizational approach
- Note any references to ai-docs files and cross-crate relationships

#### Phase 2: Implementation Analysis  
- Examine Cargo.toml to understand dependencies, features, and crate metadata
- Analyze source code structure in src/ directory to understand actual implementation
- Review public API and key components to validate documented functionality
- Examine integration points with other crates to verify relationship descriptions

#### Phase 3: Test Utils Deep Analysis
- Locate and analyze test_utils features in Cargo.toml
- Examine test utility modules and fixture creation patterns
- Identify how test_utils reduce testing complexity and provide consistency
- Extract concrete examples from actual test code showing test_utils usage
- Document patterns that enable easy, stable, and consistent testing

#### Phase 4: Synchronization Validation
- Compare documented dependencies against actual Cargo.toml dependencies
- Validate architecture descriptions against actual crate relationships
- Check code examples against current implementation patterns
- Verify that usage patterns match actual API and integration approaches
- Update the docs only if incorrect or missing information, following the style and pattern in the doc
- Update the docs such that it is trackable via git history, that is hold off from rewriting the complete doc, and instead focus on updating it

### Analysis Output Structure
For each crate, the analysis produces:

```
Crate Analysis Report:
├── Current Documentation State
│   ├── CLAUDE.md structure and content assessment
│   ├── PACKAGE.md structure and content assessment (if exists)
│   └── Documentation quality and completeness evaluation
├── Implementation Analysis
│   ├── Actual dependencies and features
│   ├── Source code structure and key components
│   ├── Public API and integration points
│   └── Cross-crate relationships and usage patterns
├── Test Utils Analysis
│   ├── Available test fixtures and their purposes
│   ├── Test utility functions and helper patterns
│   ├── Testing consistency and stability mechanisms
│   └── Concrete usage examples from actual tests
└── Synchronization Issues
    ├── Documentation-implementation mismatches
    ├── Missing critical information
    ├── Outdated examples or descriptions
    └── Recommended updates with rationale
```

## Data Models

### Analysis Framework
The LLM analysis follows structured examination patterns:

#### Documentation Structure Analysis
- Section organization and completeness
- Content accuracy and currency
- Style consistency with other crates
- Cross-references and integration descriptions

#### Implementation Reality Check
- Dependency verification against Cargo.toml
- Feature flag documentation accuracy
- API surface area coverage
- Integration pattern validation

#### Test Utils Pattern Recognition
- Fixture creation and management patterns
- Test utility function organization
- Mock and stub creation approaches
- Test data management strategies

## Error Handling

### Analysis Validation Points
The LLM analysis identifies several categories of issues:

#### Documentation-Implementation Mismatches
- Dependencies documented but not in Cargo.toml
- Features described but not implemented
- Code examples that don't match current API
- Architecture descriptions that don't reflect actual relationships

#### Missing Critical Information
- Undocumented major dependencies
- Missing integration patterns
- Absent test_utils documentation
- Incomplete usage examples

#### Outdated Content
- Deprecated API references
- Old dependency versions
- Obsolete architectural patterns
- Stale code examples

## Testing Strategy

### Test Utils Documentation Enhancement
The design emphasizes comprehensive analysis of test_utils patterns:

#### Fixture Analysis
- Identify all test fixtures and their creation patterns
- Document how fixtures enable consistent test setup
- Show how fixtures reduce test boilerplate and complexity
- Provide concrete examples from actual test implementations

#### Utility Function Analysis  
- Catalog test utility functions and their purposes
- Document how utilities enable stable and reliable testing
- Show patterns for mock creation and test data management
- Extract real usage examples from existing test code

#### Testing Pattern Documentation
- Document testing approaches that leverage test_utils
- Show how test_utils enable thorough testing without hassle
- Provide examples of complex test scenarios made simple
- Document best practices for using test utilities

## Implementation Phases

### Phase 1: Comprehensive Documentation Reading
1. Read all existing CLAUDE.md files across the workspace
2. Read all existing PACKAGE.md files to understand detailed documentation patterns
3. Identify documentation structure conventions and style patterns
4. Note cross-crate references and integration descriptions

### Phase 2: Deep Implementation Analysis
1. Analyze each crate's Cargo.toml for dependencies, features, and metadata
2. Examine source code structure and key implementation components
3. Validate public APIs and integration points
4. Verify cross-crate relationships and usage patterns

### Phase 3: Test Utils Pattern Analysis
1. Identify test_utils features across all crates
2. Analyze test fixture creation and management patterns
3. Examine test utility functions and helper implementations
4. Extract concrete examples from actual test code

### Phase 4: Synchronization and Update
1. Compare documentation against implementation reality
2. Identify gaps, mismatches, and outdated information
3. Create missing CLAUDE.md files for undocumented crates (ci_optims)
4. Apply incremental updates that preserve structure and Git history

## Quality Gates

### Analysis Completeness Gates
- All existing documentation thoroughly read and understood
- All crate implementations comprehensively analyzed
- All test_utils patterns identified and documented
- All synchronization issues cataloged with specific examples

### Update Quality Gates
- Existing documentation structure preserved in all updates
- Git diffs clean and focused on specific improvements
- Documentation style consistency maintained across crates
- No regression in existing documentation quality

### Accuracy Gates
- All documented dependencies match actual Cargo.toml files
- Architecture descriptions reflect current workspace structure
- Code examples validated against current implementation
- Test_utils documentation includes concrete usage examples

## Success Metrics

### Analysis Depth Metrics
- Complete understanding of all existing documentation patterns
- Comprehensive analysis of all 17 workspace crates
- Thorough examination of test_utils across all crates with such features
- Detailed synchronization validation between docs and implementation

### Documentation Accuracy Metrics
- 100% alignment between documented and actual dependencies
- All architecture descriptions reflect current implementation
- Code examples validated against current codebase
- Test_utils documentation enhanced with real usage examples

### Completeness Metrics
- All workspace crates have accurate CLAUDE.md documentation
- Missing CLAUDE.md created for ci_optims crate
- Test_utils patterns documented with concrete examples
- Critical information gaps identified and filled