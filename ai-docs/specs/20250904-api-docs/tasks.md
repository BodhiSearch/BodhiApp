# API Documentation Enhancement Tasks

## Overview

This document provides a comprehensive, itemized task list for enhancing our utoipa-based OpenAPI documentation. Tasks are organized into global infrastructure improvements followed by handler-specific enhancements.

## Phase 1: Global Infrastructure Enhancements

### Task 1.1: Enhanced Security Scheme Documentation
**File**: `crates/routes_app/src/openapi.rs`
**Priority**: High
**Estimated Time**: 2-3 hours

#### Subtasks:
- [ ] **1.1.1**: Enhance `bearer_auth` security scheme with detailed description
  - Add comprehensive description explaining JWT token format
  - Include examples of how to obtain tokens
  - Document token format requirements
- [ ] **1.1.2**: Enhance `session_auth` security scheme
  - Add description for session-based authentication
  - Document cookie-based authentication flow
- [ ] **1.1.3**: Add security scopes documentation
  - Document available scopes: `scope_token_user`, `scope_token_power_user`, `resource_user`, `resource_power_user`
  - Add scope descriptions and permissions mapping
- [ ] **1.1.4**: Update OpenAPIEnvModifier implementation
  - Enhance security scheme insertion with better descriptions
  - Add scope-based security requirements

**Expected Changes**:
```rust
// Enhanced security scheme with scopes and detailed descriptions
components.security_schemes.insert(
  "bearer_auth".to_string(),
  SecurityScheme::Http(
    HttpBuilder::default()
      .scheme(HttpAuthScheme::Bearer)
      .bearer_format("JWT")
      .description(Some(
        "JWT token obtained from /bodhi/v1/tokens endpoint. Include as: Authorization: Bearer <token>".to_string(),
      ))
      .build(),
  ),
);
```

### Task 1.2: Standardized Response Schema Enhancement
**File**: `crates/objs/src/error.rs` and related error types
**Priority**: High
**Estimated Time**: 2-3 hours

#### Subtasks:
- [ ] **1.2.1**: Enhance `OpenAIApiError` schema with better examples
  - Add comprehensive examples for different error types
  - Include field-level descriptions
  - Add validation constraints where applicable
- [ ] **1.2.2**: Standardize error response examples across all handlers
  - Create consistent error response patterns
  - Ensure all handlers use proper error response schemas
- [ ] **1.2.3**: Add success response example templates
  - Create reusable success response examples
  - Standardize pagination response examples

### Task 1.3: Enhanced Parameter Validation Framework
**File**: `crates/routes_app/src/api_dto.rs`
**Priority**: Medium
**Estimated Time**: 2 hours

#### Subtasks:
- [ ] **1.3.1**: Enhance `PaginationSortParams` with validation constraints
  - Add `minimum`/`maximum` constraints for page and page_size
  - Add `pattern` constraint for sort_order
  - Include comprehensive field descriptions and examples
- [ ] **1.3.2**: Create parameter validation patterns for reuse
  - Document common parameter validation patterns
  - Create reusable parameter validation examples

**Expected Changes**:
```rust
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationSortParams {
  #[schema(minimum = 1, example = 1, description = "Page number (1-based)")]
  pub page: usize,
  
  #[schema(minimum = 1, maximum = 100, example = 30, description = "Items per page (max 100)")]
  pub page_size: usize,
  
  #[schema(example = "updated_at", description = "Field to sort by")]
  pub sort: Option<String>,
  
  #[schema(pattern = "^(asc|desc)$", example = "desc", description = "Sort order")]
  pub sort_order: String,
}
```

## Phase 2: Handler-Specific Enhancements

### Task Group 2.1: System Handlers
**Files**: `crates/routes_app/src/routes_setup.rs`
**Priority**: Medium
**Estimated Time**: 4-5 hours

#### Task 2.1.1: app_info_handler Enhancement
- [ ] **2.1.1.1**: Add operation summary and description
- [ ] **2.1.1.2**: Enhance `AppInfo` schema with field descriptions
- [ ] **2.1.1.3**: Add comprehensive response examples
- [ ] **2.1.1.4**: Add validation constraints to schema fields

