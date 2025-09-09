# /task-md

Generate a phase-wise actionable tasks.md file from an existing plan.md, following the crate dependency chain and project workflow patterns.

## Usage
```
/task-md <folder-name>
```

If no folder name is provided, uses `unknown-feature` as the default.

## Instructions

Read the plan.md from `ai-docs/specs/<folder-name>/plan.md` and transform it into an executable tasks.md file that:

1. **Follows the crate dependency chain** for proper build order
  - objs
  - services
  - commands
  - server_core
  - auth_middleware
  - routes_oai
  - routes_app
  - routes_all
  - server_app
  - lib_bodhiserver
  - lib_bodhiserver_napi
  - bodhi
2. **Creates actionable task items** with clear verification steps  
3. **Includes specific test commands** for each phase
4. **Maps implementation phases** to the appropriate crates
5. **Provides progress tracking** with checkboxes and status indicators

## Crate Dependency Chain

The project has a linear dependency structure that must be followed:
```
objs → services → commands → server_core → auth_middleware → 
routes_oai → routes_app → routes_all → server_app → 
lib_bodhiserver → lib_bodhiserver_napi → bodhi → integration-tests
```

Changes should flow from lowest to highest dependency level to avoid compilation breaks.

## Task Generation Structure

### Document Header
```markdown
# [Feature Name] - Task Breakdown

## Overview
[Brief description of what needs to be implemented]

## Progress Summary
- [ ] Phase 1: [Name] - Not started
- [ ] Phase 2: [Name] - Not started
[etc...]
```

### Phase Template
For each implementation phase from the plan.md, generate:

```markdown
## Phase N: [Phase Name] ([Status])
**Goal: [Clear objective statement from plan]**

**Files Modified:**
- `path/to/file1.rs` - [Brief description]
- `path/to/file2.rs` - [Brief description]

### Task N.1: [Task Name]
- [ ] [Specific action item with file path]
- [ ] [Another action item]
- [ ] **Test:** [Test requirement description]

### Task N.2: [Task Name]
- [ ] [Action items...]
- [ ] **Test:** [Test requirement]

### Commands to Run
```bash
# Verify compilation
cargo check -p [crate-name]
cargo check -p [crate-name] --tests

# Build tests
cargo build -p [crate-name] --tests

# Run tests
cargo test -p [crate-name]

# Format code
cargo fmt -p [crate-name]
```
```

## Phase Mapping Guidelines

### Phase 1: Domain/Objects Layer
- **Crate:** `objs`
- **Typical Tasks:**
  - Create/modify structs and enums
  - Update serialization/deserialization
  - Add validation logic
  - Update test utilities
- **Key Files:** `crates/objs/src/*.rs`

### Phase 2: Services Layer
- **Crate:** `services`
- **Typical Tasks:**
  - Database migrations (if needed)
  - Service trait modifications
  - Business logic implementation
  - Add service methods
- **Key Files:** `crates/services/src/*.rs`, `crates/services/migrations/*.sql`
- **Special:** If database changes, include migration commands

### Phase 3: Server Core Layer
- **Crate:** `server_core`
- **Typical Tasks:**
  - Router updates
  - Middleware changes
  - Core server logic
- **Key Files:** `crates/server_core/src/*.rs`

### Phase 4: HTTP Routes Layer
- **Crates:** `routes_oai`, `routes_app`, `routes_all`
- **Typical Tasks:**
  - New API endpoints
  - Request/response DTOs
  - Route handlers
  - OpenAPI documentation
- **Key Files:** `crates/routes_*/src/*.rs`
- **Special:** Triggers type regeneration

### Phase 5: Frontend Implementation
- **Crate:** `bodhi` (Next.js app)
- **Typical Tasks:**
  - React components
  - API integration
  - UI/UX updates
  - State management
- **Key Files:** `crates/bodhi/src/**/*.tsx`
- **Special Commands:**
```bash
# Regenerate TypeScript types (after API changes)
make ts-client

# Run frontend tests
cd crates/bodhi && npm run test

# Rebuild UI (required after changes)
make rebuild.ui
```

