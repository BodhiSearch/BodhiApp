# API Documentation Enhancement Plan

## Executive Summary

This specification outlines a comprehensive enhancement plan for our utoipa-based OpenAPI documentation system. The goal is to create rich, accurate, and complete API documentation that generates high-quality TypeScript types, reducing bugs in network calls and improving developer experience.

## Current State Analysis

### What We're Doing Right

Our current utoipa implementation demonstrates solid foundational practices:

✅ **Schema Documentation**: Consistent use of `ToSchema` derive on request/response structures
✅ **Parameter Handling**: Proper use of `IntoParams` for query parameters (`PaginationSortParams`)
✅ **Path Parameters**: Well-documented path parameters with descriptions
✅ **Security Integration**: Configured security schemes (bearer_auth, session_auth)
✅ **Response Documentation**: Comprehensive response documentation with status codes
✅ **API Organization**: Proper tagging system for logical API grouping
✅ **Examples**: Good use of examples in schema definitions

### Current Handler Coverage

Based on analysis, we have **32 API handlers** across the system:

#### Routes App (25 handlers)
- **API Models**: 7 handlers (list, get, create, update, delete, test, fetch)
- **Authentication**: 4 handlers (initiate, callback, logout, request_access)
- **Models/Aliases**: 5 handlers (list_aliases, list_modelfiles, get_alias, create_alias, update_alias)
- **Downloads/Pull**: 4 handlers (list_downloads, create_pull, pull_by_alias, get_download_status)
- **API Tokens**: 3 handlers (create, update, list)
- **Settings**: 3 handlers (list, update, delete)
- **System**: 4 handlers (app_info, setup, ping, health)
- **User**: 1 handler (user_info)

#### Routes OAI (7 handlers)
- **OpenAI Compatibility**: 3 handlers (models, model, chat_completions)
- **Ollama Compatibility**: 3 handlers (models, show, chat)

### Key Gaps Identified

1. **Field-Level Validation**: Missing validation constraints (min/max, patterns, formats)
2. **Enhanced Examples**: Limited inline examples in path annotations
3. **Security Scopes**: Missing required scopes in security annotations
4. **Parameter Validation**: Insufficient validation constraints on parameters
5. **Response Examples**: Limited comprehensive response examples for error cases
6. **Schema Descriptions**: Incomplete field-level descriptions
7. **Header Documentation**: Missing custom header parameter documentation

## Enhancement Strategy

### Phase 1: Global Infrastructure Improvements

#### 1.1 Security Scheme Enhancement
- **Current**: Basic bearer_auth and session_auth schemes
- **Target**: Enhanced security schemes with detailed descriptions and scope requirements
- **Impact**: Better API security documentation and clearer authentication requirements

#### 1.2 Response Schema Standardization
- **Current**: Inconsistent response examples and error handling
- **Target**: Standardized error response schemas with comprehensive examples
- **Impact**: Better error handling in TypeScript clients

#### 1.3 Parameter Validation Framework
- **Current**: Basic parameter documentation
- **Target**: Comprehensive validation constraints (min/max, patterns, formats)
- **Impact**: Better client-side validation and error prevention

### Phase 2: Handler-by-Handler Enhancement

#### 2.1 Schema Enhancement Pattern
For each handler, we will implement:
- **Field-level validation**: Add constraints like `min_length`, `max_length`, `pattern`, `minimum`, `maximum`
- **Enhanced descriptions**: Add detailed field descriptions with examples
- **Comprehensive examples**: Include realistic examples for all request/response schemas
- **Enum documentation**: Proper documentation for all enum types

#### 2.2 Path Annotation Enhancement Pattern
For each endpoint, we will enhance:
- **Operation metadata**: Add summary and description
- **Parameter validation**: Add validation constraints to path/query parameters
- **Response examples**: Add comprehensive examples for all status codes
- **Security scopes**: Add required scopes for authentication
- **Header documentation**: Document custom headers where applicable

### Phase 3: Quality Assurance

#### 3.1 Validation Testing
- Ensure all enhanced schemas generate valid OpenAPI specifications
- Verify TypeScript generation produces correct types
- Test parameter validation constraints

#### 3.2 Documentation Completeness
- Verify all endpoints have complete documentation
- Ensure all schemas have proper examples
- Validate security requirements are properly documented

## Technical Implementation Details

### Enhanced Schema Pattern

```rust
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "name": "Production API Key",
    "scopes": ["scope_token_user"]
}))]
pub struct CreateApiTokenRequest {
  /// Human-readable name for the API token
  #[schema(
    min_length = 1, 
    max_length = 100, 
    example = "Production API Key",
    description = "Display name for the token"
  )]
  pub name: Option<String>,
  
  /// Required scopes for this token
  #[schema(
    example = json!(["scope_token_user", "scope_token_power_user"]),
    description = "List of scopes that define the token's permissions"
  )]
  pub scopes: Option<Vec<String>>,
}
```

### Enhanced Path Annotation Pattern