#### Task 2.1.2: setup_handler Enhancement
- [ ] **2.1.2.1**: Add operation summary and description
- [ ] **2.1.2.2**: Enhance `SetupRequest` schema with validation
  - Add `min_length` constraint for name field
  - Add field descriptions and examples
- [ ] **2.1.2.3**: Enhance `SetupResponse` schema
- [ ] **2.1.2.4**: Add comprehensive error response examples

#### Task 2.1.3: ping_handler Enhancement
- [ ] **2.1.3.1**: Add operation summary and description
- [ ] **2.1.3.2**: Enhance response examples
- [ ] **2.1.3.3**: Add proper schema for `PingResponse`

#### Task 2.1.4: health_handler Enhancement
- [ ] **2.1.4.1**: Add operation summary and description
- [ ] **2.1.4.2**: Add comprehensive health check response schema
- [ ] **2.1.4.3**: Document health status indicators

**Enhancement Pattern Example**:
```rust
#[utoipa::path(
    get,
    path = ENDPOINT_APP_INFO,
    tag = API_TAG_SYSTEM,
    operation_id = "getAppInfo",
    summary = "Get Application Information",
    description = "Retrieves current application version and status information",
    responses(
        (status = 200, description = "Application information retrieved successfully", body = AppInfo,
         example = json!({
             "version": "0.1.0",
             "status": "ready"
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Failed to retrieve application status",
                 "type": "internal_server_error",
                 "code": "system_error"
             }
         }))
    )
)]
```

### Task Group 2.2: Authentication Handlers
**Files**: `crates/routes_app/src/routes_login.rs`, `crates/routes_app/src/routes_user.rs`
**Priority**: High
**Estimated Time**: 5-6 hours

#### Task 2.2.1: auth_initiate_handler Enhancement
- [ ] **2.2.1.1**: Add operation summary and description
- [ ] **2.2.1.2**: Enhance `RedirectResponse` schema with validation
- [ ] **2.2.1.3**: Add comprehensive response examples for both 200 and 201 status codes
- [ ] **2.2.1.4**: Document OAuth flow in operation description

#### Task 2.2.2: auth_callback_handler Enhancement
- [ ] **2.2.2.1**: Add operation summary and description
- [ ] **2.2.2.2**: Enhance `AuthCallbackRequest` schema
  - Add field descriptions and examples
  - Add validation constraints for OAuth parameters
- [ ] **2.2.2.3**: Add comprehensive error response examples
- [ ] **2.2.2.4**: Document OAuth callback flow

#### Task 2.2.3: logout_handler Enhancement
- [ ] **2.2.3.1**: Add operation summary and description
- [ ] **2.2.3.2**: Add comprehensive response examples
- [ ] **2.2.3.3**: Document session cleanup behavior

#### Task 2.2.4: request_access_handler Enhancement
- [ ] **2.2.4.1**: Add operation summary and description
- [ ] **2.2.4.2**: Enhance request/response schemas
- [ ] **2.2.4.3**: Add validation constraints and examples
- [ ] **2.2.4.4**: Document access request workflow

#### Task 2.2.5: user_info_handler Enhancement
- [ ] **2.2.5.1**: Add operation summary and description
- [ ] **2.2.5.2**: Enhance `UserInfo` schema with field descriptions
- [ ] **2.2.5.3**: Enhance `TokenType` and `RoleSource` enums with descriptions
- [ ] **2.2.5.4**: Add comprehensive response examples
- [ ] **2.2.5.5**: Add security requirements documentation

### Task Group 2.3: Model and Alias Handlers
**Files**: `crates/routes_app/src/routes_models.rs`, `crates/routes_app/src/routes_create.rs`
**Priority**: High
**Estimated Time**: 6-7 hours

#### Task 2.3.1: list_aliases_handler Enhancement
- [ ] **2.3.1.1**: Add operation summary and description
- [ ] **2.3.1.2**: Add security scopes to annotation
- [ ] **2.3.1.3**: Enhance response examples with realistic data
- [ ] **2.3.1.4**: Document discriminated union response format

