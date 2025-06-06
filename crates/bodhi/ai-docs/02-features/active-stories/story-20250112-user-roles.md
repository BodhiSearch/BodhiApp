# Add User Roles to User Info API

## User Story
As a Frontend Application  
I want to receive user roles from the user info API  
So that I can make role-based UI decisions

## Background
- User roles are available in JWT token claims under resource_access
- The roles are specific to the app's resource ID
- Current user info API returns only login status and email
- Frontend needs roles information for role-based menu display

## Acceptance Criteria

### Backend Changes 
- [x] Update UserInfo struct to include roles:
```rust
pub struct UserInfo {
    pub logged_in: bool,
    pub email: Option<String>,
    pub roles: Vec<String>,
}
```
- [x] Extract roles from JWT token claims:
  - [x] Get roles from `resource_access.[resource-id].roles` array
  - [x] Return empty array if user not logged in
  - [x] Return empty array if no roles found in token

### Frontend Changes 
- [x] Update UserInfo type to include roles:
```typescript
export interface UserInfo {
  logged_in: boolean;
  email?: string;
  roles: string[];
}
```

### Testing
- [x] Update test_user_info_handler_valid_token:
  - [x] Add roles to test token claims
  - [x] Verify roles are returned in response
- [x] Add test for token without roles
- [x] Add test for token with empty roles array
- [x] Test existing frontend components using useUser hook still work

## Technical Implementation Steps

### Backend Changes
1. Update UserInfo struct in `routes_login.rs`
2. Modify user_info_handler to extract roles:
   - Parse resource_access field from claims
   - Find roles array for app's resource ID
   - Return roles in UserInfo response

### Frontend Changes
1. Update UserInfo interface in `types/models.ts`
2. Verify useUser hook in `hooks/useQuery.ts` works with updated type

## Not In Scope
- Role-based helper functions
- Additional user info fields
- Role-based access control implementation
- Role validation or verification


## Implementation Status

### Completed
1. Backend Implementation:
   - Added `roles` field to UserInfo struct
   - Implemented role extraction from JWT claims
   - Added proper error handling for missing/invalid roles
   - Updated user_info_handler to use app's client ID
   
2. Backend Testing:
   - Added test cases for valid token with roles
   - Added test cases for missing resource_access field
   - Added test cases for empty roles
   - Added test cases for roles under different resource ID

3. Frontend Implementation:
   - Updated UserInfo type definition in models.ts
   - No changes needed for useUser hook as it automatically handles new fields

4. Testing:
   - Tested existing frontend components using useUser hook
   - Verified role-based UI behavior works as expected

## Technical Details

### JWT Claims Structure
```json
{
  "resource_access": {
    "[client-id]": {
      "roles": [
        "resource_manager",
        "resource_power_user",
        "resource_user",
        "resource_admin"
      ]
    }
  }
}
```

### Error Handling
- Empty roles array returned when:
  - User is not logged in
  - Token is missing resource_access field
  - Client ID not found in resource_access
  - Roles array is empty
