# Model Prefix Routing - Task Breakdown

## Overview
Implement optional prefix-based routing to support multiple providers serving the same AI models with clear differentiation and namespace separation.

## Phase 1: Domain Model Foundation ✅ **COMPLETED**
**Goal: Extend core data structures to support optional prefixes**

### Task 1.1: Update Domain Models
- [x] Add optional `prefix` field to `ApiAlias` struct
- [x] Update constructors to handle optional prefix
- [x] Maintain serialization compatibility
- [x] Add `matchable_models()` method for efficient prefix matching
- [x] **Test:** Unit tests for prefix field serialization

### Task 1.2: Update Test Utilities
- [x] Update test builders to support prefix field
- [x] Create test fixtures with prefixed models
- [x] Add prefix-specific test helpers
- [x] **Test:** Verify all existing tests pass

## Phase 2: Database Layer ✅ **COMPLETED**
**Goal: Persist prefix configuration with backward compatibility**

### Task 2.1: Create Database Migration
- [x] Create migration `0005_api_model_prefix.up.sql`
- [x] Add nullable `prefix` column to `api_model_aliases`
- [x] Create rollback migration `.down.sql`
- [x] **Test:** Migration up/down testing

### Task 2.2: Update Database Service
- [x] Update `create_api_model_alias` to handle prefix
- [x] Update `update_api_model_alias` for prefix changes
- [x] Add prefix to `list_api_model_aliases` results
- [x] Database operations handle optional prefix correctly
- [x] **Test:** CRUD operations with prefix field

## Phase 3: Service Layer Enhancement ✅ **COMPLETED**
**Goal: Implement prefix-aware routing logic**

### Task 3.1: Enhance DataService Routing
- [x] Update `find_alias` method to use `matchable_models()`
- [x] Implement efficient prefix matching without string parsing
- [x] Simplified routing logic using pre-computed model names
- [x] Maintain backward compatibility for non-prefixed models
- [x] **Test:** Routing with prefixed and non-prefixed models

### Task 3.2: Update Model Router
- [x] ModelRouter delegates to DataService (no changes needed)
- [x] Existing routing logic handles prefixed models via DataService
- [x] Clean separation of concerns maintained
- [x] No conflicts between prefixed/non-prefixed models
- [x] **Test:** Router behavior with various prefix scenarios

### Task 3.3: Update AI API Service
- [x] Strip prefix from model name before API calls in `forward_chat_completion()`
- [x] Maintain request forwarding for non-prefixed models
- [x] Preserve all error handling paths
- [x] Ensure streaming compatibility
- [x] **Test:** Comprehensive API forwarding with prefix removal tests

## Phase 4: HTTP API Layer ✅ **COMPLETED**
**Goal: Expose prefix configuration through REST API**

### Task 4.1: Update Request/Response DTOs
- [x] Add `prefix: Option<String>` to `CreateApiModelRequest`
- [x] Add prefix to `UpdateApiModelRequest`
- [x] Include prefix in `ApiModelResponse`
- [x] Add regex validation pattern `^[a-zA-Z0-9][a-zA-Z0-9_-]*$`
- [x] **Test:** DTO serialization with prefix field

### Task 4.2: Update Route Handlers
- [x] Handle prefix in create endpoint with validation
- [x] Support prefix updates including empty string → None conversion
- [x] Include prefix in list/get responses
- [x] Updated ApiModelResponse::from_alias to include prefix
- [x] **Test:** HTTP endpoints with prefix parameter

### Task 4.3: Update OpenAPI Documentation
- [x] Prefix field automatically included in OpenAPI generation
- [x] Schema validation patterns included in spec
- [x] Examples include prefix field in documentation
- [x] Response schemas include prefix field
- [x] **Test:** OpenAPI spec generation with prefix field

## Phase 5: Frontend Implementation 🔄 **NOT STARTED**
**Goal: Enable prefix configuration in user interface**

### Task 5.1: Update API Model Form
- [ ] Add optional "Model Prefix" input field with data-testid="api-model-prefix"
- [ ] Add checkbox to control prefix inclusion in form submission
- [ ] Implement regex validation matching backend pattern
- [ ] Add helpful placeholder and tooltips explaining prefix purpose
- [ ] Handle empty string → undefined conversion for API calls
- [ ] **Test:** Form validation with prefix field

