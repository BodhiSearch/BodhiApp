---
title: 'User Management'
description: 'Admin dashboard for managing users and access requests'
order: 220
---

# User Management

## Overview

Bodhi App provides comprehensive user management for administrators and managers. From the Users page, you can manage existing users, approve access requests, assign roles, and control system access.

**Key Capabilities**:

- View all registered users
- Approve or reject access requests
- Assign and modify user roles
- Remove users from the system
- Track user registration and activity

**Access Requirements**: This page requires Manager or Admin role.

## Accessing User Management

**Navigation**: Settings → Users → <a href="/ui/users/" target="_blank" rel="noopener noreferrer">/ui/users/</a>

**URL**: <a href="http://localhost:1135/ui/users/" target="_blank" rel="noopener noreferrer">http://localhost:1135/ui/users/</a>

**Required Role**: Manager or Admin (PowerUser does not have access to user management)

## User List Tab

The Users tab displays all registered users in your Bodhi App instance.

<img
  src="/doc-images/users.jpg"
  alt="Users list showing username, email, role, and actions columns"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

### Table Columns

The user table displays:

- **Username**: User's identifier
- **Role**: Current role (User, PowerUser, Manager, Admin)
- **Actions**: Role modification and removal buttons

### Viewing Users

**Sorting**:

- Interactive sorting is not currently enabled
- Default sort order: Most recently updated users appear first
- Columns are not clickable for sorting

**Pagination**:

- Default page size: 10 users per page
- Navigate with page controls at bottom of table

**Search/Filter**:

- Search and filter capabilities are not currently available
- View all users through pagination

## Understanding User Roles

Bodhi App uses hierarchical role-based access control. Each role grants specific permissions, and higher roles inherit all lower role permissions.

### Role Hierarchy (Low to High)

1. **User**: Basic access to chat and embeddings APIs
2. **PowerUser**: Can download and delete model files, plus all User capabilities
3. **Manager**: Can manage users and approve access requests, plus all PowerUser capabilities (cannot manage other Admins)
4. **Admin**: Full system access, all permissions including managing all users
5. The role permission matrix is subject to change. We may allow some isolated capabilities for User and Manager role in future. Will keep you updated via the docs.

### Role Permission Matrix

| Feature                    | User | PowerUser | Manager | Admin |
| -------------------------- | ---- | --------- | ------- | ----- |
| Chat & Embeddings API      | ✅   | ✅        | ✅      | ✅    |
| Download Models            | ❌   | ✅        | ✅      | ✅    |
| Delete Models              | ❌   | ✅        | ✅      | ✅    |
| Create Local Model Aliases | ❌   | ✅        | ✅      | ✅    |
| Configure API Models       | ❌   | ✅        | ✅      | ✅    |
| Generate API Tokens        | ❌   | ✅        | ✅      | ✅    |
| User Management            | ❌   | ❌        | ✅\*    | ✅    |
| Access Request Approval    | ❌   | ❌        | ✅\*    | ✅    |
| View Settings              | ❌   | ❌        | ❌      | ✅    |
| Edit Settings              | ❌   | ❌        | ❌      | ✅    |
| System Configuration       | ❌   | ❌        | ❌      | ✅    |

\*Manager can only manage Users, PowerUsers, and other Managers (not Admins)

### Role Assignment Rules

- **Cannot modify users with higher role**: Managers cannot modify Admins (but can modify other Managers)
- **Cannot modify your own role**: Users cannot change their own role
- **Last admin protection**: The last admin in the system cannot downgrade their own role or remove themselves

## Modifying User Roles

Administrators and Managers can change user roles to grant or restrict permissions.

**Steps**:

1. Locate user in Users tab
2. Click the role dropdown in the Actions column
3. Select new role from list
4. Confirmation dialog appears
5. Confirm role change
6. Role updates immediately
7. User's active sessions are invalidated and user is logged out
8. User will see new permissions on next login

**Restrictions**:

- Cannot modify your own role
- Cannot assign a role higher than your own
- Cannot modify users with roles higher than yours

**Effects of Role Change**:

- User's permissions update immediately in the database
- All active sessions for that user are invalidated immediately
- User is automatically logged out from all sessions
- User must log in again to access Bodhi App with new role
- No explicit notification shown, but user will have new permissions after re-login
- Action is logged in server logs

## Removing Users

Remove users from the system when access should be permanently revoked.

**Steps**:

1. Locate user in Users tab
2. Click delete/remove icon in Actions column
3. Confirm removal in dialog
4. User is removed from system

**Effects of User Removal**:

- User account removed from system (soft delete)
- All active sessions terminated immediately
- **User's data is preserved**:
  - Chat history remains in the system
  - API tokens created by the user are preserved
  - Model files downloaded by the user remain
  - Model aliases created by the user are preserved
- User can request access again with the same email address
- If re-approved, user is treated as a new access request

**Warnings**:

- You cannot delete your own account
- The last admin in the system is protected from deletion
- User removal can be reversed by approving a new access request from the same user

## Access Requests Tab

The Access Requests tab displays all user access requests (pending, approved, and rejected).

<img
  src="/doc-images/request-all.jpg"
  alt="Access requests tab showing pending and historical requests"
  class="rounded-lg border-2 border-gray-200 dark:border-gray-700 shadow-lg hover:shadow-xl transition-shadow duration-300 max-w-[90%] mx-auto block"
/>

For the user-facing workflow, see [Access Requests Guide](/docs/features/access-requests).

### Viewing Access Requests

**Table Columns**:

- Username
- Email
- Status (Pending, Approved, Rejected)
- Requested Date
- Actions (Approve, Reject buttons for pending requests)

**Filtering**:

- Default view shows all requests (pending and historical)
- Most recently updated requests appear first

**Pagination**:

- Default page size: 10 requests per page
- Navigate with page controls at bottom of table

### Approving Access Requests

Grant system access to new users who have requested it.

**Steps**:

1. Locate pending request in Access Requests tab
2. Click "Approve" button
3. **Select role for the new user**:
   - Admin can assign any role (User, PowerUser, Manager, or Admin)
   - Manager can only assign User, PowerUser, or Manager roles (not Admin)
4. Confirm approval
5. Request status changes to "Approved"
6. User's existing session is invalidated and they are logged out
7. User can log in again and access Bodhi App with assigned role

**Role Selection**: Role is selected by the approver during the approval process (not automatically assigned)

### Rejecting Access Requests

Deny access to users who should not have system access.

**Steps**:

1. Locate pending request in Access Requests tab
2. Click "Reject" button
3. Confirm rejection (no rejection reason can be provided)
4. Request status changes to "Rejected"
5. User can see rejection status when they check the access request page
6. No notification is sent to the user

**User Can Re-request**: Yes, rejected users can submit new access requests. There is no cooldown period or maximum attempt limit.

### Request History

View all access requests regardless of status for audit purposes.

- Request history is retained indefinitely in the database
- Historical requests provide audit trail for user access management
- Server logs provide additional auditing beyond the requests table

## Best Practices

### User Approval

- Process access requests in a timely manner (no specific SLA defined)
- Verify user identity through your organization's authentication provider (OAuth)
- Start new users with User role, promote as needed based on their responsibilities
- Consider documenting your approval criteria in internal procedures

### Role Management

- Use principle of least privilege
- Grant minimum role required for user's tasks:
  - **User**: General users who only need chat and embedding access
  - **PowerUser**: Users who need to download and delete models
  - **Manager**: Trusted users who can help with user management
- Review user roles periodically as responsibilities change

### Security

- User activity is logged in server logs (no built-in activity dashboard currently)
- Sessions are automatically invalidated on role change - user must re-login
- Regularly review user list for inactive accounts
- No automatic inactivity timeout is currently enforced

## Troubleshooting

### User Cannot Login After Approval

**Symptoms**: User was approved but still cannot access Bodhi App

**Solutions**:

- Verify user status in table shows "Approved"
- User must logout completely and login again after approval
- Session is automatically cleared on approval
- If issue persists, check server logs for authentication errors

### Cannot Modify User Role

**Symptoms**: Role dropdown disabled or change fails

**Possible Causes**:

- You lack sufficient permissions (not Manager/Admin)
- Target user has role equal to or higher than yours
- Attempting to modify your own role
- Last admin protection (cannot modify the last admin's role)

### Access Request Not Appearing

**Symptoms**: User submitted request but admin doesn't see it

**Solutions**:

- Refresh the page to ensure latest data is displayed
- Verify the email address matches between user and request
- Check server logs for any submission errors
- Ensure database connection is working properly

### Common Error Messages

**"Cannot modify users with higher role"**: You can only modify users with your role or below in the hierarchy.

**"Cannot modify your own role"**: Users cannot change their own role for security reasons.

**"Last admin protected"**: The last admin in the system cannot be demoted or removed.

**"User not found"**: The user may have been deleted. Refresh the page to see current users.

## Related Documentation

- [Access Requests (User Guide)](/docs/features/access-requests) - User perspective
- [Authentication](/docs/intro#authentication) - OAuth2 setup
- [API Tokens](/docs/features/api-tokens) - Programmatic access
- [Settings](/docs/features/app-settings) - System configuration