#### Task 2.3.2: list_local_modelfiles_handler Enhancement
- [ ] **2.3.2.1**: Add operation summary and description
- [ ] **2.3.2.2**: Enhance `LocalModelResponse` schema with field descriptions
- [ ] **2.3.2.3**: Add comprehensive pagination examples
- [ ] **2.3.2.4**: Add security requirements

#### Task 2.3.3: get_user_alias_handler Enhancement
- [ ] **2.3.3.1**: Add operation summary and description
- [ ] **2.3.3.2**: Enhance path parameter documentation with validation
- [ ] **2.3.3.3**: Add comprehensive response examples
- [ ] **2.3.3.4**: Add security requirements

#### Task 2.3.4: create_alias_handler Enhancement
- [ ] **2.3.4.1**: Add operation summary and description
- [ ] **2.3.4.2**: Enhance `CreateAliasRequest` schema
  - Add field descriptions and examples
  - Add validation constraints (min_length, patterns)
- [ ] **2.3.4.3**: Add comprehensive error response examples
- [ ] **2.3.4.4**: Add security requirements with appropriate scopes

#### Task 2.3.5: update_alias_handler Enhancement
- [ ] **2.3.5.1**: Add operation summary and description
- [ ] **2.3.5.2**: Enhance `UpdateAliasRequest` schema
- [ ] **2.3.5.3**: Add path parameter validation
- [ ] **2.3.5.4**: Add comprehensive response examples

### Task Group 2.4: API Model Handlers
**Files**: `crates/routes_app/src/routes_api_models.rs`, `crates/routes_app/src/api_models_dto.rs`
**Priority**: High
**Estimated Time**: 7-8 hours

#### Task 2.4.1: list_api_models_handler Enhancement
- [ ] **2.4.1.1**: Add operation summary and description
- [ ] **2.4.1.2**: Add security scopes to annotation
- [ ] **2.4.1.3**: Enhance response examples with realistic API model data
- [ ] **2.4.1.4**: Document pagination behavior

#### Task 2.4.2: get_api_model_handler Enhancement
- [ ] **2.4.2.1**: Add operation summary and description
- [ ] **2.4.2.2**: Enhance path parameter with validation constraints
- [ ] **2.4.2.3**: Add comprehensive response examples
- [ ] **2.4.2.4**: Add security requirements

#### Task 2.4.3: create_api_model_handler Enhancement
- [ ] **2.4.3.1**: Add operation summary and description
- [ ] **2.4.3.2**: Enhance `CreateApiModelRequest` schema
  - Add field-level validation (min_length, max_length, patterns)
  - Add comprehensive field descriptions
  - Add realistic examples
- [ ] **2.4.3.3**: Add comprehensive error response examples
- [ ] **2.4.3.4**: Add security requirements with power user scopes

#### Task 2.4.4: update_api_model_handler Enhancement
- [ ] **2.4.4.1**: Add operation summary and description
- [ ] **2.4.4.2**: Enhance `UpdateApiModelRequest` schema
- [ ] **2.4.4.3**: Add path parameter validation
- [ ] **2.4.4.4**: Add comprehensive response examples

#### Task 2.4.5: delete_api_model_handler Enhancement
- [ ] **2.4.5.1**: Add operation summary and description
- [ ] **2.4.5.2**: Add path parameter validation
- [ ] **2.4.5.3**: Add comprehensive response examples
- [ ] **2.4.5.4**: Document deletion behavior and constraints

#### Task 2.4.6: test_api_model_handler Enhancement
- [ ] **2.4.6.1**: Add operation summary and description
- [ ] **2.4.6.2**: Enhance `TestPromptRequest` schema
  - Add field validation and descriptions
  - Add realistic examples
- [ ] **2.4.6.3**: Enhance `TestPromptResponse` schema
- [ ] **2.4.6.4**: Add comprehensive error response examples

