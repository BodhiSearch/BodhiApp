# Remove tool_call_id from Toolset Execute API

## Summary
Remove redundant `tool_call_id` field from toolset execute API request and response. The backend was simply echoing back a value the frontend already had.

## Status: COMPLETED

## Scope
- Remove from API request and response
- Update backend domain, DTO, and service layers
- Update frontend to use local `toolCall.id` instead of response field
- No backward compatibility concerns - internal API only

## Phases

### Phase rust-domain: Domain Models
- [x] Remove field from `ToolsetExecutionRequest`
- [x] Remove field from `ToolsetExecutionResponse`

### Phase rust-dto: DTO Layer
- [x] Remove field from `ExecuteToolsetRequest`
- [x] Update conversion implementations

### Phase rust-service: Service Layer
- [x] Remove field from all response constructions

### Phase rust-routes: Route Handlers
- [x] Update test mocks and request bodies

### Phase rust-integration: Integration Tests
- [x] Remove from request payloads and response assertions

### Phase regenerate: OpenAPI and TypeScript
- [x] Regenerated via `make build.ts-client`

### Phase frontend: Frontend Updates
- [x] Remove from request body
- [x] Use `toolCall.id` directly instead of `result.tool_call_id`

## Verification
- [x] cargo fmt
- [x] cargo build
- [x] TypeScript compilation
- [x] Frontend build
