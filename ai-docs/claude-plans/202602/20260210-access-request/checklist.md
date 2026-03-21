# Access Request Implementation - Checklist

## Phase 0: Keycloak SPI Verification ⏭️ SKIPPED

- ⏭️ Keycloak custom SPI implementation (assumed complete)
- ⏭️ POST /realms/{realm}/bodhi/users/request-access endpoint
- ⏭️ Contract validation (201 Created, 200 OK idempotent, 409 Conflict)

## Phase 1: Database Schema & Domain Objects ✅ COMPLETE

### Migrations
- ✅ `0010_app_client_toolset_configs_drop.up.sql` - Drop old table
- ✅ `0010_app_client_toolset_configs_drop.down.sql` - Rollback
- ✅ `0011_app_access_requests.up.sql` - Create new table with UUID PK
- ✅ `0011_app_access_requests.down.sql` - Rollback
- ✅ Indexes on status and app_client_id

### Domain Objects (objs crate)
- ✅ `crates/objs/src/access_request.rs` created
  - ✅ `AppAccessRequestStatus` enum (Draft, Approved, Denied, Failed)
  - ✅ `AccessRequestFlowType` enum (Redirect, Popup)
  - ✅ `ToolTypeRequest` struct
  - ✅ `ToolApproval` struct (with tool_type, status, toolset_id)
- ✅ `crates/objs/src/lib.rs` - Added exports

### Database Row Type
- ✅ `crates/services/src/db/objs.rs` - `AppAccessRequestRow` struct
  - ✅ All fields: id, app_client_id, flow_type, redirect_uri, status
  - ✅ JSON fields: tools_requested, tools_approved
  - ✅ KC response fields: resource_scope, access_request_scope
  - ✅ Timestamps: expires_at, created_at, updated_at

### API Types (services crate)
- ✅ `AppAccessRequest` struct (request DTO)
- ✅ `AppAccessResponse` struct (response DTO)
- ✅ `AppAccessRequestDetail` struct (polling response)

## Phase 2: Service Layer Implementation ✅ COMPLETE

### Error Types
- ✅ `crates/services/src/access_request_service/error.rs` created
  - ✅ `AccessRequestError` enum with all variants
  - ✅ ErrorMeta trait implementation
  - ✅ From conversions for DbError, AuthServiceError, ToolsetError

### Repository Layer
- ✅ `crates/services/src/db/access_request_repository.rs` created
  - ✅ `AccessRequestRepository` trait
  - ✅ `create()` method
  - ✅ `get()` method
  - ✅ `update_approval()` method
  - ✅ `update_denial()` method
  - ✅ `update_failure()` method
- ✅ `crates/services/src/db/service.rs` - SqliteDbService implementations
  - ✅ All CRUD operations with RETURNING clauses
  - ✅ Proper error handling and type conversions
- ✅ `crates/services/src/db/mod.rs` - Module exports

### Service Layer
- ✅ `crates/services/src/access_request_service/service.rs` created
  - ✅ `AccessRequestService` trait with automock
  - ✅ `DefaultAccessRequestService` implementation
  - ✅ `create_draft()` method with UUID generation
  - ✅ `get_request()` method with expiry checking
  - ✅ `approve_request()` method with KC integration
  - ✅ `deny_request()` method
  - ✅ `build_review_url()` method
  - ✅ `generate_description()` helper for KC consent screen
- ✅ `crates/services/src/access_request_service/mod.rs` - Exports with conditional MockAccessRequestService

### AuthService Integration
- ✅ `crates/services/src/auth_service.rs` modifications
  - ✅ `RegisterAccessRequestConsentResponse` struct
  - ✅ `register_access_request_consent()` method in trait
  - ✅ Implementation in `KeycloakAuthService`
  - ✅ POST to `/realms/{realm}/bodhi/users/request-access`
  - ✅ 409 Conflict handling
  - ✅ Idempotent 200 OK handling

### AppService Integration
- ✅ `crates/services/src/app_service.rs` modifications
  - ✅ `access_request_service()` getter in trait
  - ✅ Field in `DefaultAppService` struct
  - ✅ Constructor parameter
  - ✅ Getter implementation

### AppServiceBuilder Integration
- ✅ `crates/lib_bodhiserver/src/app_service_builder.rs` modifications
  - ✅ Import `AccessRequestService` and `DefaultAccessRequestService`
  - ✅ Field in struct
  - ✅ Initialize to None in constructor
  - ✅ `get_or_build_access_request_service()` method
  - ✅ Called in `build()` method
  - ✅ Passed to `DefaultAppService::new()`
  - ✅ Uses `public_server_url()` for frontend_url

### Test Infrastructure
- ✅ `crates/services/src/test_utils/db.rs` - TestDbService updates
  - ✅ `AccessRequestRepository` implementation with events
  - ✅ Event broadcasting for operation verification
- ✅ `crates/services/src/test_utils/app.rs` - AppServiceStub updates
  - ✅ Field added to struct
  - ✅ Default builder method
  - ✅ Trait implementation method
  - ✅ Import `MockAccessRequestService`

### Code Removal (Clean Cutover)
- ✅ Removed from `auth_service.rs`:
  - ✅ Old `request_access()` method
  - ✅ Old `RequestAccessRequest` struct
  - ✅ Old `RequestAccessResponse` struct
  - ✅ Implementation in `KeycloakAuthService`
  - ✅ Three test functions (lines 1070-1231)
- ✅ Removed from routes_app:
  - ✅ `routes_auth/request_access.rs` (old handler file)
  - ✅ Import from `routes_auth/mod.rs`
  - ✅ Old test file: `routes_auth/tests/request_access_test.rs`

