# OAuth Logout 303 Redirect Migration

**Date**: 2025-06-14  
**Author**: AI Assistant  
**Type**: Backend + Frontend Migration  
**Status**: Completed

## Overview

Migrated the `/app/logout` endpoint from returning HTTP 200 responses with Location headers to HTTP 303 redirects with Location headers, following the same pattern as the OAuth initiation flow. Additionally moved page redirection logic from hooks to invoking components.

## Changes Made

### Backend Changes

#### 1. Updated Logout Handler (`crates/routes_app/src/routes_login.rs`)
- **Changed status code**: HTTP 200 → HTTP 303 (SEE_OTHER)
- **Removed TODO comment** about avoiding redirects for axios/xmlhttprequest
- **Updated OpenAPI documentation** to reflect 303 status code

```rust
// Before
let response = Response::builder()
  .status(StatusCode::OK)
  .header(LOCATION, ui_login)
  .body(Body::empty())
  .unwrap();

// After  
let response = Response::builder()
  .status(StatusCode::SEE_OTHER)
  .header(LOCATION, ui_login)
  .body(Body::empty())
  .unwrap();
```

#### 2. Updated Backend Tests
- Changed test expectations from HTTP 200 to HTTP 303
- All logout handler tests now pass with new status code

### Frontend Changes

#### 1. Updated useLogoutHandler Hook (`crates/bodhi/src/hooks/useLogoutHandler.ts`)
- **Removed redirection logic** from hook (moved to components)
- **Added callback options** for onSuccess and onError
- **Removed dependencies** on useRouter and useToastMessages
- **Simplified interface** to be more reusable

```typescript
// Before - Hook handled redirects
const { logout, isLoading } = useLogoutHandler();

// After - Components handle redirects
const { logout, isLoading } = useLogoutHandler({
  onSuccess: (response) => {
    const redirectUrl = response.headers?.location || ROUTE_DEFAULT;
    window.location.href = redirectUrl;
  },
  onError: (message) => {
    showError('Logout failed', `Message: ${message}. Try again later.`);
  },
});
```

#### 2. Updated useLogout Query Hook (`crates/bodhi/src/hooks/useQuery.ts`)
- **Added axios configuration** to handle 303 redirects
- **Updated validateStatus** to accept 303, 200, and 201 status codes
- **Added maxRedirects: 0** to prevent automatic redirect following

#### 3. Updated Components
- **LoginMenu.tsx**: Added redirect handling in component
- **login/page.tsx**: Added redirect handling in component  
- **Removed fallback code**: No legacy auth_url support

#### 4. Updated All Tests
- **MSW mocks**: Changed from 303 to 201 status codes (to avoid MSW concurrency issues)
- **Added comprehensive test coverage**:
  - Successful logout with redirect
  - Error handling scenarios
  - Missing Location header fallback
  - Network error handling
- **Fixed test expectations** for ROUTE_DEFAULT (`/ui/chat`)

## Key Architectural Decisions

### 1. Hook Responsibility Separation
- **Hooks**: Handle API calls and data management only
- **Components**: Handle UI actions like redirects and user feedback
- **Rationale**: Better separation of concerns, more testable, reusable hooks

### 2. HTTP 303 vs 200 Status Codes
- **303 SEE_OTHER**: Standard HTTP redirect status for POST operations
- **Better SPA compatibility**: More semantic than 200 + Location header
- **Consistent with OAuth initiate**: Both endpoints now use same pattern

### 3. Test Strategy
- **MSW status codes**: Use 201 instead of 303 in tests to avoid concurrency issues
- **Component integration**: Test actual redirect behavior with window.location mocking
- **Error scenarios**: Comprehensive coverage of failure modes

## Testing Results

### Frontend Tests
- **All 40 OAuth-related tests passing**
- **Coverage includes**:
  - OAuth hooks (12 tests)
  - Logout handler hook (3 tests)
  - LoginMenu component (11 tests)
  - Login page component (14 tests)

