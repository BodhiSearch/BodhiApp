# Playwright UI Test Implementation Plan for User Access Request Feature

**Date**: 2025-01-11  
**Status**: Phase 1 - Implementation in Progress  
**Previous Context**: Building on completed backend implementation from progress.md

## Overview

We need to restore and complete the Playwright-based browser tests that validate the end-to-end user access request workflow, including multi-user scenarios with role-based approval and session clearing functionality.

## Background Context

From the stash analysis and progress document, we have:
- ✅ **Backend Implementation Complete**: User access request system with session management
- ✅ **Integration Tests Passing**: HTTP-level tests for approval workflow with session clearing  
- ❌ **UI Tests Missing**: Playwright browser tests were started but need restoration and completion

### Existing UI Components (Already Have Data-TestIds)

From code analysis, these components already have proper test attributes:

**AuthCard.tsx** (Lines 24, 34-35, 38, 44, 50, 59, 71):
```tsx
data-testid="auth-card-loading"          // Loading state
data-testid="auth-card-container"        // Container wrapper  
data-testid="auth-card"                  // Main card
data-testid="auth-card-header"           // Header section
data-testid="auth-card-content"          // Content section
data-testid="auth-card-description"      // Description text
data-testid="auth-card-actions"          // Actions container
data-testid="auth-card-action-{index}"   // Individual action buttons
```

**RequestAccessPage.tsx** (Line 74):
```tsx
data-testid="request-access-page"        // Page container
```

### Stashed Test Files (From stash@{0})

**RequestAccessPage.mjs** - Page object with these selectors:
```javascript
selectors = {
  requestAccessButton: '[data-testid="auth-card-action-0"]',
  authCard: '[data-testid="auth-card"]',
  authCardHeader: '[data-testid="auth-card-header"]',
  authCardContent: '[data-testid="auth-card-content"]', 
  authCardDescription: '[data-testid="auth-card-description"]',
  authCardActions: '[data-testid="auth-card-actions"]',
  pageContainer: '[data-testid="request-access-page"]',
};
```

**multi-user-request-approval-flow.spec.mjs** - Main test spec with:
- Multi-user test credentials setup
- Auth server client creation
- Server manager integration
- Phase 1 test for manager user request flow

## Implementation Plan

### Phase 1: Restore Core Test Files from Stash ⬅️ **CURRENT PHASE**

1. **Restore RequestAccessPage.mjs**
   - Page object for request access UI interactions
   - Contains selectors for auth-card components with data-testid attributes
   - Methods for checking request status, clicking request button, and validating pending state

2. **Restore multi-user-request-approval-flow.spec.mjs**
   - Main test spec for the complete access request workflow
   - Tests multi-user scenarios with manager requesting access and admin approval

### Phase 2: Complete UI Test Implementation

1. **Complete the Multi-User Test Spec**
   - **Phase 1 Test**: Manager user requests access
     - Login as manager user
     - Navigate to request-access page
     - Click request access button
     - Verify pending state is shown
     - Test protected page redirects back to request-access
     - Verify persistence across page reloads
   
   - **Phase 2 Test**: Admin user approves request
     - Login as admin user
     - Navigate to users management page
     - Find pending request from manager
     - Approve request with resource_user role
     - Verify request status changes to approved
   
   - **Phase 3 Test**: Manager user access after approval
     - Login as manager user again
     - Verify automatic redirect to chat page
     - Verify role is properly assigned
     - Test access to protected resources

2. **Add Users Management Page Object**
   - Create UsersManagementPage.mjs for admin interface
   - Include selectors for user request list
   - Methods for approving/rejecting requests
   - Pagination support for user lists

### Phase 3: Session Clearing Validation

1. **Test Session Invalidation**
   - Create test to verify sessions are cleared after role approval
   - Use multiple browser contexts to simulate concurrent sessions
   - Verify all sessions require re-authentication after role change
   - Test that new login has updated permissions

2. **Add Session Monitoring Helpers**
   - Create utilities to track session state
   - Methods to verify session invalidation
   - Helper to count active sessions for a user

### Phase 4: Edge Cases and Error Scenarios

1. **Duplicate Request Prevention**
   - Test that users cannot submit multiple pending requests
   - Verify 409 Conflict error handling

2. **Already Has Role Scenario**
   - Test users with existing roles cannot request access
   - Verify proper error messaging

3. **Authorization Hierarchy**
   - Test that managers can only assign equal/lower roles
   - Verify admins can assign all roles
   - Test rejection workflow

### Phase 5: UI Component Data-TestId Updates

1. **Update React Components** (Already present, verify completeness)
   - AuthCard.tsx - Has data-testid attributes ✅
   - RequestAccessPage.tsx - Has page container testid ✅ 
   - UsersManagementPage.tsx - Need to add testids for admin interface

2. **Build Embedded UI**
   - Run `make clean.ui` to clean old builds
   - Run `make build.ui` to rebuild with testid attributes
   - Ensure embedded UI is used in tests

### Phase 6: Test Infrastructure Setup

1. **Environment Configuration**
   - Verify all required environment variables are set:
     - INTEG_TEST_USER_MANAGER (manager user email)
     - INTEG_TEST_USER_ADMIN (admin user email) 
     - INTEG_TEST_PASSWORD (test user password)
     - Auth server configuration variables