#### Task 2.4.7: fetch_models_handler Enhancement
- [ ] **2.4.7.1**: Add operation summary and description
- [ ] **2.4.7.2**: Enhance `FetchModelsRequest` schema
- [ ] **2.4.7.3**: Enhance `FetchModelsResponse` schema
- [ ] **2.4.7.4**: Add comprehensive response examples

### Task Group 2.5: Download and Pull Handlers
**Files**: `crates/routes_app/src/routes_pull.rs`
**Priority**: Medium
**Estimated Time**: 5-6 hours

#### Task 2.5.1: list_downloads_handler Enhancement
- [ ] **2.5.1.1**: Add operation summary and description
- [ ] **2.5.1.2**: Add security requirements
- [ ] **2.5.1.3**: Enhance response examples with realistic download data
- [ ] **2.5.1.4**: Document pagination behavior

#### Task 2.5.2: create_pull_request_handler Enhancement
- [ ] **2.5.2.1**: Add operation summary and description
- [ ] **2.5.2.2**: Enhance `NewDownloadRequest` schema
  - Add field validation (patterns for repo/filename)
  - Add comprehensive descriptions and examples
- [ ] **2.5.2.3**: Add comprehensive error response examples
- [ ] **2.5.2.4**: Add security requirements

#### Task 2.5.3: pull_by_alias_handler Enhancement
- [ ] **2.5.3.1**: Add operation summary and description
- [ ] **2.5.3.2**: Enhance path parameter with validation
- [ ] **2.5.3.3**: Add comprehensive response examples
- [ ] **2.5.3.4**: Document alias resolution behavior

#### Task 2.5.4: get_download_status_handler Enhancement
- [ ] **2.5.4.1**: Add operation summary and description
- [ ] **2.5.4.2**: Enhance path parameter with validation
- [ ] **2.5.4.3**: Add comprehensive response examples for all download statuses
- [ ] **2.5.4.4**: Document download status workflow

### Task Group 2.6: API Token Handlers
**Files**: `crates/routes_app/src/routes_api_token.rs`
**Priority**: High
**Estimated Time**: 4-5 hours

#### Task 2.6.1: create_token_handler Enhancement
- [ ] **2.6.1.1**: Add operation summary and description
- [ ] **2.6.1.2**: Enhance `CreateApiTokenRequest` schema
  - Add field validation and descriptions
  - Add examples for token creation
- [ ] **2.6.1.3**: Enhance `ApiTokenResponse` schema
- [ ] **2.6.1.4**: Add comprehensive error response examples
- [ ] **2.6.1.5**: Add security requirements with appropriate scopes

#### Task 2.6.2: update_token_handler Enhancement
- [ ] **2.6.2.1**: Add operation summary and description
- [ ] **2.6.2.2**: Enhance `UpdateApiTokenRequest` schema
- [ ] **2.6.2.3**: Add path parameter validation
- [ ] **2.6.2.4**: Add comprehensive response examples

#### Task 2.6.3: list_tokens_handler Enhancement
- [ ] **2.6.3.1**: Add operation summary and description
- [ ] **2.6.3.2**: Add security requirements
- [ ] **2.6.3.3**: Enhance response examples with realistic token data
- [ ] **2.6.3.4**: Document pagination and security filtering

### Task Group 2.7: Settings Handlers
**Files**: `crates/routes_app/src/routes_settings.rs`
**Priority**: Medium
**Estimated Time**: 3-4 hours

#### Task 2.7.1: list_settings_handler Enhancement
- [ ] **2.7.1.1**: Add operation summary and description
- [ ] **2.7.1.2**: Add security requirements (admin only)
- [ ] **2.7.1.3**: Enhance response examples with realistic settings data
- [ ] **2.7.1.4**: Document settings metadata format

#### Task 2.7.2: update_setting_handler Enhancement
- [ ] **2.7.2.1**: Add operation summary and description
- [ ] **2.7.2.2**: Enhance `UpdateSettingRequest` schema
- [ ] **2.7.2.3**: Add path parameter validation
- [ ] **2.7.2.4**: Add comprehensive error response examples
- [ ] **2.7.2.5**: Document setting validation rules