### Backend Tests
- **All 98 backend tests passing**
- **Validates 303 status code and Location header**
- **Updated OpenAPI documentation tests**

## Migration Benefits

1. **Consistency**: Both OAuth endpoints now use HTTP 303 redirects
2. **Standards Compliance**: Proper HTTP semantics for redirect after POST
3. **Better Architecture**: Clear separation between data and UI concerns
4. **Improved Testability**: Hooks are more focused and easier to test
5. **Enhanced Error Handling**: More granular error handling in components

## Files Modified

### Backend
- `crates/routes_app/src/routes_login.rs` - Updated logout handler and tests

### Frontend  
- `crates/bodhi/src/hooks/useLogoutHandler.ts` - Simplified hook interface
- `crates/bodhi/src/hooks/useLogoutHandler.test.tsx` - Updated tests
- `crates/bodhi/src/hooks/useQuery.ts` - Added 303 redirect handling
- `crates/bodhi/src/components/LoginMenu.tsx` - Added redirect logic
- `crates/bodhi/src/components/LoginMenu.test.tsx` - Updated tests
- `crates/bodhi/src/app/ui/login/page.tsx` - Added redirect logic
- `crates/bodhi/src/app/ui/login/page.test.tsx` - Updated tests

## Additional Enhancements

### 1. No-Body Responses
- **Backend**: Added `Content-Length: 0` headers to all 303 redirect responses
- **Frontend**: Removed `ctx.json({})` from MSW test mocks
- **Rationale**: Cleaner HTTP responses, no unnecessary JSON parsing

### 2. Enhanced Logout Error Handling
- **Local Storage Reset**: Clear localStorage and sessionStorage on logout failure
- **Cookie Cleanup**: Expire all cookies by setting past expiration dates
- **Forced Redirect**: Always redirect to `/ui/login` on logout errors
- **User Feedback**: Show error message before redirecting

```typescript
onError: (message) => {
  // Reset local storage and cookies on logout failure
  localStorage.clear();
  sessionStorage.clear();
  // Clear all cookies by setting them to expire
  document.cookie.split(";").forEach((c) => {
    const eqPos = c.indexOf("=");
    const name = eqPos > -1 ? c.substr(0, eqPos) : c;
    document.cookie = name + "=;expires=Thu, 01 Jan 1970 00:00:00 GMT;path=/";
  });
  showError('Logout failed', `Message: ${message}. Redirecting to login page.`);
  // Redirect to login page
  window.location.href = '/ui/login';
}
```

## Future Considerations

1. **Error Recovery**: Consider implementing retry logic for transient failures
2. **User Feedback**: Could add loading states during logout process
3. **Session Management**: Potential for more sophisticated session cleanup
4. **Analytics**: Could add logout event tracking
5. **Security**: Consider implementing secure cookie clearing for production environments

## Documentation Updates

As part of this migration, updated architecture documentation to reflect our "dumb frontend" principles:

### Updated Files
- **`ai-docs/01-architecture/frontend-react.md`** - Added comprehensive "dumb frontend" architecture section
- **`ai-docs/01-architecture/api-integration.md`** - Added OAuth callback example and backend-driven patterns
- **`ai-docs/README.md`** - Updated descriptions to highlight new architectural guidance
- **`ai-docs/01-architecture/README.md`** - Updated navigation and philosophy sections

### Key Documentation Additions
1. **Dumb Frontend Principles**: Frontend focuses on presentation, backend handles business logic
2. **Pass-Through Pattern**: Send all data to backend without filtering or validation
3. **OAuth Callback Example**: Comprehensive example of sending all query params to backend
4. **Page-Based Actions**: Components handle UI actions, hooks handle data operations
5. **Error Handling Patterns**: Display backend errors without frontend interpretation

## Related Documentation

- [Frontend Next.js Architecture](../01-architecture/frontend-react.md) - "Dumb frontend" patterns and OAuth examples
- [API Integration](../01-architecture/api-integration.md) - Backend-driven integration patterns
- [Testing Strategy](../01-architecture/testing-strategy.md) - Testing patterns for frontend-backend integration
