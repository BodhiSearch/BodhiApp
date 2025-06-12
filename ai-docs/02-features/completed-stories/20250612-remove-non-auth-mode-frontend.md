# Remove Non-Authenticated Mode from Frontend

## Overview
Following the backend changes to remove non-authenticated mode installation option, this story focuses on cleaning up all frontend code related to the now-deprecated non-authenticated mode. The goal is to simplify the frontend codebase by removing conditional logic that checks for `authz` status, updating authentication flows, and ensuring all components assume authenticated-only operation.

## Current vs. Desired State

### Current State (To Be Removed)
```
Frontend Behavior:
- Components check AppInfo.authz to determine authentication requirements
- Login components display special messaging for non-authenticated mode
- AppInitializer has conditional logic for non-authenticated mode
- Setup flow includes non-authenticated mode option
- API requests include authz parameter during setup
- Types include authz fields in various interfaces
```

### Desired State (Authenticated Only)
```
Frontend Behavior:
- No authz checks in components (always assume authenticated)
- Simplified login flow without non-authenticated mode messaging
- Streamlined AppInitializer with authentication-only logic
- Setup flow proceeds directly to authenticated setup, change the mode setup screen from 2-col to single column, with only Setup button option, removing other info related to this mode setup
- Few steps of setup screens require public access of pages, after Resource Admin, screens are only served to authenticated, otherwise redirected appropriately
- No authz parameter in API requests
- Types updated to remove authz fields
```

## Core Features to Remove

### 1. AppInfo Type and API Integration

#### Remove authz Field
- Update AppInfo interface to remove authz field
- Remove authz-related conditional logic in components using AppInfo
- Update tests that mock AppInfo responses

#### Files Affected
- Types definitions
- API hooks and query functions
- Components that consume AppInfo

### 2. Authentication Flow Components

#### Simplify Authentication Logic
- Remove conditional rendering based on authz status
- Update login components to assume authenticated mode
- Simplify AppInitializer component logic
- Remove non-authenticated mode messaging

#### Files Affected
- Authentication components
- Login page components
- AppInitializer and related utilities
- Navigation guards and route protection

### 3. Setup Flow Components

#### Remove Non-Authenticated Option
- Update setup flow to only offer authenticated mode
- Remove authz parameter from setup API calls
- Simplify setup component logic
- Update setup-related tests

#### Files Affected
- Setup page components
- Setup mode selection components
- Setup API integration hooks

### 4. Error Handling and Messaging

#### Update Error Messages
- Remove error messages specific to non-authenticated mode
- Update error handling for authentication failures
- Ensure consistent messaging for authentication requirements

#### Files Affected
- Error components
- Toast notifications
- Error handling utilities

## Implementation Phases

### Phase 1: Update Core Types and API Integration
1. Remove authz field from AppInfo interface
2. Update API hooks that use AppInfo
3. Remove authz parameter from setup API calls
4. Update tests for API integration

### Phase 2: Simplify Authentication Components
1. Update AppInitializer to remove non-authenticated mode logic
2. Simplify login components to assume authenticated mode
3. Update navigation guards and route protection
4. Remove conditional rendering based on authz status

### Phase 3: Update Setup Flow
1. Remove non-authenticated mode option from setup UI
2. Update setup flow to proceed directly to authenticated setup
3. Simplify setup component logic
4. Update setup-related tests

### Phase 4: Clean Up Error Handling and Edge Cases
1. Update error messages related to authentication
2. Ensure consistent error handling for authentication failures
3. Remove any remaining references to non-authenticated mode
4. Comprehensive testing of authentication flows

## Testing Requirements

### Unit Testing
- Update unit tests for components that previously had authz-related logic
- Remove test cases specific to non-authenticated mode
- Add tests to verify authenticated-only behavior
- Update mocks and fixtures to remove authz field

### Integration Testing
- Test complete authentication flow
- Verify setup process works correctly
- Ensure proper redirects for unauthenticated users
- Test error handling for authentication failures

### End-to-End Testing
- Complete setup flow validation
- Authentication enforcement verification
- Login and session management
- Error handling and user feedback

## Security Considerations

### 1. Enhanced Security Posture
- **Consistent Authentication**: All users properly authenticated
- **No Bypass Paths**: Eliminates potential security bypasses
- **Clear Security Model**: Single authentication paradigm

### 2. User Experience
- **Simplified Flow**: Clearer user journey without mode selection
- **Consistent Messaging**: Authentication requirements clearly communicated
- **Reduced Confusion**: No mixed messaging about authentication options

## Potential Risks and Edge Cases

### 1. Backward Compatibility
- Users with existing non-authenticated installations will need to reconfigure
- Documentation should be updated to explain the change

### 2. Error Handling
- Ensure appropriate error messages when unauthenticated users attempt access
- Provide clear guidance on authentication requirements

### 3. Testing Coverage
- Comprehensive testing needed to ensure all authz checks are removed
- Edge cases around authentication failures need thorough testing

## Success Metrics

### Code Quality Metrics
- Removal of all authz-related conditional logic
- Simplified component structure
- Reduced cyclomatic complexity in authentication components
- Better test coverage for authentication flows

### User Experience Metrics
- Streamlined setup process
- Clearer authentication requirements
- Consistent security model throughout application

## Documentation Updates Required

### User Documentation
- Update setup process documentation
- Remove references to non-authenticated mode
- Clarify authentication requirements

### Developer Documentation
- Update authentication architecture documentation
- Update component documentation to reflect authenticated-only mode
- Update testing documentation to remove non-authenticated test scenarios