## Phase 3: API Endpoints ✅ MOSTLY COMPLETE

### Route DTOs (routes_app crate)
- ✅ `crates/routes_app/src/routes_apps/access_request.rs` created
  - ✅ `CreateAppAccessRequest` struct (route-specific DTO)
  - ✅ `AppAccessResponseDTO` struct (route-specific response)
  - ✅ Constants: `ENDPOINT_APP_REQUEST_ACCESS`, `ENDPOINT_APP_REQUEST_ACCESS_STATUS`

### Error Types
- ✅ `crates/routes_app/src/routes_apps/error.rs` created
  - ✅ `AppAccessRequestError` enum
  - ✅ `InvalidFlowType` variant
  - ✅ `MissingRedirectUri` variant
  - ✅ `InvalidToolsFormat` variant (for timestamp/JSON errors)
  - ✅ Transparent ServiceError wrapping

### Handlers
- ✅ `create_app_access_request_handler` implementation
  - ✅ Validation: flow_type must be "redirect" or "popup"
  - ✅ Validation: redirect_uri required for "redirect" flow
  - ✅ Call service.create_draft()
  - ✅ Build review URL
  - ✅ Return 201 Created with response
  - ✅ Proper error handling
- ✅ `get_app_access_request_handler` implementation
  - ✅ Call service.get_request()
  - ✅ Parse tools_requested JSON
  - ✅ Parse tools_approved JSON (if present)
  - ✅ Build scopes array from KC response
  - ✅ Convert timestamps using `DateTime::from_timestamp()`
  - ✅ Return AppAccessRequestDetail
  - ✅ Handle NotFound error

### Router Integration
- ✅ `crates/routes_app/src/routes.rs` modifications
  - ✅ Import new handlers
  - ✅ Import new constants
  - ✅ Register POST /apps/request-access route
  - ✅ Register GET /apps/request-access/:id route
  - ✅ Add to optional_auth middleware group

### OpenAPI Documentation
- ✅ `crates/routes_app/src/shared/openapi.rs` modifications
  - ✅ Import new handler path functions
  - ✅ Import new DTO types
  - ✅ Import domain types (ToolTypeRequest, ToolApproval) from objs
  - ✅ Import AppAccessRequestDetail from services
  - ✅ Add schemas to components section
  - ✅ Add handlers to paths section
  - ✅ Remove old request_access_handler references

### Module Structure
- ✅ `crates/routes_app/src/routes_apps/mod.rs` created
  - ✅ Module declarations (access_request, error)
  - ✅ Conditional tests module declaration
  - ✅ Public exports

### Tests
- ⏸️ **PENDING**: Handler tests (THIS IS THE NEXT TASK)
  - ⬜ Create `crates/routes_app/src/routes_apps/tests/` directory
  - ⬜ Create `mod.rs` in tests directory
  - ⬜ Create `access_request_test.rs`
  - ⬜ Test: `test_create_draft_success`
  - ⬜ Test: `test_create_draft_invalid_flow_type`
  - ⬜ Test: `test_create_draft_missing_redirect_uri`
  - ⬜ Test: `test_create_draft_with_popup_flow`
  - ⬜ Test: `test_get_request_success_draft_status`
  - ⬜ Test: `test_get_request_success_approved_status`
  - ⬜ Test: `test_get_request_not_found`
  - ⬜ Test: `test_get_request_expired`
  - ⬜ Test: `test_approved_request_returns_scopes`

## Verification ✅ COMPLETE

### Compilation
- ✅ `cargo check` - Full backend compiles
- ✅ `cargo check -p routes_app` - Routes app compiles
- ✅ `cargo check -p services` - Services compiles
- ✅ `cargo check -p objs` - Domain objects compile

### Test Suite
- ✅ Services library compiles for tests
- ⚠️ Pre-existing `StubNetworkService` error (not related to our changes)
- ⏸️ Handler tests not yet created (NEXT TASK)

## Future Phases (Out of Scope for Current Session)

### Phase 4: Authentication Middleware
- ⬜ Validate `access_request_id` claim in tokens
- ⬜ Middleware to check approved tool instances
- ⬜ Update auth_middleware for access request validation

### Phase 5: Remove Old Endpoint (Already Done)
- ✅ Old handler already removed in Phase 3

### Phase 6: Frontend Review/Approve Page
- ⬜ UI component for reviewing access requests
- ⬜ Tool instance selection interface
- ⬜ Approve/deny buttons

### Phase 7: Comprehensive Integration Tests
- ⬜ End-to-end flow testing
- ⬜ KC integration tests
- ⬜ Multi-service coordination tests

### Phase 8: Frontend Component Tests
- ⬜ React component tests
- ⬜ Playwright E2E tests

### Phase 9: E2E Tests with Real Keycloak
- ⬜ Test against real KC instance
- ⬜ Validate complete OAuth flow

## Current Session Summary

**Completed:**
- ✅ All database migrations and schema
- ✅ All domain objects and DTOs
- ✅ Complete service layer with repository pattern
- ✅ AuthService KC integration method
- ✅ AppService and AppServiceBuilder integration
- ✅ Complete handler implementations
- ✅ Router integration with middleware
- ✅ OpenAPI documentation updates
- ✅ Removed all old code (clean cutover)
- ✅ Full compilation verification

**Pending for Next Session:**
- ⬜ Create comprehensive handler tests
- ⬜ Verify test coverage for error paths
- ⬜ Ensure all edge cases are tested

**Total Progress: ~95% of Phases 0-3 Complete**