### Phase 6: Integration Testing
- **Crate:** `lib_bodhiserver_napi`
- **Typical Tasks:**
  - Playwright tests
  - End-to-end scenarios
  - API integration tests
- **Key Files:** `crates/lib_bodhiserver_napi/tests-js/**/*.spec.mjs`
- **Commands:**
```bash
cd crates/lib_bodhiserver_napi && npm run test
```

## Task Item Patterns

### Action Item Format
- Use clear action verbs: Create, Update, Modify, Add, Remove, Implement, Fix, Refactor
- Include specific file paths
- Reference method/function names where applicable
- Keep items atomic and testable

### Examples:
```markdown
- [ ] Create `ApiModelAlias` struct in `crates/objs/src/api_model_alias.rs`
- [ ] Update `find_alias()` method in `crates/services/src/data_service.rs` to handle new enum
- [ ] Add database migration `0005_add_prefix_column.up.sql` 
- [ ] Implement `test_prompt()` handler in `crates/routes_app/src/routes_api_models.rs`
- [ ] **Test:** Verify serialization round-trip for new struct
```

## Progress Indicators

Use these consistently throughout the document:
- `[ ]` - Not started
- `[x]` or `✅` - Completed
- `⏳` - In progress
- `❌` - Blocked or failed
- `🔄` - Needs revision
- `🟡` - Partially complete

## Additional Sections to Include

### Acceptance Criteria
Extract from plan.md's success criteria and format as:
```markdown
## Acceptance Criteria

### Functional Requirements
- [ ] [Requirement 1]
- [ ] [Requirement 2]

### Non-Functional Requirements
- [ ] Performance targets met
- [ ] No regression in existing features
- [ ] Documentation updated
```

### Dependencies
```markdown
## Dependencies

### Technical Dependencies
- [List from plan.md]

### Testing Dependencies  
- Test database available
- Mock services configured
- Environment variables set
```

### Definition of Done
```markdown
## Definition of Done

Each phase is complete when:
1. All code implemented and reviewed
2. Unit tests written and passing
3. Integration tests verified
4. Documentation updated
5. No regression in existing features
```

### Risk Items
Extract any risks or concerns from plan.md:
```markdown
## Risk Items

1. **[Risk Name]**: [Description and mitigation]
2. **[Risk Name]**: [Description and mitigation]
```

## Special Considerations

### Database Migrations
When plan includes database changes:
```markdown
### Task X.X: Database Migration
- [ ] Create migration file `migrations/NNNN_description.up.sql`
- [ ] Create rollback file `migrations/NNNN_description.down.sql`
- [ ] Test migration up/down locally
- [ ] **Test:** Verify schema changes with test data
```

### API Changes
When routes or DTOs change:
```markdown
### Post-API Change Tasks
- [ ] Regenerate OpenAPI spec: `cargo run --package xtask openapi`
- [ ] Regenerate TypeScript types: `make ts-client`
- [ ] Update frontend to use new types
- [ ] **Test:** Verify type compatibility
```

### Frontend Changes
After any frontend modifications:
```markdown
### Frontend Build Tasks
- [ ] Run frontend tests: `cd crates/bodhi && npm run test`
- [ ] Rebuild UI: `make rebuild.ui`
- [ ] Run Playwright tests: `cd crates/lib_bodhiserver_napi && npm run test`
- [ ] **Test:** Verify UI changes in browser
```

## Output Location

Create the tasks.md file at:
```
ai-docs/specs/<folder-name>/tasks.md
```

## Generation Guidelines

1. **Preserve Technical Details**: Include specific file paths, method names, and technical requirements from plan.md
2. **Make Tasks Actionable**: Each task should be a clear, executable step
3. **Include Verification**: Every task group should have test requirements
4. **Follow Build Order**: Respect the crate dependency chain
5. **Add Helper Commands**: Include all necessary commands for testing and verification
6. **Track Progress**: Use consistent checkbox format for easy progress tracking

Remember: The goal is to create a step-by-step execution plan that any developer can follow to implement the feature described in the plan.md, with clear verification steps at each phase.