```rust
#[utoipa::path(
    post,
    path = "/bodhi/v1/tokens",
    tag = "api-keys",
    operation_id = "createApiToken",
    summary = "Create API Token",
    description = "Creates a new API token for programmatic access to the API",
    request_body(
        content = CreateApiTokenRequest,
        description = "Token creation parameters",
        example = json!({
            "name": "Production API Key",
            "scopes": ["scope_token_user"]
        })
    ),
    responses(
        (status = 201, description = "Token created successfully", body = ApiTokenResponse,
         example = json!({
             "offline_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9...",
             "token_id": "token_123",
             "name": "Production API Key",
             "created_at": "2024-01-15T10:30:00Z"
         })),
        (status = 400, description = "Invalid request data", body = OpenAIApiError,
         example = json!({
             "error": {
                 "message": "Validation failed: name is required",
                 "type": "invalid_request_error",
                 "code": "validation_error"
             }
         })),
        (status = 500, description = "Internal server error", body = OpenAIApiError)
    ),
    security(
        ("bearer_auth" = ["scope_token_power_user"]),
        ("session_auth" = ["resource_power_user"])
    )
)]
```

### Enhanced Parameter Documentation

```rust
#[derive(Debug, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct PaginationSortParams {
  /// Page number (1-based)
  #[serde(default = "default_page")]
  #[schema(
    minimum = 1, 
    example = 1,
    description = "Page number for pagination, starting from 1"
  )]
  pub page: usize,

  /// Number of items per page
  #[serde(default = "default_page_size")]
  #[schema(
    minimum = 1, 
    maximum = 100, 
    example = 30,
    description = "Number of items to return per page (max 100)"
  )]
  pub page_size: usize,

  /// Field to sort by
  #[serde(default)]
  #[schema(
    example = "updated_at",
    description = "Field name to sort by"
  )]
  pub sort: Option<String>,

  /// Sort order
  #[serde(default = "default_sort_order")]
  #[schema(
    pattern = "^(asc|desc)$", 
    example = "desc",
    description = "Sort order: 'asc' for ascending, 'desc' for descending"
  )]
  pub sort_order: String,
}
```

## Expected Outcomes

### 1. Enhanced TypeScript Generation
- **More Precise Types**: Field-level validation constraints will generate more precise TypeScript interfaces
- **Better Enum Support**: Properly documented enums will generate accurate TypeScript union types
- **Comprehensive Error Handling**: Standardized error responses will improve client error handling

### 2. Improved Developer Experience
- **Self-Documenting API**: Rich descriptions and examples make the API self-explanatory
- **Better IDE Support**: Enhanced schemas provide better autocomplete and validation in IDEs
- **Reduced Integration Time**: Comprehensive documentation reduces time needed to understand and integrate with the API

### 3. Reduced Bug Potential
- **Client-Side Validation**: Enhanced parameter constraints enable better client-side validation
- **Type Safety**: More precise TypeScript types catch more errors at compile time
- **Clear Error Responses**: Standardized error formats improve error handling consistency

## Success Metrics

### Quantitative Metrics
- **100% Handler Coverage**: All 32 handlers have enhanced documentation
- **Field Coverage**: All request/response fields have descriptions and constraints
- **Example Coverage**: All schemas have realistic examples
- **Validation Coverage**: All parameters have appropriate validation constraints

### Qualitative Metrics
- **TypeScript Quality**: Generated TypeScript types are accurate and comprehensive
- **Documentation Clarity**: API documentation is self-explanatory and complete
- **Developer Feedback**: Positive feedback from frontend developers using the generated types

## Risk Mitigation

### Technical Risks
- **Breaking Changes**: Ensure enhancements don't break existing API functionality
- **Performance Impact**: Monitor OpenAPI generation performance with enhanced schemas
- **TypeScript Generation**: Verify enhanced schemas don't break TypeScript generation

### Mitigation Strategies
- **Incremental Rollout**: Implement enhancements incrementally by handler groups
- **Testing Strategy**: Comprehensive testing of each enhancement before deployment
- **Rollback Plan**: Maintain ability to rollback to previous schema versions if issues arise

## Timeline Estimate

### Phase 1: Global Infrastructure (1-2 days)
- Security scheme enhancement
- Response schema standardization
- Parameter validation framework

### Phase 2: Handler Enhancement (5-7 days)
- System handlers (1 day)
- Authentication handlers (1 day)
- Model/Alias handlers (1.5 days)
- API Model handlers (1.5 days)
- Download/Pull handlers (1 day)
- Settings/Token handlers (1 day)
- OAI/Ollama handlers (1 day)

### Phase 3: Quality Assurance (1 day)
- Validation testing
- Documentation review
- TypeScript generation verification

**Total Estimated Time: 7-10 days**

## Conclusion

This enhancement plan will transform our API documentation from good to exceptional, providing developers with comprehensive, accurate, and type-safe interfaces. The systematic approach ensures consistency across all endpoints while the phased implementation minimizes risk and allows for iterative improvements.

The enhanced documentation will serve as both a developer tool and a quality assurance mechanism, catching potential issues early and providing clear guidance for API consumers.