#### Task 2.7.3: delete_setting_handler Enhancement
- [ ] **2.7.3.1**: Add operation summary and description
- [ ] **2.7.3.2**: Add path parameter validation
- [ ] **2.7.3.3**: Add comprehensive response examples
- [ ] **2.7.3.4**: Document setting deletion constraints

### Task Group 2.8: OpenAI Compatible Handlers
**Files**: `crates/routes_oai/src/routes_oai_models.rs`, `crates/routes_oai/src/routes_chat.rs`
**Priority**: High
**Estimated Time**: 4-5 hours

#### Task 2.8.1: oai_models_handler Enhancement
- [ ] **2.8.1.1**: Add operation summary and description
- [ ] **2.8.1.2**: Enhance custom `ListModelResponse` schema
- [ ] **2.8.1.3**: Add comprehensive response examples
- [ ] **2.8.1.4**: Add security requirements
- [ ] **2.8.1.5**: Document OpenAI API compatibility

#### Task 2.8.2: oai_model_handler Enhancement
- [ ] **2.8.2.1**: Add operation summary and description
- [ ] **2.8.2.2**: Enhance path parameter with validation
- [ ] **2.8.2.3**: Add comprehensive response examples
- [ ] **2.8.2.4**: Document model resolution behavior

#### Task 2.8.3: chat_completions_handler Enhancement
- [ ] **2.8.3.1**: Add operation summary and description
- [ ] **2.8.3.2**: Enhance request body schema (using serde_json::Value)
- [ ] **2.8.3.3**: Add comprehensive response examples for both streaming and non-streaming
- [ ] **2.8.3.4**: Add comprehensive error response examples
- [ ] **2.8.3.5**: Document OpenAI API compatibility and streaming behavior

### Task Group 2.9: Ollama Compatible Handlers
**Files**: `crates/routes_oai/src/routes_ollama.rs`
**Priority**: Medium
**Estimated Time**: 4-5 hours

#### Task 2.9.1: ollama_models_handler Enhancement
- [ ] **2.9.1.1**: Add operation summary and description
- [ ] **2.9.1.2**: Enhance `ModelsResponse` and `Model` schemas
  - Add field descriptions and examples
  - Add validation constraints where applicable
- [ ] **2.9.1.3**: Add comprehensive response examples
- [ ] **2.9.1.4**: Add security requirements

#### Task 2.9.2: ollama_model_show_handler Enhancement
- [ ] **2.9.2.1**: Add operation summary and description
- [ ] **2.9.2.2**: Enhance `ShowRequest` and `ShowResponse` schemas
- [ ] **2.9.2.3**: Add comprehensive response examples
- [ ] **2.9.2.4**: Document Ollama API compatibility

#### Task 2.9.3: ollama_model_chat_handler Enhancement
- [ ] **2.9.3.1**: Add operation summary and description
- [ ] **2.9.3.2**: Enhance `ChatRequest` schema with field descriptions
- [ ] **2.9.3.3**: Add comprehensive response examples for streaming and non-streaming
- [ ] **2.9.3.4**: Add error response examples
- [ ] **2.9.3.5**: Document Ollama chat API compatibility

## Phase 3: Quality Assurance and Testing

### Task 3.1: Schema Validation Testing
**Priority**: High
**Estimated Time**: 2-3 hours

#### Subtasks:
- [ ] **3.1.1**: Generate OpenAPI specification and validate syntax
- [ ] **3.1.2**: Test TypeScript generation with enhanced schemas
- [ ] **3.1.3**: Validate all parameter constraints work correctly
- [ ] **3.1.4**: Test all response examples are valid JSON
- [ ] **3.1.5**: Verify security scheme documentation is complete

### Task 3.2: Documentation Completeness Review
**Priority**: Medium
**Estimated Time**: 2 hours

#### Subtasks:
- [ ] **3.2.1**: Review all handlers for consistent documentation patterns
- [ ] **3.2.2**: Verify all schemas have proper examples
- [ ] **3.2.3**: Check all security requirements are properly documented
- [ ] **3.2.4**: Validate all error responses follow standard patterns
- [ ] **3.2.5**: Test generated documentation in Swagger UI