2. **Test Data Setup**
   - Create helper to reset user roles before tests
   - Ensure clean state for each test run
   - Helper to clear all pending requests

### Phase 7: Run and Validate Tests

1. **Execute Test Suite**
   ```bash
   cd crates/lib_bodhiserver_napi
   npm test -- tests-js/specs/access-request/
   ```

2. **Validate Coverage**
   - Request submission flow
   - Approval workflow
   - Session clearing on role change
   - Error handling scenarios
   - Multi-user interactions

## Implementation Details

### Key Components to Create/Restore

#### 1. RequestAccessPage.mjs - Page object with methods:
- `expectRequestAccessPage()` - Verify on request access page
- `clickRequestAccess()` - Submit access request  
- `expectPendingState()` - Verify pending request UI
- `getSubmittedDate()` - Extract submission date
- `testProtectedPageRedirect()` - Test redirect to request page

#### 2. UsersManagementPage.mjs - New page object:
- `navigateToUsersPage()` - Go to users management
- `findPendingRequest(username)` - Locate specific request
- `approveRequest(id, role)` - Approve with role
- `rejectRequest(id)` - Reject request
- `expectRequestInList(username, status)` - Verify request status

#### 3. multi-user-request-approval-flow.spec.mjs - Complete test:
- Setup with multiple test users
- Comprehensive workflow testing
- Session invalidation verification
- Cleanup after tests

### Backend Integration Points
- `/bodhi/v1/user/request-access` - Submit request
- `/bodhi/v1/user/request-status` - Check status
- `/bodhi/v1/access-requests` - List all requests (admin)
- `/bodhi/v1/access-requests/{id}/approve` - Approve request
- `/bodhi/v1/access-requests/{id}/reject` - Reject request

### Session Testing Strategy
- Use multiple Playwright browser contexts
- Track cookies/sessions before and after approval
- Verify forced re-authentication
- Test new permissions are active

### Test Environment Requirements

**Environment Variables Needed:**
```bash
INTEG_TEST_USER_MANAGER=manager@example.com
INTEG_TEST_USER_ADMIN=admin@example.com  
INTEG_TEST_PASSWORD=test123
INTEG_TEST_MAIN_AUTH_URL=https://auth.example.com
INTEG_TEST_AUTH_REALM=test-realm
INTEG_TEST_DEV_CONSOLE_CLIENT_SECRET=dev-console-secret
```

**Test User Roles:**
- Manager user: No initial role, will request access
- Admin user: resource_manager role, can approve requests

### Directory Structure
```
crates/lib_bodhiserver_napi/tests-js/
├── pages/
│   ├── BasePage.mjs                     ✅ (exists)
│   ├── LoginPage.mjs                    ✅ (exists)
│   ├── RequestAccessPage.mjs            ❌ (needs restore)
│   └── UsersManagementPage.mjs          ❌ (needs creation)
├── specs/
│   └── access-request/
│       └── multi-user-request-approval-flow.spec.mjs  ❌ (needs restore)
└── playwright/
    ├── auth-server-client.mjs           ✅ (exists)
    └── bodhi-app-server.mjs             ✅ (exists)
```

## Current Status & Next Actions

**Phase 1 Tasks:**
1. ✅ Create this context file
2. ✅ Restore RequestAccessPage.mjs from stash
3. ✅ Restore multi-user-request-approval-flow.spec.mjs from stash  
4. ✅ Run Phase 1 test and verify it passes
5. ✅ Debug and fix issues (1 attempt - fixed user ID issue)

**Success Criteria for Phase 1:**
- ✅ RequestAccessPage.mjs properly restored with all methods
- ✅ Multi-user test spec restored with Phase 1 test
- ✅ Test successfully runs and passes manager request flow
- ✅ Proper authentication and page navigation working
- ✅ Request submission and pending state validation working

## Phase 1 Implementation Results

**Status**: ✅ **COMPLETED SUCCESSFULLY**

**Issues Fixed:**
1. **User ID vs Username Issue**: The `makeResourceAdmin` API changed to require user ID instead of username/email. Fixed by using `INTEG_TEST_USER_ADMIN_ID` from `.env.test`.

**Test Validation Results:**
- ✅ Manager user can login via OAuth flow
- ✅ Manager user redirects to request-access page 
- ✅ Request access button is visible and clickable
- ✅ After clicking request access, pending state is shown
- ✅ Protected page redirects back to request-access page  
- ✅ Request persistence across page reloads works
- ✅ Date formatting and extraction working correctly

**Files Successfully Created:**
- `/tests-js/pages/RequestAccessPage.mjs` - Complete page object with all methods
- `/tests-js/specs/access-request/multi-user-request-approval-flow.spec.mjs` - Phase 1 test spec

**Test Execution Results:**
```
✓  1 [chromium] › Multi-User Request and Approval Flow › Phase 1: Manager User Request Flow (3.1s)
1 passed (5.6s)
```

**Next Phase Ready For:**
Phase 2 - Complete UI Test Implementation with admin approval workflow

This comprehensive plan ensures full validation of the user access request feature including the critical session clearing functionality that ensures security when roles change.