### Task 5.2: Update Model Display
- [ ] Show prefixed model names in lists
- [ ] Display prefix with visual distinction
- [ ] Update model selector components
- [ ] Handle prefixed models in navigation
- [ ] **Test:** UI display with prefixed models

### Task 5.3: Update Chat Interface
- [ ] Support prefixed model selection
- [ ] Display full prefixed name in chat settings
- [ ] Handle routing with prefixed models
- [ ] Maintain compatibility with non-prefixed models
- [ ] **Test:** Chat flow with prefixed models

## Phase 6: Integration Testing 🔄 **NOT STARTED**
**Goal: Ensure end-to-end functionality**

### Task 6.1: Backward Compatibility Testing
- [ ] Verify existing models work without prefix
- [ ] Test migration with production-like data
- [ ] Ensure no breaking API changes
- [ ] Validate UI works for both modes
- [ ] **Test:** Full regression test suite

### Task 6.2: Prefix Routing Testing
- [ ] Test multiple providers with same model
- [ ] Verify prefix stripping before API calls
- [ ] Test conflict resolution
- [ ] Validate error handling
- [ ] **Test:** End-to-end prefix routing

### Task 6.3: Performance Testing
- [ ] Benchmark routing with prefixes
- [ ] Measure database query impact
- [ ] Test with many prefixed models
- [ ] Validate no degradation
- [ ] **Test:** Load and performance tests

## Phase 7: Documentation 🔄 **NOT STARTED**
**Goal: Provide comprehensive user guidance**

### Task 7.1: User Documentation
- [ ] Document prefix feature purpose
- [ ] Create configuration guide
- [ ] Add troubleshooting section
- [ ] Provide best practices
- [ ] Include migration guide

### Task 7.2: API Documentation
- [ ] Update API reference with prefix field
- [ ] Add code examples
- [ ] Document validation rules
- [ ] Include error responses
- [ ] Update client libraries

## Acceptance Criteria

### Functional Requirements
- [ ] Users can add optional prefix to API model configurations
- [ ] Models with same name from different providers are differentiated
- [ ] Prefix appears in all model displays and selections
- [ ] Routing correctly identifies prefixed models
- [ ] API calls work with prefix stripped from model name

### Non-Functional Requirements
- [ ] Zero breaking changes to existing functionality
- [ ] No performance degradation in model routing
- [ ] Database migration is reversible
- [ ] UI remains intuitive with optional prefix
- [ ] All existing tests continue to pass

### Security Requirements
- [ ] Prefix validation prevents injection attacks
- [ ] API keys remain encrypted regardless of prefix
- [ ] Access control unchanged by prefix addition
- [ ] Audit logs include prefix information
- [ ] No information leakage through prefix

## Progress Tracking

### Completion Status
- Phase 1: Domain Models - ✅ **COMPLETED** (100%)
- Phase 2: Database Layer - ✅ **COMPLETED** (100%) 
- Phase 3: Service Layer - ✅ **COMPLETED** (100%)
- Phase 4: HTTP API - ✅ **COMPLETED** (100%)
- Phase 5: Frontend - 🔄 **IN PROGRESS** (0%)
- Phase 6: Integration Testing - **NOT STARTED** (0%)
- Phase 7: Documentation - 🔄 **IN PROGRESS** (30%)

**Overall Progress: 57% (4/7 phases complete)**

## Risk Items

1. **Backward Compatibility**: Extensive testing required
2. **User Confusion**: Clear UI/UX needed for prefix concept
3. **Performance**: Prefix matching must be efficient
4. **Data Migration**: Must handle existing deployments
5. **Validation Complexity**: Prefix uniqueness across configs

## Dependencies

### Technical Dependencies
- Existing API model feature must be stable
- Database migration system operational
- Frontend routing supports "/" character
- TypeScript types regeneration working

### Testing Dependencies
- Test database available
- Mock API services configured
- UI test framework operational
- Performance testing environment ready

## Definition of Done

Each phase is complete when:
1. All code implemented and reviewed
2. Unit tests written and passing
3. Integration tests verified
4. Documentation updated
5. No regression in existing features
6. Performance benchmarks met
7. Security review completed

## Notes

- Prefix is **optional** - all existing configurations work unchanged
- Feature can be rolled out gradually
- No data migration required for existing models
- UI should make prefix purpose clear to users
- Consider auto-suggestion of prefixes based on provider