### Task 3.3: Integration Testing
**Priority**: High
**Estimated Time**: 2 hours

#### Subtasks:
- [ ] **3.3.1**: Test OpenAPI specification generation
- [ ] **3.3.2**: Validate TypeScript client generation
- [ ] **3.3.3**: Test API functionality with enhanced documentation
- [ ] **3.3.4**: Verify no breaking changes to existing functionality
- [ ] **3.3.5**: Performance test OpenAPI generation with enhanced schemas

## Implementation Guidelines

### Enhancement Checklist for Each Handler

For each handler, ensure the following enhancements are applied:

#### Path Annotation Enhancements:
- [ ] **Summary**: Add concise operation summary
- [ ] **Description**: Add detailed operation description
- [ ] **Parameters**: Enhance all parameters with validation and examples
- [ ] **Request Body**: Add detailed request body documentation with examples
- [ ] **Responses**: Add comprehensive response documentation for all status codes
- [ ] **Security**: Add appropriate security requirements with scopes
- [ ] **Examples**: Include realistic examples for all request/response schemas

#### Schema Enhancements:
- [ ] **Field Descriptions**: Add descriptions for all fields
- [ ] **Validation Constraints**: Add appropriate validation (min/max length, patterns, etc.)
- [ ] **Examples**: Add field-level and schema-level examples
- [ ] **Enum Documentation**: Document all enum variants with descriptions
- [ ] **Default Values**: Document default values where applicable

#### Response Documentation:
- [ ] **Success Responses**: Comprehensive examples for all success status codes
- [ ] **Error Responses**: Standardized error response examples
- [ ] **Status Code Coverage**: Document all possible status codes
- [ ] **Content Types**: Specify appropriate content types
- [ ] **Headers**: Document custom response headers where applicable

### Common Patterns to Follow

#### Security Requirements Pattern:
```rust
security(
    ("bearer_auth" = ["scope_token_user"]),
    ("session_auth" = ["resource_user"])
)
```

#### Parameter Validation Pattern:
```rust
params(
    ("id" = String, Path, description = "Resource identifier", 
     example = "resource_123", min_length = 1, max_length = 50)
)
```

#### Error Response Pattern:
```rust
(status = 400, description = "Invalid request", body = OpenAIApiError,
 example = json!({
     "error": {
         "message": "Validation failed: field is required",
         "type": "invalid_request_error", 
         "code": "validation_error"
     }
 }))
```

## Success Criteria

### Completion Criteria:
- [ ] All 32 handlers have enhanced documentation
- [ ] All schemas have field-level descriptions and validation
- [ ] All endpoints have comprehensive response examples
- [ ] All security requirements are properly documented
- [ ] TypeScript generation produces accurate and comprehensive types
- [ ] OpenAPI specification validates successfully
- [ ] No breaking changes to existing API functionality

### Quality Gates:
- [ ] **Documentation Review**: All enhancements reviewed for consistency
- [ ] **Schema Validation**: All schemas validate correctly
- [ ] **TypeScript Generation**: Generated types are accurate and complete
- [ ] **API Testing**: All enhanced endpoints function correctly
- [ ] **Performance Testing**: OpenAPI generation performance is acceptable

## Risk Mitigation

### Technical Risks and Mitigations:
1. **Breaking Changes**: Test all changes thoroughly before deployment
2. **Performance Impact**: Monitor OpenAPI generation performance
3. **TypeScript Generation Issues**: Validate generated types after each enhancement
4. **Schema Validation Errors**: Test all schema changes incrementally

### Implementation Best Practices:
1. **Incremental Changes**: Implement enhancements in small, testable increments
2. **Consistent Patterns**: Follow established patterns for consistency
3. **Comprehensive Testing**: Test each enhancement thoroughly
4. **Documentation**: Document any deviations from standard patterns
5. **Rollback Strategy**: Maintain ability to rollback changes if issues arise

## Estimated Total Time: 40-50 hours

This comprehensive task list ensures systematic enhancement of all API documentation while maintaining quality and consistency across the entire codebase.
